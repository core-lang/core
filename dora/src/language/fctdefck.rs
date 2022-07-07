use std::collections::HashSet;

use crate::language::error::msg::SemError;
use crate::language::sem_analysis::{
    FctDefinition, FctDefinitionId, FctParent, SemAnalysis, TypeParam, TypeParamId,
};
use crate::language::sym::{NestedSymTable, Sym};
use crate::language::ty::SourceType;
use crate::language::{self, AllowSelf, TypeParamContext};

pub fn check(sa: &SemAnalysis) {
    for fct in sa.fcts.iter() {
        let mut fct = fct.write();
        let ast = fct.ast.clone();

        let mut sym_table = NestedSymTable::new(sa, fct.module_id);
        sym_table.push_level();

        match fct.parent {
            FctParent::Impl(impl_id) => {
                let impl_ = sa.impls[impl_id].read();

                for (type_param_id, param) in impl_.type_params.iter().enumerate() {
                    let sym = Sym::TypeParam(TypeParamId(type_param_id));
                    sym_table.insert(param.name, sym);
                    fct.type_params.push(param.clone());
                }

                if fct.has_self() {
                    fct.param_types.push(impl_.extended_ty.clone());
                }
            }

            FctParent::Extension(extension_id) => {
                let extension = sa.extensions[extension_id].read();

                for (type_param_id, param) in extension.type_params.iter().enumerate() {
                    let sym = Sym::TypeParam(TypeParamId(type_param_id));
                    sym_table.insert(param.name, sym);
                    fct.type_params.push(param.clone());
                }

                if fct.has_self() {
                    fct.param_types.push(extension.ty.clone());
                }
            }

            FctParent::Trait(trait_id) => {
                let trait_ = sa.traits[trait_id].read();

                for (type_param_id, param) in trait_.type_params.iter().enumerate() {
                    let sym = Sym::TypeParam(TypeParamId(type_param_id));
                    sym_table.insert(param.name, sym);
                    fct.type_params.push(param.clone());
                }

                if fct.has_self() {
                    fct.param_types.push(SourceType::This);
                }
            }

            FctParent::None => {}

            FctParent::Function(_) => unimplemented!(),
        }

        let container_type_params = fct.type_params.len();
        fct.container_type_params = container_type_params;

        if let Some(ref type_params) = ast.type_params {
            if type_params.len() > 0 {
                let mut names = HashSet::new();

                for (type_param_id, type_param) in type_params.iter().enumerate() {
                    if !names.insert(type_param.name) {
                        let name = sa.interner.str(type_param.name).to_string();
                        let msg = SemError::TypeParamNameNotUnique(name);
                        sa.diag.lock().report(fct.file_id, type_param.pos, msg);
                    }

                    fct.type_params.push(TypeParam::new(type_param.name));

                    for bound in &type_param.bounds {
                        let ty = language::read_type(
                            sa,
                            &sym_table,
                            fct.file_id,
                            bound,
                            TypeParamContext::Fct(&*fct),
                            AllowSelf::No,
                        );

                        match ty {
                            Some(SourceType::Trait(trait_id, _)) => {
                                if !fct.type_params[container_type_params + type_param_id]
                                    .trait_bounds
                                    .insert(trait_id)
                                {
                                    let msg = SemError::DuplicateTraitBound;
                                    sa.diag.lock().report(fct.file_id, type_param.pos, msg);
                                }
                            }

                            None => {
                                // unknown type, error is already thrown
                            }

                            _ => {
                                let msg = SemError::BoundExpected;
                                sa.diag.lock().report(fct.file_id, bound.pos(), msg);
                            }
                        }
                    }

                    let sym = Sym::TypeParam(TypeParamId(container_type_params + type_param_id));
                    sym_table.insert(type_param.name, sym);
                }
            } else {
                let msg = SemError::TypeParamsExpected;
                sa.diag.lock().report(fct.file_id, fct.pos, msg);
            }
        }

        for p in &ast.params {
            if fct.is_variadic {
                sa.diag
                    .lock()
                    .report(fct.file_id, p.pos, SemError::VariadicParameterNeedsToBeLast);
            }

            let ty = language::read_type(
                sa,
                &sym_table,
                fct.file_id,
                &p.data_type,
                TypeParamContext::Fct(&*fct),
                if fct.in_trait() {
                    AllowSelf::Yes
                } else {
                    AllowSelf::No
                },
            )
            .unwrap_or(SourceType::Error);

            fct.param_types.push(ty);

            if p.variadic {
                fct.is_variadic = true;
            }
        }

        if let Some(ret) = ast.return_type.as_ref() {
            let ty = language::read_type(
                sa,
                &sym_table,
                fct.file_id,
                ret,
                TypeParamContext::Fct(&*fct),
                if fct.in_trait() {
                    AllowSelf::Yes
                } else {
                    AllowSelf::No
                },
            )
            .unwrap_or(SourceType::Error);

            fct.return_type = ty;
        } else {
            fct.return_type = SourceType::Unit;
        }

        fct.initialized = true;

        check_test(sa, &*fct);

        match fct.parent {
            FctParent::Trait(traitid) => {
                let trait_ = sa.traits[traitid].read();
                check_against_methods(sa, &*fct, &trait_.methods);
            }

            FctParent::Impl(implid) => {
                let impl_ = sa.impls[implid].read();
                check_against_methods(sa, &*fct, &impl_.methods);
            }

            _ => {}
        }
    }
}

fn check_test(sa: &SemAnalysis, fct: &FctDefinition) {
    debug_assert!(fct.initialized);

    if !fct.is_test {
        return;
    }

    if !fct.parent.is_none()
        || !fct.type_params.is_empty()
        || !fct.param_types.is_empty()
        || (!fct.return_type.is_unit() && !fct.return_type.is_error())
    {
        let msg = SemError::InvalidTestAnnotationUsage;
        sa.diag.lock().report(fct.file_id, fct.pos, msg);
    }
}

fn check_against_methods(sa: &SemAnalysis, fct: &FctDefinition, methods: &[FctDefinitionId]) {
    for &method in methods {
        if method == fct.id() {
            continue;
        }

        let method = sa.fcts.idx(method);
        let method = method.read();

        if method.initialized && method.name == fct.name && method.is_static == fct.is_static {
            let method_name = sa.interner.str(method.name).to_string();

            let msg = SemError::MethodExists(method_name, method.pos);
            sa.diag.lock().report(fct.file_id, fct.ast.pos, msg);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::language::error::msg::SemError;
    use crate::language::tests::*;

    #[test]
    fn self_param() {
        err(
            "fn foo(x: Self) {}",
            pos(1, 11),
            SemError::SelfTypeUnavailable,
        );
    }

    #[test]
    fn self_return_type() {
        err(
            "fn foo(): Self {}",
            pos(1, 11),
            SemError::SelfTypeUnavailable,
        );
    }

    #[test]
    fn allow_same_method_as_static_and_non_static() {
        ok("class Foo
            impl Foo {
                @static fn foo() {}
                fn foo() {}
            }");
    }

    #[test]
    fn fct_with_type_params() {
        ok("fn f[T]() {}");
        ok("fn f[X, Y]() {}");
        err(
            "fn f[T, T]() {}",
            pos(1, 9),
            SemError::TypeParamNameNotUnique("T".into()),
        );
        err("fn f[]() {}", pos(1, 1), SemError::TypeParamsExpected);
    }

    #[test]
    fn fct_with_type_param_in_annotation() {
        ok("fn f[T](val: T) {}");
    }

    #[test]
    fn lambdas() {
        ok("fn f() { || {}; }");
        ok("fn f() { |a: Int32| {}; }");
        ok("fn f() { ||: Int32 { return 2; }; }");

        err(
            "fn f() { ||: Foo { }; }",
            pos(1, 14),
            SemError::UnknownIdentifier("Foo".into()),
        );
        err(
            "fn f() { |a: Foo| { }; }",
            pos(1, 14),
            SemError::UnknownIdentifier("Foo".into()),
        );
    }

    #[test]
    fn generic_bounds() {
        err(
            "fn f[T: Foo]() {}",
            pos(1, 9),
            SemError::UnknownIdentifier("Foo".into()),
        );
        err(
            "class Foo fn f[T: Foo]() {}",
            pos(1, 19),
            SemError::BoundExpected,
        );
        ok("trait Foo {} fn f[T: Foo]() {}");

        err(
            "trait Foo {}
            fn f[T: Foo + Foo]() {  }",
            pos(2, 18),
            SemError::DuplicateTraitBound,
        );
    }

    #[test]
    fn check_previous_defined_type_params() {
        // Type params need to be cleaned up such that the following code is an error:
        err(
            "fn f(a: T) {}",
            pos(1, 9),
            SemError::UnknownIdentifier("T".into()),
        );
    }
}
