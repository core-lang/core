use parking_lot::RwLock;
use std::sync::Arc;

use crate::cpu::{has_lzcnt, has_popcnt, has_tzcnt};
use crate::gc::Address;
use crate::mem;
use crate::object::Header;
use crate::size::InstanceSize;
use crate::stack;
use crate::stdlib;
use crate::sym::{NestedSymTable, TermSym, TypeSym};
use crate::ty::{SourceType, TypeList};
use crate::vm::{
    ClassDef, ClassDefId, ClassId, EnumId, FctId, Intrinsic, ModuleId, NamespaceId, TraitId, VM,
};
use crate::vtable::VTableBox;

pub fn resolve_internal_classes(vm: &mut VM) {
    vm.known.classes.unit = internal_class(vm, "Unit", Some(SourceType::Unit));
    vm.known.classes.bool = internal_class(vm, "Bool", Some(SourceType::Bool));

    vm.known.classes.uint8 = internal_class(vm, "UInt8", Some(SourceType::UInt8));
    vm.known.classes.char = internal_class(vm, "Char", Some(SourceType::Char));
    vm.known.classes.int32 = internal_class(vm, "Int32", Some(SourceType::Int32));
    vm.known.classes.int64 = internal_class(vm, "Int64", Some(SourceType::Int64));

    vm.known.classes.float32 = internal_class(vm, "Float32", Some(SourceType::Float32));
    vm.known.classes.float64 = internal_class(vm, "Float64", Some(SourceType::Float64));

    vm.known.classes.object = internal_class(vm, "Object", None);
    vm.known.classes.string = internal_class(vm, "String", None);
    vm.known.modules.string = internal_module(vm, "String", None);

    vm.known.classes.string_buffer = internal_class(vm, "StringBuffer", None);
    vm.known.modules.string_buffer = internal_module(vm, "StringBuffer", None);

    let cls = vm.classes.idx(vm.known.classes.string);
    let mut cls = cls.write();
    cls.is_str = true;

    vm.known.classes.array = internal_class(vm, "Array", None);
    vm.known.modules.array = internal_module(vm, "Array", None);

    let cls = vm.classes.idx(vm.known.classes.array);
    let mut cls = cls.write();
    cls.is_array = true;

    vm.known.classes.testing = internal_class(vm, "Testing", None);

    vm.known.classes.stacktrace = internal_class(vm, "Stacktrace", None);
    vm.known.classes.stacktrace_element = internal_class(vm, "StacktraceElement", None);

    vm.known.traits.stringable = find_trait(vm, "Stringable");
    vm.known.traits.zero = find_trait(vm, "Zero");
    vm.known.traits.iterator = find_trait(vm, "Iterator");

    vm.known.enums.option = find_enum(vm, "Option");

    internal_free_classes(vm);
}

pub fn fill_prelude(vm: &mut VM) {
    let symbols = [
        "Unit",
        "Bool",
        "UInt8",
        "Char",
        "Int32",
        "Int64",
        "Float32",
        "Float64",
        "Object",
        "String",
        "Array",
        "Vec",
        "print",
        "println",
        "Option",
        "unimplemented",
        "unreachable",
        "assert",
    ];

    let stdlib = vm.stdlib_namespace();
    let stdlib = stdlib.read();

    let prelude = vm.prelude_namespace();
    let mut prelude = prelude.write();

    for sym in &symbols {
        let name = vm.interner.intern(sym);
        let sym_term = stdlib.get_term(name);
        let sym_type = stdlib.get_type(name);

        assert!(sym_term.is_some() || sym_type.is_some());

        if let Some(sym_term) = sym_term {
            let old_sym = prelude.insert_term(name, sym_term);
            assert!(old_sym.is_none());
        }

        if let Some(sym_type) = sym_type {
            let old_sym = prelude.insert_type(name, sym_type);
            assert!(old_sym.is_none());
        }
    }

    let stdlib_name = vm.interner.intern("std");
    prelude.insert_term(stdlib_name, TermSym::Namespace(vm.stdlib_namespace_id));

    {
        // include None and Some from Option
        let option_name = vm.interner.intern("Option");
        let option_id = stdlib.get_type(option_name);

        match option_id {
            Some(TypeSym::Enum(enum_id)) => {
                let xenum = &vm.enums[enum_id];
                let xenum = xenum.read();

                for variant in &xenum.variants {
                    let old_sym =
                        prelude.insert_term(variant.name, TermSym::EnumValue(enum_id, variant.id));
                    assert!(old_sym.is_none());
                }
            }

            _ => unreachable!(),
        }
    }
}

pub fn discover_known_methods(vm: &mut VM) {
    vm.known.functions.string_buffer_empty =
        find_module_method(vm, vm.known.modules.string_buffer, "empty");
    vm.known.functions.string_buffer_append =
        find_class_method(vm, vm.known.classes.string_buffer, "append");
    vm.known.functions.string_buffer_to_string =
        find_class_method(vm, vm.known.classes.string_buffer, "toString");
}

fn internal_free_classes(vm: &mut VM) {
    let free_object: ClassDefId;
    let free_array: ClassDefId;

    {
        let mut class_defs = vm.class_defs.lock();
        let next = class_defs.len();

        free_object = next.into();
        free_array = (next + 1).into();

        class_defs.push(Arc::new(RwLock::new(ClassDef {
            id: free_object,
            cls_id: None,
            type_params: TypeList::empty(),
            parent_id: None,
            size: InstanceSize::Fixed(Header::size()),
            fields: Vec::new(),
            ref_fields: Vec::new(),
            vtable: None,
        })));

        class_defs.push(Arc::new(RwLock::new(ClassDef {
            id: free_array,
            cls_id: None,
            type_params: TypeList::empty(),
            parent_id: None,
            size: InstanceSize::FreeArray,
            fields: Vec::new(),
            ref_fields: Vec::new(),
            vtable: None,
        })));

        {
            let free_object_class_def = &class_defs[free_object.to_usize()];
            let mut free_object_class_def = free_object_class_def.write();
            let clsptr = (&*free_object_class_def) as *const ClassDef as *mut ClassDef;
            let vtable = VTableBox::new(clsptr, Header::size() as usize, 0, &[]);
            free_object_class_def.vtable = Some(vtable);
        }

        {
            let free_array_class_def = &class_defs[free_array.to_usize()];
            let mut free_array_class_def = free_array_class_def.write();
            let clsptr = (&*free_array_class_def) as *const ClassDef as *mut ClassDef;
            let vtable = VTableBox::new(clsptr, 0, mem::ptr_width_usize(), &[]);
            free_array_class_def.vtable = Some(vtable);
        }
    }

    vm.known.free_object_class_def = free_object;
    vm.known.free_array_class_def = free_array;
}

fn internal_class(vm: &mut VM, name: &str, ty: Option<SourceType>) -> ClassId {
    let iname = vm.interner.intern(name);
    let clsid = vm.stdlib_namespace().read().get_class(iname);

    if let Some(clsid) = clsid {
        let cls = vm.classes.idx(clsid);
        let mut cls = cls.write();

        if cls.internal {
            if let Some(ty) = ty {
                cls.ty = ty;
            }

            cls.internal_resolved = true;
        }

        clsid
    } else {
        panic!("class {} not found!", name);
    }
}

fn internal_module(vm: &mut VM, name: &str, ty: Option<SourceType>) -> ModuleId {
    let iname = vm.interner.intern(name);
    let module_id = vm.stdlib_namespace().read().get_module(iname);

    if let Some(module_id) = module_id {
        let module = vm.modules.idx(module_id);
        let mut module = module.write();

        if module.internal {
            if let Some(ty) = ty {
                module.ty = ty;
            }

            module.internal_resolved = true;
        }

        module_id
    } else {
        panic!("module {} not found!", name);
    }
}

fn find_trait(vm: &mut VM, name: &str) -> TraitId {
    let iname = vm.interner.intern(name);

    vm.stdlib_namespace()
        .read()
        .get_trait(iname)
        .expect("trait not found")
}

fn find_enum(vm: &mut VM, name: &str) -> EnumId {
    let iname = vm.interner.intern(name);

    vm.stdlib_namespace()
        .read()
        .get_enum(iname)
        .expect("enum not found")
}

pub fn resolve_internal_functions(vm: &mut VM) {
    let stdlib = vm.stdlib_namespace_id;
    native_fct(vm, "fatalError", stdlib::fatal_error as *const u8, stdlib);
    native_fct(vm, "abort", stdlib::abort as *const u8, stdlib);
    native_fct(vm, "exit", stdlib::exit as *const u8, stdlib);
    intrinsic_fct(vm, "unreachable", Intrinsic::Unreachable, stdlib);

    native_fct(vm, "print", stdlib::print as *const u8, stdlib);
    native_fct(vm, "println", stdlib::println as *const u8, stdlib);
    intrinsic_fct(vm, "assert", Intrinsic::Assert, stdlib);
    intrinsic_fct(vm, "debug", Intrinsic::Debug, stdlib);
    native_fct(vm, "argc", stdlib::argc as *const u8, stdlib);
    native_fct(vm, "argv", stdlib::argv as *const u8, stdlib);
    native_fct(vm, "forceCollect", stdlib::gc_collect as *const u8, stdlib);
    native_fct(vm, "timestamp", stdlib::timestamp as *const u8, stdlib);
    native_fct(
        vm,
        "forceMinorCollect",
        stdlib::gc_minor_collect as *const u8,
        stdlib,
    );
    native_fct(vm, "sleep", stdlib::sleep as *const u8, stdlib);
    native_fct(
        vm,
        "encodedBytecode",
        stdlib::bytecode as *const u8,
        vm.boots_namespace_id,
    );

    native_fct(vm, "call", stdlib::call as *const u8, stdlib);

    intrinsic_fct(vm, "unsafeKillRefs", Intrinsic::UnsafeKillRefs, stdlib);
    intrinsic_fct(vm, "unsafeIsNull", Intrinsic::UnsafeIsNull, stdlib);
    intrinsic_fct(
        vm,
        "unsafeLoadEnumVariant",
        Intrinsic::UnsafeLoadEnumVariant,
        stdlib,
    );
    intrinsic_fct(
        vm,
        "unsafeLoadEnumElement",
        Intrinsic::UnsafeLoadEnumElement,
        stdlib,
    );

    let clsid = vm.known.classes.uint8;
    native_class_method(vm, clsid, "toString", stdlib::uint8_to_string as *const u8);

    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::ByteToInt64);
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::ByteToInt32);
    intrinsic_class_method(vm, clsid, "toChar", Intrinsic::ByteToChar);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::ByteEq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::ByteCmp);
    intrinsic_class_method(vm, clsid, "not", Intrinsic::ByteNot);

    let clsid = vm.known.classes.char;
    native_class_method(vm, clsid, "toString", stdlib::char_to_string as *const u8);

    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::CharToInt64);
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::CharToInt32);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::CharEq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::CharCmp);

    let clsid = vm.known.classes.int32;
    native_class_method(vm, clsid, "toString", stdlib::int32_to_string as *const u8);

    intrinsic_class_method(vm, clsid, "toUInt8", Intrinsic::Int32ToByte);
    intrinsic_class_method(vm, clsid, "toCharUnchecked", Intrinsic::Int32ToChar);
    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::Int32ToInt64);

    intrinsic_class_method(vm, clsid, "toFloat32", Intrinsic::Int32ToFloat32);
    intrinsic_class_method(vm, clsid, "toFloat64", Intrinsic::Int32ToFloat64);

    intrinsic_class_method(vm, clsid, "asFloat32", Intrinsic::ReinterpretInt32AsFloat32);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::Int32Eq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::Int32Cmp);

    intrinsic_class_method(vm, clsid, "plus", Intrinsic::Int32Add);
    intrinsic_class_method(vm, clsid, "minus", Intrinsic::Int32Sub);
    intrinsic_class_method(vm, clsid, "times", Intrinsic::Int32Mul);
    intrinsic_class_method(vm, clsid, "div", Intrinsic::Int32Div);
    intrinsic_class_method(vm, clsid, "mod", Intrinsic::Int32Mod);

    intrinsic_class_method(vm, clsid, "bitwiseOr", Intrinsic::Int32Or);
    intrinsic_class_method(vm, clsid, "bitwiseAnd", Intrinsic::Int32And);
    intrinsic_class_method(vm, clsid, "bitwiseXor", Intrinsic::Int32Xor);

    intrinsic_class_method(vm, clsid, "shiftLeft", Intrinsic::Int32Shl);
    intrinsic_class_method(vm, clsid, "shiftRight", Intrinsic::Int32Shr);
    intrinsic_class_method(vm, clsid, "shiftRightSigned", Intrinsic::Int32Sar);

    intrinsic_class_method(vm, clsid, "rotateLeft", Intrinsic::Int32RotateLeft);
    intrinsic_class_method(vm, clsid, "rotateRight", Intrinsic::Int32RotateRight);

    intrinsic_class_method(vm, clsid, "unaryPlus", Intrinsic::Int32Plus);
    intrinsic_class_method(vm, clsid, "unaryMinus", Intrinsic::Int32Neg);
    intrinsic_class_method(vm, clsid, "not", Intrinsic::Int32Not);

    let clsid = vm.known.classes.int64;
    native_class_method(vm, clsid, "toString", stdlib::int64_to_string as *const u8);

    intrinsic_class_method(vm, clsid, "toCharUnchecked", Intrinsic::Int64ToChar);
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::Int64ToInt32);
    intrinsic_class_method(vm, clsid, "toUInt8", Intrinsic::Int64ToByte);

    intrinsic_class_method(vm, clsid, "toFloat32", Intrinsic::Int64ToFloat32);
    intrinsic_class_method(vm, clsid, "toFloat64", Intrinsic::Int64ToFloat64);

    intrinsic_class_method(vm, clsid, "asFloat64", Intrinsic::ReinterpretInt64AsFloat64);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::Int64Eq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::Int64Cmp);

    intrinsic_class_method(vm, clsid, "plus", Intrinsic::Int64Add);
    intrinsic_class_method(vm, clsid, "minus", Intrinsic::Int64Sub);
    intrinsic_class_method(vm, clsid, "times", Intrinsic::Int64Mul);
    intrinsic_class_method(vm, clsid, "div", Intrinsic::Int64Div);
    intrinsic_class_method(vm, clsid, "mod", Intrinsic::Int64Mod);

    intrinsic_class_method(vm, clsid, "bitwiseOr", Intrinsic::Int64Or);
    intrinsic_class_method(vm, clsid, "bitwiseAnd", Intrinsic::Int64And);
    intrinsic_class_method(vm, clsid, "bitwiseXor", Intrinsic::Int64Xor);

    intrinsic_class_method(vm, clsid, "shiftLeft", Intrinsic::Int64Shl);
    intrinsic_class_method(vm, clsid, "shiftRight", Intrinsic::Int64Shr);
    intrinsic_class_method(vm, clsid, "shiftRightSigned", Intrinsic::Int64Sar);

    intrinsic_class_method(vm, clsid, "rotateLeft", Intrinsic::Int64RotateLeft);
    intrinsic_class_method(vm, clsid, "rotateRight", Intrinsic::Int64RotateRight);

    intrinsic_class_method(vm, clsid, "unaryPlus", Intrinsic::Int64Plus);
    intrinsic_class_method(vm, clsid, "unaryMinus", Intrinsic::Int64Neg);
    intrinsic_class_method(vm, clsid, "not", Intrinsic::Int64Not);

    let clsid = vm.known.classes.bool;
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::BoolToInt32);
    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::BoolToInt64);
    intrinsic_class_method(vm, clsid, "equals", Intrinsic::BoolEq);
    intrinsic_class_method(vm, clsid, "not", Intrinsic::BoolNot);

    let clsid = vm.known.classes.string;
    native_class_method(vm, clsid, "compareTo", stdlib::strcmp as *const u8);
    native_class_method(
        vm,
        clsid,
        "toInt32Success",
        stdlib::str_to_int32_success as *const u8,
    );
    native_class_method(
        vm,
        clsid,
        "toInt64Success",
        stdlib::str_to_int64_success as *const u8,
    );
    native_class_method(
        vm,
        clsid,
        "toInt32OrZero",
        stdlib::str_to_int32 as *const u8,
    );
    native_class_method(
        vm,
        clsid,
        "toInt64OrZero",
        stdlib::str_to_int64 as *const u8,
    );
    native_class_method(vm, clsid, "plus", stdlib::strcat as *const u8);

    intrinsic_class_method(vm, clsid, "size", Intrinsic::StrLen);
    intrinsic_class_method(vm, clsid, "getByte", Intrinsic::StrGet);
    native_class_method(vm, clsid, "clone", stdlib::str_clone as *const u8);

    let clsid = vm.known.classes.float32;
    native_class_method(
        vm,
        clsid,
        "toString",
        stdlib::float32_to_string as *const u8,
    );
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::Float32ToInt32);
    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::Float32ToInt64);
    intrinsic_class_method(vm, clsid, "toFloat64", Intrinsic::PromoteFloat32ToFloat64);

    intrinsic_class_method(vm, clsid, "asInt32", Intrinsic::ReinterpretFloat32AsInt32);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::Float32Eq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::Float32Cmp);

    intrinsic_class_method(vm, clsid, "plus", Intrinsic::Float32Add);
    intrinsic_class_method(vm, clsid, "minus", Intrinsic::Float32Sub);
    intrinsic_class_method(vm, clsid, "times", Intrinsic::Float32Mul);
    intrinsic_class_method(vm, clsid, "div", Intrinsic::Float32Div);

    intrinsic_class_method(vm, clsid, "unaryPlus", Intrinsic::Float32Plus);
    intrinsic_class_method(vm, clsid, "unaryMinus", Intrinsic::Float32Neg);

    intrinsic_class_method(vm, clsid, "isNan", Intrinsic::Float32IsNan);
    intrinsic_class_method(vm, clsid, "sqrt", Intrinsic::Float32Sqrt);

    let clsid = vm.known.classes.float64;
    native_class_method(
        vm,
        clsid,
        "toString",
        stdlib::float64_to_string as *const u8,
    );
    intrinsic_class_method(vm, clsid, "toInt32", Intrinsic::Float64ToInt32);
    intrinsic_class_method(vm, clsid, "toInt64", Intrinsic::Float64ToInt64);
    intrinsic_class_method(vm, clsid, "toFloat32", Intrinsic::DemoteFloat64ToFloat32);

    intrinsic_class_method(vm, clsid, "asInt64", Intrinsic::ReinterpretFloat64AsInt64);

    intrinsic_class_method(vm, clsid, "equals", Intrinsic::Float64Eq);
    intrinsic_class_method(vm, clsid, "compareTo", Intrinsic::Float64Cmp);

    intrinsic_class_method(vm, clsid, "plus", Intrinsic::Float64Add);
    intrinsic_class_method(vm, clsid, "minus", Intrinsic::Float64Sub);
    intrinsic_class_method(vm, clsid, "times", Intrinsic::Float64Mul);
    intrinsic_class_method(vm, clsid, "div", Intrinsic::Float64Div);

    intrinsic_class_method(vm, clsid, "unaryPlus", Intrinsic::Float64Plus);
    intrinsic_class_method(vm, clsid, "unaryMinus", Intrinsic::Float64Neg);

    intrinsic_class_method(vm, clsid, "isNan", Intrinsic::Float64IsNan);
    intrinsic_class_method(vm, clsid, "sqrt", Intrinsic::Float64Sqrt);

    let module_id = vm.known.modules.string;
    native_module_method(
        vm,
        module_id,
        "fromBytesPart",
        stdlib::str_from_bytes as *const u8,
    );
    native_module_method(
        vm,
        module_id,
        "fromStringPart",
        stdlib::str_from_bytes as *const u8,
    );

    let clsid = vm.known.classes.array;
    intrinsic_ctor(vm, clsid, Intrinsic::ArrayWithValues);
    intrinsic_class_method(vm, clsid, "size", Intrinsic::ArrayLen);
    intrinsic_class_method(vm, clsid, "get", Intrinsic::ArrayGet);
    intrinsic_class_method(vm, clsid, "set", Intrinsic::ArraySet);

    let module_id = vm.known.modules.array;
    intrinsic_module_method(vm, module_id, "ofSizeUnsafe", Intrinsic::ArrayNewOfSize);

    let clsid = vm.known.classes.stacktrace;
    native_class_method(
        vm,
        clsid,
        "retrieveStacktrace",
        stack::retrieve_stack_trace as *const u8,
    );
    native_class_method(
        vm,
        clsid,
        "getStacktraceElement",
        stack::stack_element as *const u8,
    );

    let iname = vm.interner.intern("Thread");
    let clsid = vm.stdlib_namespace().read().get_class(iname);

    if let Some(clsid) = clsid {
        native_class_method(vm, clsid, "start", stdlib::spawn_thread as *const u8);
    }

    install_conditional_intrinsics(vm);

    intrinsic_impl_enum(vm, vm.known.enums.option, "isNone", Intrinsic::OptionIsNone);

    intrinsic_impl_enum(vm, vm.known.enums.option, "isSome", Intrinsic::OptionIsSome);

    intrinsic_impl_enum(vm, vm.known.enums.option, "unwrap", Intrinsic::OptionUnwrap);
}

fn native_class_method(vm: &mut VM, clsid: ClassId, name: &str, fctptr: *const u8) {
    internal_class_method(
        vm,
        clsid,
        name,
        FctImplementation::Native(Address::from_ptr(fctptr)),
    );
}

fn intrinsic_ctor(vm: &mut VM, clsid: ClassId, intrinsic: Intrinsic) {
    let cls = vm.classes.idx(clsid);
    let cls = cls.read();

    if let Some(ctor_id) = cls.constructor {
        let ctor = vm.fcts.idx(ctor_id);
        let mut ctor = ctor.write();

        ctor.intrinsic = Some(intrinsic);
    } else {
        panic!("no constructor");
    }
}

fn intrinsic_class_method(vm: &mut VM, clsid: ClassId, name: &str, intrinsic: Intrinsic) {
    internal_class_method(vm, clsid, name, FctImplementation::Intrinsic(intrinsic));
}

fn internal_class_method(vm: &mut VM, clsid: ClassId, name: &str, kind: FctImplementation) {
    let cls = vm.classes.idx(clsid);
    let cls = cls.read();
    let name = vm.interner.intern(name);

    for &mid in &cls.methods {
        let mtd = vm.fcts.idx(mid);
        let mut mtd = mtd.write();

        if mtd.name == name {
            if mtd.internal {
                match kind {
                    FctImplementation::Intrinsic(intrinsic) => mtd.intrinsic = Some(intrinsic),
                    FctImplementation::Native(address) => {
                        mtd.native_pointer = Some(address);
                    }
                }
                mtd.internal_resolved = true;
                break;
            } else {
                panic!(
                    "method {} found, but was not @internal",
                    vm.interner.str(name)
                )
            }
        }
    }
}

fn find_class_method(vm: &VM, clsid: ClassId, name: &str) -> FctId {
    let cls = vm.classes.idx(clsid);
    let cls = cls.read();
    let intern_name = vm.interner.intern(name);

    for &mid in &cls.methods {
        let mtd = vm.fcts.idx(mid);
        let mtd = mtd.read();

        if mtd.name == intern_name {
            return mid;
        }
    }

    panic!("cannot find class method `{}`", name)
}

fn native_module_method(vm: &mut VM, module_id: ModuleId, name: &str, fctptr: *const u8) {
    internal_module_method(
        vm,
        module_id,
        name,
        FctImplementation::Native(Address::from_ptr(fctptr)),
    );
}

fn intrinsic_module_method(vm: &mut VM, module_id: ModuleId, name: &str, intrinsic: Intrinsic) {
    internal_module_method(vm, module_id, name, FctImplementation::Intrinsic(intrinsic));
}

fn internal_module_method(vm: &mut VM, module_id: ModuleId, name: &str, kind: FctImplementation) {
    let module = vm.modules.idx(module_id);
    let module = module.read();
    let name = vm.interner.intern(name);

    for &mid in &module.methods {
        let mtd = vm.fcts.idx(mid);
        let mut mtd = mtd.write();

        if mtd.name == name {
            if mtd.internal {
                match kind {
                    FctImplementation::Intrinsic(intrinsic) => mtd.intrinsic = Some(intrinsic),
                    FctImplementation::Native(address) => {
                        mtd.native_pointer = Some(address);
                    }
                }
                mtd.internal_resolved = true;
                break;
            } else {
                panic!(
                    "method {} found, but was not @internal",
                    vm.interner.str(name)
                )
            }
        }
    }
}

fn find_module_method(vm: &VM, module_id: ModuleId, name: &str) -> FctId {
    let module = vm.modules.idx(module_id);
    let module = module.read();
    let intern_name = vm.interner.intern(name);

    for &mid in &module.methods {
        let mtd = vm.fcts.idx(mid);
        let mtd = mtd.read();

        if mtd.name == intern_name {
            return mid;
        }
    }

    panic!("cannot find module method `{}`", name)
}

fn native_fct(vm: &mut VM, name: &str, fctptr: *const u8, namespace_id: NamespaceId) {
    internal_fct(
        vm,
        name,
        FctImplementation::Native(Address::from_ptr(fctptr)),
        namespace_id,
    );
}

fn intrinsic_fct(vm: &mut VM, name: &str, intrinsic: Intrinsic, namespace_id: NamespaceId) {
    internal_fct(
        vm,
        name,
        FctImplementation::Intrinsic(intrinsic),
        namespace_id,
    );
}

fn internal_fct(vm: &mut VM, name: &str, kind: FctImplementation, namespace_id: NamespaceId) {
    let name = vm.interner.intern(name);
    let fctid = NestedSymTable::new(vm, namespace_id).get_fct(name);

    if let Some(fctid) = fctid {
        let fct = vm.fcts.idx(fctid);
        let mut fct = fct.write();

        if fct.internal {
            match kind {
                FctImplementation::Intrinsic(intrinsic) => fct.intrinsic = Some(intrinsic),
                FctImplementation::Native(address) => {
                    fct.native_pointer = Some(address);
                }
            }
            fct.internal_resolved = true;
        }
    }
}

enum FctImplementation {
    Intrinsic(Intrinsic),
    Native(Address),
}

fn intrinsic_impl(vm: &mut VM, clsid: ClassId, tid: TraitId, name: &str, intrinsic: Intrinsic) {
    internal_impl(
        vm,
        clsid,
        tid,
        name,
        FctImplementation::Intrinsic(intrinsic),
    );
}

fn internal_impl(vm: &mut VM, clsid: ClassId, tid: TraitId, name: &str, kind: FctImplementation) {
    let name = vm.interner.intern(name);
    let cls = vm.classes.idx(clsid);
    let cls = cls.read();

    for &iid in &cls.impls {
        let i = vm.impls[iid].read();

        if Some(tid) == i.trait_id {
            for &fid in &i.methods {
                let fct = vm.fcts.idx(fid);
                let mut fct = fct.write();

                if fct.internal && fct.name == name {
                    match kind {
                        FctImplementation::Intrinsic(intrinsic) => fct.intrinsic = Some(intrinsic),
                        FctImplementation::Native(address) => {
                            fct.native_pointer = Some(address);
                        }
                    }
                    fct.internal_resolved = true;
                    return;
                }
            }
        }
    }
}

fn intrinsic_impl_enum(vm: &mut VM, enum_id: EnumId, name: &str, intrinsic: Intrinsic) {
    internal_impl_enum(vm, enum_id, name, FctImplementation::Intrinsic(intrinsic));
}

fn internal_impl_enum(vm: &mut VM, enum_id: EnumId, name: &str, kind: FctImplementation) {
    let name = vm.interner.intern(name);
    let xenum = &vm.enums[enum_id];
    let xenum = xenum.read();

    for &eid in &xenum.extensions {
        let ext = vm.extensions[eid].read();

        for &fid in &ext.methods {
            let fct = vm.fcts.idx(fid);
            let mut fct = fct.write();

            if fct.name == name {
                match kind {
                    FctImplementation::Intrinsic(intrinsic) => fct.intrinsic = Some(intrinsic),
                    FctImplementation::Native(address) => {
                        fct.native_pointer = Some(address);
                    }
                }
                fct.internal_resolved = true;
                return;
            }
        }
    }
}

fn install_conditional_intrinsics(vm: &mut VM) {
    let clsid = vm.known.classes.int32;
    if has_popcnt() {
        intrinsic_class_method(vm, clsid, "countZeroBits", Intrinsic::Int32CountZeroBits);
        intrinsic_class_method(vm, clsid, "countOneBits", Intrinsic::Int32CountOneBits);
    }
    if has_lzcnt() {
        intrinsic_class_method(
            vm,
            clsid,
            "countZeroBitsLeading",
            Intrinsic::Int32CountZeroBitsLeading,
        );
        intrinsic_class_method(
            vm,
            clsid,
            "countOneBitsLeading",
            Intrinsic::Int32CountOneBitsLeading,
        );
    }
    if has_tzcnt() {
        intrinsic_class_method(
            vm,
            clsid,
            "countZeroBitsTrailing",
            Intrinsic::Int32CountZeroBitsTrailing,
        );
        intrinsic_class_method(
            vm,
            clsid,
            "countOneBitsTrailing",
            Intrinsic::Int32CountOneBitsTrailing,
        );
    }

    let clsid = vm.known.classes.int64;
    if has_popcnt() {
        intrinsic_class_method(vm, clsid, "countZeroBits", Intrinsic::Int64CountZeroBits);
        intrinsic_class_method(vm, clsid, "countOneBits", Intrinsic::Int64CountOneBits);
    }
    if has_lzcnt() {
        intrinsic_class_method(
            vm,
            clsid,
            "countZeroBitsLeading",
            Intrinsic::Int64CountZeroBitsLeading,
        );
        intrinsic_class_method(
            vm,
            clsid,
            "countOneBitsLeading",
            Intrinsic::Int64CountOneBitsLeading,
        );
    }
    if has_tzcnt() {
        intrinsic_class_method(
            vm,
            clsid,
            "countZeroBitsTrailing",
            Intrinsic::Int64CountZeroBitsTrailing,
        );
        intrinsic_class_method(
            vm,
            clsid,
            "countOneBitsTrailing",
            Intrinsic::Int64CountOneBitsTrailing,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::semck::tests::*;

    #[test]
    fn builtin_functions() {
        ok("fun f() { assert(true); }");
        ok("fun f() { print(\"test\"); }");
        ok("fun f() { println(\"test\"); }");
    }
}
