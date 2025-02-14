use core_parser::ast::*;
use core_parser::lexer::position::Position;

pub fn returns_value(s: &Stmt) -> Result<(), Position> {
    match *s {
        Stmt::Return(_) => Ok(()),
        Stmt::For(ref stmt) => Err(stmt.pos),
        Stmt::While(ref stmt) => Err(stmt.pos),
        Stmt::Let(ref stmt) => Err(stmt.pos),
        Stmt::Expr(ref stmt) => expr_returns_value(&stmt.expr),
    }
}

pub fn expr_returns_value(e: &Expr) -> Result<(), Position> {
    match *e {
        Expr::Block(ref block) => expr_block_returns_value(block),
        Expr::If(ref expr) => expr_if_returns_value(expr),
        _ => Err(e.pos()),
    }
}

pub fn expr_block_returns_value(e: &ExprBlockType) -> Result<(), Position> {
    let mut pos = e.pos;

    for stmt in &e.stmts {
        match returns_value(stmt) {
            Ok(_) => return Ok(()),
            Err(err_pos) => pos = err_pos,
        }
    }

    if let Some(ref expr) = e.expr {
        expr_returns_value(expr)
    } else {
        Err(pos)
    }
}

fn expr_if_returns_value(e: &ExprIfType) -> Result<(), Position> {
    for case in &e.cases {
        expr_returns_value(&case.value)?;
    }

    match e.else_block {
        Some(ref block) => expr_returns_value(block),
        None => Err(e.pos),
    }
}

#[cfg(test)]
mod tests {
    use crate::language::error::msg::ErrorMessage;
    use crate::language::tests::*;

    #[test]
    fn returns_unit() {
        ok("fun f(): Unit {}");
        ok("fun f(): Unit { if true { return; } }");
        ok("fun f(): Unit { while true { return; } }");
    }

    #[test]
    fn returns_int() {
        err(
            "fun f(): Int32 { }",
            pos(1, 16),
            ErrorMessage::ReturnType("Int32".into(), "()".into()),
        );
        err(
            "fun f(): Int32 { if true { return 1; } }",
            pos(1, 16),
            ErrorMessage::ReturnType("Int32".into(), "()".into()),
        );
        err(
            "fun f(): Int32 { if true { } else { return 1; } }",
            pos(1, 16),
            ErrorMessage::ReturnType("Int32".into(), "()".into()),
        );
        err(
            "fun f(): Int32 { while true { return 1; } }",
            pos(1, 16),
            ErrorMessage::ReturnType("Int32".into(), "()".into()),
        );
        ok("fun f(): Int32 { if true { return 1; } else { return 2; } }");
        ok("fun f(): Int32 { return 1; 1+2; }");
    }
}
