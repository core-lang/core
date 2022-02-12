use parking_lot::RwLock;

use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

use crate::language::ty::SourceType;
use crate::vm::{
    AnnotationDefinition, FctDefinitionId, FileId, NamespaceId, SemAnalysis, TypeParam, TypeParamId,
};

pub use self::matching::{extension_matches, extension_matches_ty};
use dora_parser::ast;
use dora_parser::interner::Name;
use dora_parser::lexer::position::Position;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExtensionId(u32);

impl From<usize> for ExtensionId {
    fn from(data: usize) -> ExtensionId {
        ExtensionId(data as u32)
    }
}

impl ExtensionId {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
pub struct ExtensionData {
    pub id: ExtensionId,
    pub file_id: FileId,
    pub ast: Arc<ast::Impl>,
    pub namespace_id: NamespaceId,
    pub pos: Position,
    pub type_params: Vec<TypeParam>,
    pub ty: SourceType,
    pub methods: Vec<FctDefinitionId>,
    pub instance_names: HashMap<Name, FctDefinitionId>,
    pub static_names: HashMap<Name, FctDefinitionId>,
}

impl ExtensionData {
    pub fn init_modifiers(&mut self, sa: &SemAnalysis) {
        let annotation_usages = &ast::AnnotationUsages::new();
        AnnotationDefinition::reject_modifiers(
            sa,
            annotation_usages,
            &[
                sa.known.annotations.abstract_,
                sa.known.annotations.final_,
                sa.known.annotations.internal,
                sa.known.annotations.open,
                sa.known.annotations.optimize_immediately,
                sa.known.annotations.override_,
                sa.known.annotations.pub_,
                sa.known.annotations.static_,
                sa.known.annotations.test,
            ],
        );
    }

    pub fn type_param(&self, id: TypeParamId) -> &TypeParam {
        &self.type_params[id.to_usize()]
    }
}

impl Index<ExtensionId> for Vec<RwLock<ExtensionData>> {
    type Output = RwLock<ExtensionData>;

    fn index(&self, index: ExtensionId) -> &RwLock<ExtensionData> {
        &self[index.to_usize()]
    }
}

mod matching {
    use crate::language::ty::{implements_trait, SourceType, SourceTypeArray};
    use crate::vm::{ExtensionId, TypeParam, TypeParamDefinition, VM};

    pub fn extension_matches(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        check_type_param_defs2: Option<&TypeParamDefinition>,
        extension_id: ExtensionId,
    ) -> Option<SourceTypeArray> {
        let extension = vm.extensions[extension_id].read();
        extension_matches_ty(
            vm,
            check_ty,
            check_type_param_defs,
            check_type_param_defs2,
            extension.ty.clone(),
            &extension.type_params,
        )
    }

    pub fn extension_matches_ty(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
    ) -> Option<SourceTypeArray> {
        let mut bindings = vec![None; ext_type_param_defs.len()];

        let result = matches(
            vm,
            check_ty,
            check_type_param_defs,
            check_type_param_defs2,
            ext_ty.clone(),
            ext_type_param_defs,
            None,
            &mut bindings,
        );

        if result {
            Some(SourceTypeArray::with(
                bindings.into_iter().map(|t| t.unwrap()).collect(),
            ))
        } else {
            None
        }
    }

    fn matches(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
        ext_type_param_defs2: Option<&TypeParamDefinition>,
        bindings: &mut [Option<SourceType>],
    ) -> bool {
        if let SourceType::TypeParam(tp_id) = ext_ty {
            let binding = bindings[tp_id.to_usize()].clone();

            if let Some(binding) = binding {
                compare_concrete_types(
                    vm,
                    check_ty,
                    check_type_param_defs,
                    check_type_param_defs2,
                    binding,
                    ext_type_param_defs,
                    ext_type_param_defs2,
                    bindings,
                )
            } else {
                let result = if check_ty.is_type_param() {
                    compare_type_param_bounds(
                        vm,
                        check_ty.clone(),
                        check_type_param_defs,
                        check_type_param_defs2,
                        ext_ty,
                        ext_type_param_defs,
                        ext_type_param_defs2,
                    )
                } else {
                    concrete_type_fulfills_bounds(
                        vm,
                        check_ty.clone(),
                        check_type_param_defs,
                        check_type_param_defs2,
                        ext_ty,
                        ext_type_param_defs,
                        ext_type_param_defs2,
                    )
                };

                bindings[tp_id.to_usize()] = Some(check_ty);

                result
            }
        } else {
            if check_ty.is_type_param() {
                false
            } else {
                compare_concrete_types(
                    vm,
                    check_ty,
                    check_type_param_defs,
                    check_type_param_defs2,
                    ext_ty,
                    ext_type_param_defs,
                    ext_type_param_defs2,
                    bindings,
                )
            }
        }
    }

    fn compare_type_param_bounds(
        _vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        _check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
        _ext_type_param_defs2: Option<&TypeParamDefinition>,
    ) -> bool {
        let ext_tp_id = ext_ty.type_param_id().expect("expected type param");
        let ext_tp_def = &ext_type_param_defs[ext_tp_id.to_usize()];

        let check_tp_id = check_ty.type_param_id().expect("expected type param");
        let check_tp_def = &check_type_param_defs[check_tp_id.to_usize()];

        for &trait_id in &ext_tp_def.trait_bounds {
            if !check_tp_def.trait_bounds.contains(&trait_id) {
                return false;
            }
        }

        true
    }

    fn concrete_type_fulfills_bounds(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        _check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
        _ext_type_param_defs2: Option<&TypeParamDefinition>,
    ) -> bool {
        let ext_tp_id = ext_ty.type_param_id().expect("expected type param");
        let ext_tp_def = &ext_type_param_defs[ext_tp_id.to_usize()];

        for &trait_id in &ext_tp_def.trait_bounds {
            if !implements_trait(vm, check_ty.clone(), check_type_param_defs, trait_id) {
                return false;
            }
        }

        true
    }

    fn compare_concrete_types(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
        ext_type_param_defs2: Option<&TypeParamDefinition>,
        bindings: &mut [Option<SourceType>],
    ) -> bool {
        match check_ty {
            SourceType::Unit
            | SourceType::Bool
            | SourceType::Char
            | SourceType::UInt8
            | SourceType::Int32
            | SourceType::Int64
            | SourceType::Float32
            | SourceType::Float64
            | SourceType::TypeParam(_) => check_ty == ext_ty,

            SourceType::Module(_) | SourceType::Lambda(_) | SourceType::Trait(_, _) => {
                unimplemented!()
            }

            SourceType::Tuple(check_tuple_id) => {
                let check_subtypes = vm.tuples.lock().get(check_tuple_id);

                let ext_tuple_id = if let Some(tuple_id) = ext_ty.tuple_id() {
                    tuple_id
                } else {
                    return false;
                };

                let ext_subtypes = vm.tuples.lock().get(ext_tuple_id);

                if check_subtypes.len() != ext_subtypes.len() {
                    return false;
                }

                for (check_subty, ext_subty) in check_subtypes.iter().zip(ext_subtypes.iter()) {
                    if !matches(
                        vm,
                        check_subty.clone(),
                        check_type_param_defs,
                        None,
                        ext_subty.clone(),
                        ext_type_param_defs,
                        None,
                        bindings,
                    ) {
                        return false;
                    }
                }

                true
            }

            SourceType::Struct(check_struct_id, _) => {
                let ext_struct_id = if let Some(struct_id) = ext_ty.struct_id() {
                    struct_id
                } else {
                    return false;
                };

                if check_struct_id != ext_struct_id {
                    return false;
                }

                compare_type_params(
                    vm,
                    check_ty,
                    check_type_param_defs,
                    check_type_param_defs2,
                    ext_ty,
                    ext_type_param_defs,
                    ext_type_param_defs2,
                    bindings,
                )
            }

            SourceType::Enum(check_enum_id, _) => {
                let ext_enum_id = if let Some(enum_id) = ext_ty.enum_id() {
                    enum_id
                } else {
                    return false;
                };

                if check_enum_id != ext_enum_id {
                    return false;
                }

                compare_type_params(
                    vm,
                    check_ty,
                    check_type_param_defs,
                    check_type_param_defs2,
                    ext_ty,
                    ext_type_param_defs,
                    ext_type_param_defs2,
                    bindings,
                )
            }

            SourceType::Class(check_cls_id, _) => {
                let ext_cls_id = if let Some(cls_id) = ext_ty.cls_id() {
                    cls_id
                } else {
                    return false;
                };

                if check_cls_id != ext_cls_id {
                    return false;
                }

                compare_type_params(
                    vm,
                    check_ty,
                    check_type_param_defs,
                    check_type_param_defs2,
                    ext_ty,
                    ext_type_param_defs,
                    ext_type_param_defs2,
                    bindings,
                )
            }

            SourceType::Ptr | SourceType::Error | SourceType::This | SourceType::Any => {
                unreachable!()
            }
        }
    }

    fn compare_type_params(
        vm: &VM,
        check_ty: SourceType,
        check_type_param_defs: &[TypeParam],
        check_type_param_defs2: Option<&TypeParamDefinition>,
        ext_ty: SourceType,
        ext_type_param_defs: &[TypeParam],
        ext_type_param_defs2: Option<&TypeParamDefinition>,
        bindings: &mut [Option<SourceType>],
    ) -> bool {
        let check_tps = check_ty.type_params();
        let ext_tps = ext_ty.type_params();

        assert_eq!(check_tps.len(), ext_tps.len());

        for (check_tp, ext_tp) in check_tps.iter().zip(ext_tps.iter()) {
            if !matches(
                vm,
                check_tp,
                check_type_param_defs,
                check_type_param_defs2,
                ext_tp,
                ext_type_param_defs,
                ext_type_param_defs2,
                bindings,
            ) {
                return false;
            }
        }

        true
    }
}
