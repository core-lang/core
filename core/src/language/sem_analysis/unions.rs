use std::collections::hash_map::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use core_parser::ast;
use core_parser::interner::Name;
use core_parser::lexer::position::Position;

use crate::language::sem_analysis::{
    extension_matches, impl_matches, module_path, Candidate, ExtensionDefinitionId,
    ImplDefinitionId, ModuleDefinitionId, PackageDefinitionId, SemAnalysis, SourceFileId,
    TypeParamDefinition, Visibility,
};
use crate::language::ty::{SourceType, SourceTypeArray};
use crate::utils::Id;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionDefinitionId(u32);

impl UnionDefinitionId {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Id for UnionDefinition {
    type IdType = UnionDefinitionId;

    fn id_to_usize(id: UnionDefinitionId) -> usize {
        id.0 as usize
    }

    fn usize_to_id(value: usize) -> UnionDefinitionId {
        UnionDefinitionId(value.try_into().unwrap())
    }

    fn store_id(value: &mut UnionDefinition, id: UnionDefinitionId) {
        value.id = Some(id);
    }
}

#[derive(Debug)]
pub struct UnionDefinition {
    pub id: Option<UnionDefinitionId>,
    pub package_id: PackageDefinitionId,
    pub module_id: ModuleDefinitionId,
    pub file_id: SourceFileId,
    pub ast: Arc<ast::Union>,
    pub pos: Position,
    pub name: Name,
    pub visibility: Visibility,
    pub type_params: Option<TypeParamDefinition>,
    pub variants: Vec<UnionVariant>,
    pub impls: Vec<ImplDefinitionId>,
    pub extensions: Vec<ExtensionDefinitionId>,
    pub simple_enumeration: bool,
}

impl UnionDefinition {
    pub fn new(
        package_id: PackageDefinitionId,
        module_id: ModuleDefinitionId,
        file_id: SourceFileId,
        node: &Arc<ast::Union>,
    ) -> UnionDefinition {
        UnionDefinition {
            id: None,
            package_id,
            module_id,
            file_id,
            ast: node.clone(),
            pos: node.pos,
            name: node.name,
            type_params: None,
            visibility: Visibility::from_ast(node.visibility),
            variants: Vec::new(),
            impls: Vec::new(),
            extensions: Vec::new(),
            simple_enumeration: false,
        }
    }

    pub fn id(&self) -> UnionDefinitionId {
        self.id.expect("id missing")
    }

    pub fn type_params(&self) -> &TypeParamDefinition {
        self.type_params.as_ref().expect("uninitialized")
    }

    pub fn name(&self, sa: &SemAnalysis) -> String {
        module_path(sa, self.module_id, self.name)
    }

    pub fn name_with_params(&self, sa: &SemAnalysis, type_list: &SourceTypeArray) -> String {
        let name = sa.interner.str(self.name);

        if type_list.len() > 0 {
            let type_list = type_list
                .iter()
                .map(|p| p.name(sa))
                .collect::<Vec<_>>()
                .join(", ");

            format!("{}[{}]", name, type_list)
        } else {
            name.to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub id: usize,
    pub type_: SourceType,
}

pub fn find_methods_in_union(
    sa: &SemAnalysis,
    object_type: SourceType,
    type_param_defs: &TypeParamDefinition,
    name: Name,
    is_static: bool,
) -> Vec<Candidate> {
    let union_id = object_type.union_id().unwrap();
    let union_ = sa.unions.idx(union_id);
    let union_ = union_.read();

    for &extension_id in &union_.extensions {
        if let Some(bindings) =
            extension_matches(sa, object_type.clone(), type_param_defs, extension_id)
        {
            let extension = sa.extensions[extension_id].read();

            let table = if is_static {
                &extension.static_names
            } else {
                &extension.instance_names
            };

            if let Some(&fct_id) = table.get(&name) {
                return vec![Candidate {
                    object_type: object_type.clone(),
                    container_type_params: bindings,
                    fct_id,
                }];
            }
        }
    }

    let mut candidates = Vec::new();

    for &impl_id in &union_.impls {
        if let Some(bindings) = impl_matches(sa, object_type.clone(), type_param_defs, impl_id) {
            let impl_ = sa.impls[impl_id].read();

            let table = if is_static {
                &impl_.static_names
            } else {
                &impl_.instance_names
            };

            if let Some(&method_id) = table.get(&name) {
                candidates.push(Candidate {
                    object_type: object_type.clone(),
                    container_type_params: bindings.clone(),
                    fct_id: method_id,
                });
            }
        }
    }

    candidates
}
