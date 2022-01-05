use crate::semck::{self, AllowSelf, TypeParamContext};
use crate::sym::NestedSymTable;
use crate::ty::SourceType;
use crate::vm::{ConstId, FileId, NamespaceId, SemAnalysis};

use dora_parser::ast;

pub fn check(sa: &SemAnalysis) {
    for xconst in sa.consts.iter() {
        let (const_id, file_id, ast, namespace_id) = {
            let xconst = xconst.read();
            (
                xconst.id,
                xconst.file_id,
                xconst.ast.clone(),
                xconst.namespace_id,
            )
        };

        let mut clsck = ConstCheck {
            sa,
            const_id,
            file_id,
            ast: &ast,
            namespace_id,
            symtable: NestedSymTable::new(sa, namespace_id),
        };

        clsck.check();
    }
}

struct ConstCheck<'x> {
    sa: &'x SemAnalysis,
    const_id: ConstId,
    file_id: FileId,
    ast: &'x ast::Const,
    namespace_id: NamespaceId,
    symtable: NestedSymTable<'x>,
}

impl<'x> ConstCheck<'x> {
    fn check(&mut self) {
        let ty = semck::read_type(
            self.sa,
            &self.symtable,
            self.file_id,
            &self.ast.data_type,
            TypeParamContext::None,
            AllowSelf::No,
        )
        .unwrap_or(SourceType::Error);

        let xconst = self.sa.consts.idx(self.const_id);
        let mut xconst = xconst.write();
        xconst.ty = ty;
    }
}

#[cfg(test)]
mod tests {
    use crate::error::msg::SemError;
    use crate::semck::tests::*;

    #[test]
    fn const_unknown_type() {
        err(
            "const x: Foo = 0;",
            pos(1, 10),
            SemError::UnknownIdentifier("Foo".into()),
        );

        ok("const x: Int32 = 0;");
    }
}
