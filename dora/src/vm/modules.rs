use parking_lot::RwLock;
use std::sync::Arc;

use crate::language::ty::SourceType;
use crate::size::InstanceSize;
use crate::utils::GrowableVec;
use crate::vm::{
    namespace_path, replace_type_param, AnnotationDefinition, Candidate, FctDefinitionId, Field,
    FieldDef, FileId, NamespaceId, SemAnalysis, TraitDefinitionId, VM,
};

use crate::vtable::VTableBox;
use dora_parser::ast;
use dora_parser::interner::Name;
use dora_parser::lexer::position::Position;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModuleId(usize);

impl ModuleId {
    pub fn max() -> ModuleId {
        ModuleId(usize::max_value())
    }
}

impl From<ModuleId> for usize {
    fn from(data: ModuleId) -> usize {
        data.0
    }
}

impl From<usize> for ModuleId {
    fn from(data: usize) -> ModuleId {
        ModuleId(data)
    }
}

impl GrowableVec<RwLock<Module>> {
    pub fn idx(&self, index: ModuleId) -> Arc<RwLock<Module>> {
        self.idx_usize(index.0)
    }
}

pub static DISPLAY_SIZE: usize = 6;

#[derive(Debug)]
pub struct Module {
    pub id: ModuleId,
    pub file_id: FileId,
    pub ast: Arc<ast::Module>,
    pub namespace_id: NamespaceId,
    pub pos: Position,
    pub name: Name,
    pub ty: SourceType,
    pub parent_class: Option<SourceType>,
    pub internal: bool,
    pub internal_resolved: bool,
    pub has_constructor: bool,
    pub is_pub: bool,

    pub constructor: Option<FctDefinitionId>,
    pub fields: Vec<Field>,
    pub methods: Vec<FctDefinitionId>,
    pub virtual_fcts: Vec<FctDefinitionId>,

    pub traits: Vec<TraitDefinitionId>,
}

impl Module {
    pub fn new(
        id: ModuleId,
        file_id: FileId,
        namespace_id: NamespaceId,
        ast: &Arc<ast::Module>,
        ty: SourceType,
        has_constructor: bool,
    ) -> Self {
        Module {
            id,
            file_id,
            ast: ast.clone(),
            namespace_id,
            pos: ast.pos,
            name: ast.name,
            ty,
            parent_class: None,
            internal: false,
            internal_resolved: false,
            has_constructor,
            is_pub: false,
            constructor: None,
            fields: Vec::new(),
            methods: Vec::new(),
            virtual_fcts: Vec::new(),
            traits: Vec::new(),
        }
    }

    pub fn init_modifiers(&mut self, sa: &SemAnalysis) {
        let annotation_usages = &ast::AnnotationUsages::new();
        self.internal = AnnotationDefinition::is_internal(annotation_usages, sa);
        self.is_pub = AnnotationDefinition::is_pub(annotation_usages, sa);
        AnnotationDefinition::reject_modifiers(
            sa,
            self.file_id,
            annotation_usages,
            &[
                sa.known.annotations.abstract_,
                sa.known.annotations.final_,
                sa.known.annotations.open,
                sa.known.annotations.optimize_immediately,
                sa.known.annotations.override_,
                sa.known.annotations.static_,
                sa.known.annotations.test,
            ],
        );
    }

    pub fn name(&self, vm: &VM) -> String {
        namespace_path(vm, self.namespace_id, self.name)
    }
}

pub fn find_methods_in_module(vm: &VM, object_type: SourceType, name: Name) -> Vec<Candidate> {
    let mut ignores = HashSet::new();

    let mut module_type = object_type;

    loop {
        let module_id = module_type.module_id().expect("no module");
        let module = vm.modules.idx(module_id);
        let module = module.read();

        for &method in &module.methods {
            let method = vm.fcts.idx(method);
            let method = method.read();

            if method.name == name {
                if let Some(overrides) = method.overrides {
                    ignores.insert(overrides);
                }

                if !ignores.contains(&method.id) {
                    return vec![Candidate {
                        object_type: module_type.clone(),
                        container_type_params: module_type.type_params(),
                        fct_id: method.id,
                    }];
                }
            }
        }

        if let Some(parent_class) = module.parent_class.clone() {
            let type_list = module_type.type_params();
            module_type = replace_type_param(vm, parent_class, &type_list, None);
        } else {
            break;
        }
    }

    Vec::new()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleDefId(usize);

impl ModuleDefId {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for ModuleDefId {
    fn from(data: usize) -> ModuleDefId {
        ModuleDefId(data)
    }
}

impl GrowableVec<RwLock<ModuleInstance>> {
    pub fn idx(&self, index: ModuleDefId) -> Arc<RwLock<ModuleInstance>> {
        self.idx_usize(index.0)
    }
}

#[derive(Debug)]
pub struct ModuleInstance {
    pub id: ModuleDefId,
    pub mod_id: Option<ModuleId>,
    pub parent_id: Option<ModuleDefId>,
    pub fields: Vec<FieldDef>,
    pub size: InstanceSize,
    pub ref_fields: Vec<i32>,
    pub vtable: Option<VTableBox>,
}

impl ModuleInstance {
    pub fn name(&self, vm: &VM) -> String {
        if let Some(module_id) = self.mod_id {
            let module = vm.modules.idx(module_id);
            let module = module.read();
            let name = vm.interner.str(module.name);

            format!("{}", name)
        } else {
            "<Unknown>".into()
        }
    }
}
