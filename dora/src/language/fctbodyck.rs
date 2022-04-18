use crate::language::fctbodyck::body::TypeCheck;
use crate::language::fctbodyck::constck::ConstCheck;
use crate::language::sem_analysis::{AnalysisData, SemAnalysis};
use crate::language::sym::NestedSymTable;

use dora_parser::ast::NodeId;

pub mod body;
mod constck;
mod lookup;
#[cfg(test)]
mod tests;

pub fn check(sa: &SemAnalysis) {
    for fct in sa.fcts.iter() {
        let analysis = {
            let fct = fct.read();

            if !fct.has_body() {
                continue;
            }

            if fct.is_lambda() {
                // Lambdas will be type-checked by their parent.
                continue;
            }

            let mut analysis = AnalysisData::new();
            let symtable = NestedSymTable::new(sa, fct.module_id);

            let mut typeck = TypeCheck {
                sa,
                fct: &fct,
                file_id: fct.file_id,
                module_id: fct.module_id,
                analysis: &mut analysis,
                ast: &fct.ast,
                symtable: symtable,
                in_loop: false,
                self_ty: None,
            };

            typeck.check();

            analysis
        };

        fct.write().analysis = Some(analysis);
    }

    for const_ in sa.consts.iter() {
        let mut const_ = const_.write();

        let (_, value) = {
            let mut constck = ConstCheck {
                sa,
                const_: &*const_,
                negative_expr_id: NodeId(0),
            };

            constck.check_expr(&const_.expr)
        };

        const_.value = value;
    }
}
