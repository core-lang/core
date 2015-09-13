use mem;
use parser::ast::ctxt::Context;

use parser::ast::*;
use parser::ast::Stmt::*;
use parser::ast::Expr::*;
use parser::ast::visit::*;

pub fn generate<'a, 'ast>(ctxt: &'a Context<'a, 'ast>, fct: &'ast Function) {
    CodeGenInfo::new(ctxt, fct).generate();
}

struct CodeGenInfo<'a, 'ast: 'a> {
    ctxt: &'a Context<'a, 'ast>,
    fct: &'ast Function,

    stacksize: u32,
    contains_fct_invocation: bool,
}

impl<'a, 'ast> CodeGenInfo<'a, 'ast> {
    fn new(ctxt: &'a Context<'a, 'ast>, fct: &'ast Function) -> CodeGenInfo<'a, 'ast> {
        CodeGenInfo {
            ctxt: ctxt,
            fct: fct,

            stacksize: 0,
            contains_fct_invocation: false,
        }
    }

    fn generate(&mut self) {
        self.visit_fct(self.fct);

        self.ctxt.function(self.fct.id, |fct| {
            fct.stacksize = self.stacksize;
            fct.contains_fct_invocation = self.contains_fct_invocation;
        });
    }

    fn increase_stack(&mut self, id: NodeId) {
        self.ctxt.var(id, |v| {
            let ty_size = v.data_type.size();
            self.stacksize = mem::align(self.stacksize + ty_size, ty_size);
            v.offset = -(self.stacksize as i32);
        });
    }
}

impl<'a, 'ast> Visitor<'ast> for CodeGenInfo<'a, 'ast> {
    fn visit_param(&mut self, p: &'ast Param) {
        self.increase_stack(p.id);
    }

    fn visit_stmt(&mut self, s: &'ast Stmt) {
        if let StmtVar(ref var) = *s {
            self.increase_stack(var.id);
        }

        visit::walk_stmt(self, s);
    }

    fn visit_expr(&mut self, e: &'ast Expr) {
        if let ExprCall(_) = *e {
            self.contains_fct_invocation = true;
        }

        visit::walk_expr(self, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::ast::ctxt::Context;
    use driver::cmd::Args;
    use error::msg::Msg;
    use parser::ast;
    use parser::Parser;
    use semck;

    pub fn parse<F>(code: &'static str, f: F)
            where F: for<'a, 'ast> FnOnce(&'ast Context<'a, 'ast>) -> () {
        let mut parser = Parser::from_str(code);
        let (ast, interner) = parser.parse().unwrap();
        let map = ast::map::build(&ast, &interner);
        let args : Args = Default::default();

        ast::dump::dump(&ast, &interner);

        let ctxt = Context::new(&args, &interner, &map, &ast);

        semck::check(&ctxt);
        assert!(!ctxt.diag.borrow().has_errors());

        f(&ctxt);
    }

    #[test]
    fn test_invocation_flag() {
        parse("fn f() { g(); } fn g() { }", |ctxt| {
            let fct1 = ctxt.ast.elements[0].to_function().unwrap();
            generate(ctxt, fct1);
            assert_eq!(true, ctxt.function(fct1.id, |fct| fct.contains_fct_invocation));

            let fct2 = ctxt.ast.elements[1].to_function().unwrap();
            generate(ctxt, fct2);
            assert_eq!(false, ctxt.function(fct2.id, |fct| fct.contains_fct_invocation));
        });
    }

    #[test]
    fn test_param_offset() {
        parse("fn f(a: bool, b: int) { var c = 1; }", |ctxt| {
            let fct = ctxt.ast.elements[0].to_function().unwrap();
            generate(ctxt, fct);
            assert_eq!(12, ctxt.function(fct.id, |fct| fct.stacksize));

            ctxt.function(fct.id, |fct| {
                for (varid, offset) in fct.vars.iter().zip(&[-1, -8, -12]) {
                    assert_eq!(*offset, ctxt.var_infos.borrow()[varid.0].offset);
                }
            });
        });
    }

    #[test]
    fn test_var_offset() {
        parse("fn f() { var a = true; var b = false; var c = 2; var d = \"abc\"; }", |ctxt| {
            let fct = ctxt.ast.elements[0].to_function().unwrap();
            generate(ctxt, fct);
            assert_eq!(16, ctxt.function(fct.id, |fct| fct.stacksize));

            ctxt.function(fct.id, |fct| {
                for (varid, offset) in fct.vars.iter().zip(&[-1, -2, -8, -16]) {
                    assert_eq!(*offset, ctxt.var_infos.borrow()[varid.0].offset);
                }
            });
        });
    }
}
