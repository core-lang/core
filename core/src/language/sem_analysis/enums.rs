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
pub struct EnumDefinitionId(u32);

impl EnumDefinitionId {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Id for EnumDefinition {
    type IdType = EnumDefinitionId;

    fn id_to_usize(id: EnumDefinitionId) -> usize {
        id.0 as usize
    }

    fn usize_to_id(value: usize) -> EnumDefinitionId {
        EnumDefinitionId(value.try_into().unwrap())
    }

    fn store_id(value: &mut EnumDefinition, id: EnumDefinitionId) {
        value.id = Some(id);
    }
}

#[derive(Debug)]
pub struct EnumDefinition {
    pub id: Option<EnumDefinitionId>,
    pub package_id: PackageDefinitionId,
    pub module_id: ModuleDefinitionId,
    pub file_id: SourceFileId,
    pub ast: Arc<ast::Enum>,
    pub pos: Position,
    pub name: Name,
    pub visibility: Visibility,
    pub type_params: Option<TypeParamDefinition>,
    pub variants: Vec<EnumVariant>,
    pub name_to_value: HashMap<Name, u32>,
    pub impls: Vec<ImplDefinitionId>,
    pub extensions: Vec<ExtensionDefinitionId>,
    pub simple_enumeration: bool,
}

impl EnumDefinition {
    pub fn new(
        package_id: PackageDefinitionId,
        module_id: ModuleDefinitionId,
        file_id: SourceFileId,
        node: &Arc<ast::Enum>,
    ) -> EnumDefinition {
        EnumDefinition {
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
            name_to_value: HashMap::new(),
            impls: Vec::new(),
            extensions: Vec::new(),
            simple_enumeration: false,
        }
    }

    pub fn id(&self) -> EnumDefinitionId {
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

#[derive(Debug)]
pub struct EnumVariant {
    pub id: usize,
    pub name: Name,
    pub types: Vec<SourceType>,
}

pub fn find_methods_in_enum(
    sa: &SemAnalysis,
    object_type: SourceType,
    type_param_defs: &TypeParamDefinition,
    name: Name,
    is_static: bool,
    is_nullary: bool,
) -> Vec<Candidate> {
    let enum_id = object_type.enum_id().unwrap();
    let enum_ = sa.enums.idx(enum_id);
    let enum_ = enum_.read();

    for &extension_id in &enum_.extensions {
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
                let function = sa.fcts.idx(fct_id);
                let function = function.read();
                if function.is_nullary == is_nullary {
                    return vec![Candidate {
                        object_type: object_type.clone(),
                        container_type_params: bindings,
                        fct_id,
                    }];
                }
            }
        }
    }

    let mut candidates = Vec::new();

    for &impl_id in &enum_.impls {
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
