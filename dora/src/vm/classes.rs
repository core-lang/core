use parking_lot::RwLock;

use std::convert::From;
use std::iter::Iterator;
use std::sync::Arc;

use crate::language::sem_analysis::ClassDefinitionId;
use crate::language::ty::{SourceType, SourceTypeArray};
use crate::size::InstanceSize;
use crate::utils::{GrowableVec, Id};
use crate::vm::VM;
use crate::vtable::{ensure_display, VTableBox};

pub static DISPLAY_SIZE: usize = 6;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClassInstanceId(usize);

impl ClassInstanceId {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for ClassInstanceId {
    fn from(data: usize) -> ClassInstanceId {
        ClassInstanceId(data)
    }
}

impl GrowableVec<ClassInstance> {
    pub fn idx(&self, index: ClassInstanceId) -> Arc<ClassInstance> {
        self.idx_usize(index.0)
    }
}

impl Id for ClassInstance {
    type IdType = ClassInstanceId;

    fn id_to_usize(id: ClassInstanceId) -> usize {
        id.0 as usize
    }

    fn usize_to_id(value: usize) -> ClassInstanceId {
        ClassInstanceId(value.try_into().unwrap())
    }

    fn store_id(value: &mut ClassInstance, id: ClassInstanceId) {
        value.id = Some(id);
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    pub id: Option<ClassInstanceId>,
    pub cls_id: Option<ClassDefinitionId>,
    pub trait_object: Option<SourceType>,
    pub type_params: SourceTypeArray,
    pub parent_id: Option<ClassInstanceId>,
    pub fields: Vec<FieldInstance>,
    pub size: InstanceSize,
    pub ref_fields: Vec<i32>,
    pub vtable: RwLock<Option<VTableBox>>,
}

impl ClassInstance {
    pub fn id(&self) -> ClassInstanceId {
        self.id.expect("missing id")
    }

    pub fn name(&self, vm: &VM) -> String {
        if let Some(cls_id) = self.cls_id {
            let cls = vm.classes.idx(cls_id);
            let cls = cls.read();
            let name = vm.interner.str(cls.name);

            let params = if self.type_params.len() > 0 {
                self.type_params
                    .iter()
                    .map(|p| p.name_cls(vm, &*cls))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                return name.to_string();
            };

            format!("{}[{}]", name, params)
        } else {
            "<Unknown>".into()
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldInstance {
    pub offset: i32,
    pub ty: SourceType,
}

pub fn create_class_instance_with_vtable(
    vm: &VM,
    class_instance: ClassInstance,
    instance_size: usize,
    element_size: usize,
    vtable_entries: Vec<usize>,
) -> ClassInstanceId {
    let class_instance_id = vm.class_instances.push(class_instance);
    let class_instance = vm.class_instances.idx(class_instance_id);
    let class_instance_ptr = &*class_instance as *const ClassInstance as *mut ClassInstance;

    let mut vtable = VTableBox::new(
        class_instance_ptr,
        instance_size,
        element_size,
        &vtable_entries,
    );
    ensure_display(vm, &mut vtable, class_instance.parent_id);

    *class_instance.vtable.write() = Some(vtable);

    class_instance_id
}
