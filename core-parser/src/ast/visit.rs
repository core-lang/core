use crate::ast::*;

pub trait Visitor: Sized {
    fn visit_file(&mut self, a: &File) {
        walk_file(self, a);
    }

    fn visit_global(&mut self, g: &Arc<Global>) {
        walk_global(self, g);
    }

    fn visit_trait(&mut self, t: &Arc<Trait>) {
        walk_trait(self, t);
    }

    fn visit_impl(&mut self, i: &Arc<Impl>) {
        walk_impl(self, i);
    }

    fn visit_class(&mut self, c: &Arc<Class>) {
        walk_class(self, c);
    }

    fn visit_value(&mut self, s: &Arc<Value>) {
        walk_value(self, s);
    }

    fn visit_union(&mut self, u: &Arc<Union>) {
        walk_union(self, u);
    }

    fn visit_annotation(&mut self, a: &Arc<Annotation>) {
        walk_annotation(self, a);
    }

    fn visit_const(&mut self, c: &Arc<Const>) {
        walk_const(self, c);
    }

    fn visit_enum(&mut self, e: &Arc<Enum>) {
        walk_enum(self, e);
    }

    fn visit_alias(&mut self, e: &Arc<Alias>) {
        walk_alias(self, e);
    }

    fn visit_module(&mut self, e: &Arc<Module>) {
        walk_module(self, e);
    }

    fn visit_use(&mut self, i: &Arc<Use>) {
        walk_use(self, i);
    }

    fn visit_value_field(&mut self, f: &ValueField) {
        walk_value_field(self, f);
    }

    fn visit_ctor(&mut self, m: &Arc<Function>) {
        walk_fct(self, &m);
    }

    fn visit_method(&mut self, m: &Arc<Function>) {
        walk_fct(self, &m);
    }

    fn visit_field(&mut self, p: &Field) {
        walk_field(self, p);
    }

    fn visit_fct(&mut self, f: &Arc<Function>) {
        walk_fct(self, &f);
    }

    fn visit_param(&mut self, p: &Param) {
        walk_param(self, p);
    }

    fn visit_type(&mut self, t: &Type) {
        walk_type(self, t);
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        walk_stmt(self, s);
    }

    fn visit_expr(&mut self, e: &Expr) {
        walk_expr(self, e);
    }
}

pub fn walk_file<V: Visitor>(v: &mut V, f: &File) {
    for e in &f.elements {
        walk_elem(v, e);
    }
}

pub fn walk_elem<V: Visitor>(v: &mut V, e: &Elem) {
    match e {
        Elem::Function(f) => v.visit_fct(f),
        Elem::Class(ref c) => v.visit_class(c),
        Elem::Value(ref s) => v.visit_value(s),
        Elem::Union(ref u) => v.visit_union(u),
        Elem::Trait(ref t) => v.visit_trait(t),
        Elem::Impl(ref i) => v.visit_impl(i),
        Elem::Annotation(ref a) => v.visit_annotation(a),
        Elem::Global(ref g) => v.visit_global(g),
        Elem::Const(ref c) => v.visit_const(c),
        Elem::Enum(ref e) => v.visit_enum(e),
        Elem::Alias(ref e) => v.visit_alias(e),
        Elem::Module(ref e) => v.visit_module(e),
        Elem::Use(ref i) => v.visit_use(i),
    }
}

pub fn walk_global<V: Visitor>(v: &mut V, g: &Global) {
    v.visit_type(&g.data_type);

    if let Some(ref initializer) = g.initializer {
        v.visit_fct(initializer);
    }
}

pub fn walk_trait<V: Visitor>(v: &mut V, t: &Arc<Trait>) {
    for m in &t.methods {
        v.visit_method(m);
    }
}

pub fn walk_impl<V: Visitor>(v: &mut V, i: &Arc<Impl>) {
    for m in &i.methods {
        v.visit_method(m);
    }
}

pub fn walk_class<V: Visitor>(v: &mut V, c: &Arc<Class>) {
    for f in &c.fields {
        v.visit_field(f);
    }
}

pub fn walk_annotation<V: Visitor>(_v: &mut V, _a: &Arc<Annotation>) {}

pub fn walk_const<V: Visitor>(v: &mut V, c: &Arc<Const>) {
    v.visit_type(&c.data_type);
    v.visit_expr(&c.expr);
}

pub fn walk_enum<V: Visitor>(_v: &mut V, _e: &Arc<Enum>) {
    // nothing to do
}

pub fn walk_alias<V: Visitor>(v: &mut V, a: &Arc<Alias>) {
    v.visit_type(&a.ty);
}

pub fn walk_module<V: Visitor>(v: &mut V, node: &Arc<Module>) {
    if let Some(ref elements) = node.elements {
        for e in elements {
            walk_elem(v, e);
        }
    }
}

pub fn walk_use<V: Visitor>(_v: &mut V, _use: &Arc<Use>) {
    // nothing to do
}

pub fn walk_value<V: Visitor>(v: &mut V, s: &Value) {
    for f in &s.fields {
        v.visit_value_field(f);
    }
}

pub fn walk_value_field<V: Visitor>(v: &mut V, f: &ValueField) {
    v.visit_type(&f.data_type);
}

pub fn walk_union<V: Visitor>(_v: &mut V, _u: &Union) {
    // nothing to do
}

pub fn walk_field<V: Visitor>(v: &mut V, f: &Field) {
    v.visit_type(&f.data_type);
}

pub fn walk_fct<V: Visitor>(v: &mut V, f: &Function) {
    for p in &f.params {
        v.visit_param(p);
    }

    if let Some(ref ty) = f.return_type {
        v.visit_type(ty);
    }

    if let Some(ref block) = f.block {
        for stmt in &block.stmts {
            v.visit_stmt(stmt);
        }

        if let Some(ref value) = block.expr {
            v.visit_expr(value);
        }
    }
}

pub fn walk_param<V: Visitor>(v: &mut V, p: &Param) {
    v.visit_type(&p.data_type);
}

pub fn walk_type<V: Visitor>(v: &mut V, t: &Type) {
    match *t {
        Type::This(_) => {}
        Type::Basic(_) => {}
        Type::Tuple(ref tuple) => {
            for ty in &tuple.subtypes {
                v.visit_type(ty);
            }
        }

        Type::Lambda(ref fct) => {
            for ty in &fct.params {
                v.visit_type(ty);
            }

            v.visit_type(&fct.ret);
        }
    }
}

pub fn walk_stmt<V: Visitor>(v: &mut V, s: &Stmt) {
    match *s {
        Stmt::Let(ref value) => {
            if let Some(ref ty) = value.data_type {
                v.visit_type(ty);
            }

            if let Some(ref e) = value.expr {
                v.visit_expr(e);
            }
        }

        Stmt::For(ref value) => {
            v.visit_expr(&value.expr);
            v.visit_stmt(&value.block);
        }

        Stmt::While(ref value) => {
            v.visit_expr(&value.cond);
            v.visit_stmt(&value.block);
        }

        Stmt::Expr(ref value) => {
            v.visit_expr(&value.expr);
        }

        Stmt::Return(ref value) => {
            if let Some(ref e) = value.expr {
                v.visit_expr(e);
            }
        }
    }
}

pub fn walk_expr<V: Visitor>(v: &mut V, e: &Expr) {
    match *e {
        Expr::Un(ref value) => {
            v.visit_expr(&value.opnd);
        }

        Expr::Bin(ref value) => {
            v.visit_expr(&value.lhs);
            v.visit_expr(&value.rhs);
        }

        Expr::Call(ref call) => {
            v.visit_expr(&call.callee);

            for arg in &call.args {
                v.visit_expr(&arg.expr);
            }
        }

        Expr::TypeParam(ref expr) => {
            v.visit_expr(&expr.callee);

            for arg in &expr.args {
                v.visit_type(arg);
            }
        }

        Expr::Path(ref path) => {
            v.visit_expr(&path.lhs);
            v.visit_expr(&path.rhs);
        }

        Expr::Dot(ref value) => {
            v.visit_expr(&value.lhs);
            v.visit_expr(&value.rhs);
        }

        Expr::Conv(ref value) => {
            v.visit_expr(&value.object);
            v.visit_type(&value.data_type);
        }

        Expr::Lambda(ref fct) => v.visit_fct(fct),

        Expr::Block(ref value) => {
            for stmt in &value.stmts {
                v.visit_stmt(stmt);
            }

            if let Some(ref expr) = value.expr {
                v.visit_expr(expr);
            }
        }

        Expr::Template(ref value) => {
            for part in &value.parts {
                v.visit_expr(part);
            }
        }

        Expr::Tuple(ref value) => {
            for expr in &value.values {
                v.visit_expr(expr);
            }
        }

        Expr::Paren(ref value) => {
            v.visit_expr(&value.expr);
        }

        Expr::If(ref value) => {
            v.visit_stmt(&Stmt::Let(value.cond.as_ref().clone()));
            for case in &value.cases {
                match &case.data {
                    IfCaseData::Simple => {}
                    IfCaseData::Continuation(expr) => v.visit_expr(expr),
                    IfCaseData::Patterns(_) => {}
                }
                v.visit_expr(&case.value);
            }

            if let Some(ref b) = value.else_block {
                v.visit_expr(b);
            }
        }

        Expr::This(_) => {}
        Expr::LitChar(_) => {}
        Expr::LitInt(_) => {}
        Expr::LitFloat(_) => {}
        Expr::LitStr(_) => {}
        Expr::LitBool(_) => {}
        Expr::Ident(_) => {}
    }
}
