use parking_lot::RwLock;

use crate::error::msg::SemError;
use crate::semck;
use crate::semck::typeparamck::{self, ErrorReporting};
use crate::ty::BuiltinType;
use crate::vm::{EnumId, ExtensionId, Fct, FctKind, FctParent, FctSrc, FileId, NodeMap, VM};

use dora_parser::ast::visit::{self, Visitor};
use dora_parser::ast::{self, Ast};
use dora_parser::lexer::position::Position;

pub fn check<'ast>(vm: &mut VM<'ast>, ast: &'ast Ast, map_extension_defs: &NodeMap<ExtensionId>) {
    let mut clsck = ExtensionCheck {
        vm,
        ast,
        extension_id: None,
        map_extension_defs,
        file_id: 0,
        extension_ty: BuiltinType::Error,
    };

    clsck.check();
}

struct ExtensionCheck<'x, 'ast: 'x> {
    vm: &'x mut VM<'ast>,
    ast: &'ast ast::Ast,
    map_extension_defs: &'x NodeMap<ExtensionId>,
    file_id: u32,

    extension_id: Option<ExtensionId>,
    extension_ty: BuiltinType,
}

impl<'x, 'ast> ExtensionCheck<'x, 'ast> {
    fn check(&mut self) {
        self.visit_ast(self.ast);
    }

    fn visit_extension(&mut self, i: &'ast ast::Impl) {
        assert!(i.trait_type.is_none());
        self.extension_id = Some(*self.map_extension_defs.get(i.id).unwrap());

        self.vm.sym.lock().push_level();

        if let Some(ref type_params) = i.type_params {
            self.check_type_params(self.extension_id.unwrap(), type_params);
        }

        if let Some(extension_ty) = semck::read_type(self.vm, self.file_id.into(), &i.class_type) {
            self.extension_ty = extension_ty;

            match extension_ty {
                BuiltinType::Enum(enum_id, _) => {
                    let mut xenum = self.vm.enums[enum_id].write();
                    xenum.extensions.push(self.extension_id.unwrap());
                }

                _ => {
                    let cls_id = extension_ty.cls_id(self.vm).unwrap();
                    let cls = self.vm.classes.idx(cls_id);
                    let mut cls = cls.write();
                    cls.extensions.push(self.extension_id.unwrap());
                }
            }

            let mut extension = self.vm.extensions[self.extension_id.unwrap()].write();
            extension.ty = extension_ty;
        }

        visit::walk_impl(self, i);

        self.extension_id = None;
        self.vm.sym.lock().pop_level();
    }

    fn check_type_params(
        &mut self,
        extension_id: ExtensionId,
        _type_params: &'ast [ast::TypeParam],
    ) {
        let extension = &self.vm.extensions[extension_id];
        let extension = extension.read();

        let error = ErrorReporting::Yes(self.file_id.into(), extension.pos);
        typeparamck::check_enum(self.vm, extension.ty, error);
    }

    fn check_in_enum(&self, f: &ast::Function, enum_id: EnumId) -> bool {
        let xenum = self.vm.enums[enum_id].read();

        for &extension_id in &xenum.extensions {
            if !self.check_extension(f, extension_id) {
                return false;
            }
        }

        true
    }

    fn check_in_class(&self, f: &ast::Function) -> bool {
        let cls_id = self.extension_ty.cls_id(self.vm).unwrap();
        let cls = self.vm.classes.idx(cls_id);
        let cls = cls.read();

        for &method in &cls.methods {
            let method = self.vm.fcts.idx(method);
            let method = method.read();

            let is_static = f.annotation_usages.contains(
                self.vm
                    .annotations
                    .idx(self.vm.known.annotations.static_)
                    .read()
                    .name,
            );
            if method.name == f.name && method.is_static == is_static {
                let method_name = self.vm.interner.str(method.name).to_string();
                let msg = SemError::MethodExists(method_name, method.pos);
                self.vm.diag.lock().report(self.file_id.into(), f.pos, msg);
                return false;
            }
        }

        for &extension_id in &cls.extensions {
            if !self.check_extension(f, extension_id) {
                return false;
            }
        }

        true
    }

    fn check_extension(&self, f: &ast::Function, extension_id: ExtensionId) -> bool {
        let extension = self.vm.extensions[extension_id].read();

        if extension.ty.type_params(self.vm) != self.extension_ty.type_params(self.vm) {
            return true;
        }

        let is_static = f.annotation_usages.contains(
            self.vm
                .annotations
                .idx(self.vm.known.annotations.static_)
                .read()
                .name,
        );
        let table = if is_static {
            &extension.static_names
        } else {
            &extension.instance_names
        };

        if let Some(&method_id) = table.get(&f.name) {
            let method = self.vm.fcts.idx(method_id);
            let method = method.read();
            let method_name = self.vm.interner.str(method.name).to_string();
            let msg = SemError::MethodExists(method_name, method.pos);
            self.vm.diag.lock().report(self.file_id.into(), f.pos, msg);
            false
        } else {
            true
        }
    }
}

impl<'x, 'ast> Visitor<'ast> for ExtensionCheck<'x, 'ast> {
    fn visit_file(&mut self, f: &'ast ast::File) {
        visit::walk_file(self, f);
        self.file_id += 1;
    }

    fn visit_impl(&mut self, i: &'ast ast::Impl) {
        if i.trait_type.is_none() {
            self.visit_extension(i);
        }
    }

    fn visit_method(&mut self, f: &'ast ast::Function) {
        if self.extension_id.is_none() {
            return;
        }

        let extension_id = self.extension_id.unwrap();

        let internal = f.annotation_usages.contains(
            self.vm
                .annotations
                .idx(self.vm.known.annotations.internal)
                .read()
                .name,
        );
        if f.block.is_none() && !internal {
            report(
                self.vm,
                self.file_id.into(),
                f.pos,
                SemError::MissingFctBody,
            );
        }

        let kind = if internal {
            FctKind::Definition
        } else {
            FctKind::Source(RwLock::new(FctSrc::new()))
        };

        let parent = FctParent::Extension(extension_id);

        let fct = Fct::new(self.vm, f, self.file_id.into(), kind, parent, false);

        let is_static = fct.is_static;

        let fct_id = self.vm.add_fct(fct);

        if self.extension_ty.is_error() {
            return;
        }

        let success = match self.extension_ty {
            BuiltinType::Enum(enum_id, _) => self.check_in_enum(f, enum_id),
            _ => self.check_in_class(f),
        };

        if !success {
            return;
        }

        let mut extension = self.vm.extensions[extension_id].write();
        extension.methods.push(fct_id);

        let table = if is_static {
            &mut extension.static_names
        } else {
            &mut extension.instance_names
        };

        if !table.contains_key(&f.name) {
            table.insert(f.name, fct_id);
        }
    }
}

fn report(vm: &VM, file: FileId, pos: Position, msg: SemError) {
    vm.diag.lock().report(file, pos, msg);
}

#[cfg(test)]
mod tests {
    use crate::error::msg::SemError;
    use crate::semck::tests::*;

    #[test]
    fn extension_empty() {
        ok("class A impl A {}");
        ok("class A impl A {} impl A {}");
        err(
            "class A impl A[String] {}",
            pos(1, 14),
            SemError::WrongNumberTypeParams(0, 1),
        );

        ok("class A[T] impl A[Int32] {} impl A[String] {}");
        err(
            "class A[T: Zero] impl A[Int32] {} impl A[String] {}",
            pos(1, 40),
            SemError::TraitBoundNotSatisfied("String".into(), "Zero".into()),
        );
    }

    #[test]
    fn extension_method() {
        ok("class A impl A { fun foo() {} fun bar() {} }");
        err(
            "class A impl A { fun foo() {} fun foo() {} }",
            pos(1, 31),
            SemError::MethodExists("foo".into(), pos(1, 18)),
        );
    }

    #[test]
    fn extension_defined_twice() {
        err(
            "class A { fun foo() {} }
            impl A { fun foo() {} }",
            pos(2, 22),
            SemError::MethodExists("foo".into(), pos(1, 11)),
        );

        err(
            "class A
            impl A { fun foo() {} }
            impl A { fun foo() {} }",
            pos(3, 22),
            SemError::MethodExists("foo".into(), pos(2, 22)),
        );
    }

    #[test]
    fn extension_defined_twice_with_type_params_in_class() {
        err(
            "class Foo[T]
            impl Foo[Int32] { fun foo() {} }
            impl Foo[Int32] { fun foo() {} }",
            pos(3, 31),
            SemError::MethodExists("foo".into(), pos(2, 31)),
        );

        ok("class Foo[T]
            impl Foo[Int32] { fun foo() {} }
            impl Foo[Int64] { fun foo() {} }");
    }

    #[test]
    fn extension_with_illegal_type_param_in_class() {
        err(
            "trait MyTrait {}
            class Foo[T: MyTrait]
            impl Foo[String] {}
        ",
            pos(3, 18),
            SemError::TraitBoundNotSatisfied("String".into(), "MyTrait".into()),
        );
    }

    #[test]
    fn extension_enum() {
        ok("enum MyEnum { A, B } impl MyEnum {}");
        ok("enum MyEnum { A, B } impl MyEnum {} impl MyEnum {}");
        ok("enum MyEnum { A, B } impl MyEnum { fun foo() {} fun bar() {} }");

        err(
            "enum MyEnum { A, B } impl MyEnum { fun foo() {} fun foo() {} }",
            pos(1, 49),
            SemError::MethodExists("foo".into(), pos(1, 36)),
        );
    }
}
