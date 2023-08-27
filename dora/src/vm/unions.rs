use parking_lot::RwLock;

use crate::language::sem_analysis::{UnionDefinition, UnionDefinitionId};
use crate::language::ty::{SourceType, SourceTypeArray};
use crate::utils::Id;
use crate::vm::ClassInstanceId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionInstanceId(u32);

impl Id for UnionInstance {
    type IdType = UnionInstanceId;

    fn id_to_usize(id: UnionInstanceId) -> usize {
        id.0 as usize
    }

    fn usize_to_id(value: usize) -> UnionInstanceId {
        UnionInstanceId(value.try_into().unwrap())
    }

    fn store_id(_value: &mut UnionInstance, _id: UnionInstanceId) {}
}

#[derive(Debug)]
pub struct UnionInstance {
    pub union_id: UnionDefinitionId,
    pub type_params: SourceTypeArray,
    pub layout: UnionLayout,
    pub variants: RwLock<Vec<Option<ClassInstanceId>>>,
}

impl UnionInstance {
    pub fn field_id(
        &self,
        union: &UnionDefinition,
        variant_idx: usize,
        element_idx: usize,
    ) -> usize {
        let variant = &union.variants[variant_idx];
        let mut units = 0;

        for ty in &variant.types[0..element_idx as usize] {
            if ty.is_unit() {
                units += 1;
            }
        }

        1 + element_idx - units
    }
}

#[derive(Copy, Clone, Debug)]
pub enum UnionLayout {
    Int,
    Ptr,
    Tagged,
}

#[derive(Debug)]
pub struct UnionInstanceVariant {
    pub types: Vec<SourceType>,
}
