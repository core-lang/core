use crate::ty::SourceType;
use crate::utils::GrowableVec;
use crate::vm::{FileId, NamespaceId, SemAnalysis, TypeParam};
use dora_parser::ast::{AnnotationParam, AnnotationUsages, Modifier};
use dora_parser::interner::Name;
use dora_parser::Position;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AnnotationDefinitionId(usize);

impl AnnotationDefinitionId {
    pub fn max() -> AnnotationDefinitionId {
        AnnotationDefinitionId(usize::max_value())
    }
}

impl From<AnnotationDefinitionId> for usize {
    fn from(data: AnnotationDefinitionId) -> usize {
        data.0
    }
}

impl From<usize> for AnnotationDefinitionId {
    fn from(data: usize) -> AnnotationDefinitionId {
        AnnotationDefinitionId(data)
    }
}

impl GrowableVec<RwLock<AnnotationDefinition>> {
    pub fn idx(&self, index: AnnotationDefinitionId) -> Arc<RwLock<AnnotationDefinition>> {
        self.idx_usize(index.0)
    }
}

#[derive(Debug)]
pub struct AnnotationDefinition {
    pub id: AnnotationDefinitionId,
    pub file_id: FileId,
    pub pos: Position,
    pub name: Name,
    pub namespace_id: NamespaceId,
    pub ty: SourceType,
    pub internal_annotation: Option<Modifier>,

    pub type_params: Option<Vec<TypeParam>>,
    pub term_params: Option<Vec<AnnotationParam>>,
}

impl AnnotationDefinition {
    pub fn new(
        id: AnnotationDefinitionId,
        file_id: FileId,
        pos: Position,
        name: Name,
        namespace_id: NamespaceId,
    ) -> AnnotationDefinition {
        AnnotationDefinition {
            id,
            file_id,
            pos,
            name,
            namespace_id,
            ty: SourceType::Error,
            internal_annotation: None,
            type_params: None,
            term_params: None,
        }
    }

    pub fn internal(&self) -> bool {
        self.internal_annotation.is_some()
    }

    pub fn is_abstract(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa
            .annotations
            .idx(sa.known.annotations.abstract_)
            .read()
            .name;
        annotation_usages.contains(name)
    }
    pub fn is_final(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.final_).read().name;
        annotation_usages.contains(name)
    }
    pub fn is_internal(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa
            .annotations
            .idx(sa.known.annotations.internal)
            .read()
            .name;
        annotation_usages.contains(name)
    }
    pub fn is_open(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.open).read().name;
        annotation_usages.contains(name)
    }
    pub fn is_override(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa
            .annotations
            .idx(sa.known.annotations.override_)
            .read()
            .name;
        annotation_usages.contains(name)
    }
    pub fn is_pub(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.pub_).read().name;
        annotation_usages.contains(name)
    }
    pub fn is_static(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.static_).read().name;
        annotation_usages.contains(name)
    }

    pub fn is_test(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.test).read().name;
        annotation_usages.contains(name)
    }

    pub fn is_cannon(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa.annotations.idx(sa.known.annotations.cannon).read().name;
        annotation_usages.contains(name)
    }
    pub fn is_optimize_immediately(annotation_usages: &AnnotationUsages, sa: &SemAnalysis) -> bool {
        let name = sa
            .annotations
            .idx(sa.known.annotations.optimize_immediately)
            .read()
            .name;
        annotation_usages.contains(name)
    }
}
