use std::sync::Arc;

use parking_lot::RwLock;

use core_parser::ast;

use crate::language::error::msg::ErrorMessage;
use crate::language::sem_analysis::{
    EnumDefinition, EnumVariant, SemAnalysis, SourceFileId, UnionDefinition, UnionVariant,
};
use crate::language::sym::{ModuleSymTable, Sym};
use crate::language::ty::SourceType;
use crate::language::{read_type, uniondefck, AllowSelf, TypeParamContext};

pub fn check(sa: &SemAnalysis) {
    for union_ in sa.unions.iter() {
        let ast = union_.read().ast.clone();

        let mut enumck = UnionCheck {
            sa,
            file_id: union_.read().file_id,
            ast: &ast,
            union_: &union_,
        };

        enumck.check();
    }
}

struct UnionCheck<'x> {
    sa: &'x SemAnalysis,
    file_id: SourceFileId,
    ast: &'x Arc<ast::Union>,
    union_: &'x RwLock<UnionDefinition>,
}

impl<'x> UnionCheck<'x> {
    fn check(&mut self) {
        let mut symtable = ModuleSymTable::new(self.sa, self.union_.read().module_id);

        symtable.push_level();

        {
            let union_ = self.union_.read();

            for (id, name) in union_.type_params().names() {
                symtable.insert(name, Sym::TypeParam(id));
            }
        }

        check_variants(self.sa);

        self.union_.write().simple_enumeration = false;

        symtable.pop_level();
    }
}

pub fn check_variants(sa: &SemAnalysis) {
    for union_ in sa.unions.iter() {
        let union_variants;
        {
            let mut union_ = union_.read();
            let ast = union_.ast.clone();

            let mut unionck = UnionCheckVariants {
                sa,
                file_id: union_.file_id,
                ast: &ast,
                union_: &*union_,
            };
            union_variants = unionck.check();
        }

        union_.write().variants = union_variants;
    }
}

struct UnionCheckVariants<'x> {
    sa: &'x SemAnalysis,
    file_id: SourceFileId,
    ast: &'x Arc<ast::Union>,
    union_: &'x UnionDefinition,
}

impl<'x> UnionCheckVariants<'x> {
    fn check(&mut self) -> Vec<UnionVariant> {
        let mut next_variant_id: u32 = 0;

        let mut symtable = ModuleSymTable::new(self.sa, self.union_.module_id);

        symtable.push_level();

        let mut union_variants = Vec::new();

        for variant in &self.ast.variants {
            let variant_ty = read_type(
                self.sa,
                &symtable,
                self.file_id.into(),
                &variant.type_,
                TypeParamContext::Union(self.union_.id()),
                AllowSelf::No,
            )
            .unwrap_or(SourceType::Error);
            println!(
                "{} is_class={}",
                variant.type_.to_string(&self.sa.interner),
                variant_ty.is_cls()
            );
            let variant = UnionVariant {
                id: next_variant_id as usize,
                type_: variant_ty,
            };

            union_variants.push(variant);

            next_variant_id += 1;
        }

        union_variants
    }
}

#[cfg(test)]
mod tests {
    use crate::language::error::msg::ErrorMessage;
    use crate::language::tests::*;

    #[test]
    fn enum_definitions() {
        err("enum Foo {}", pos(1, 1), ErrorMessage::NoEnumVariant);
        ok("enum Foo { A, B, C }");
        err(
            "enum Foo { A, A }",
            pos(1, 15),
            ErrorMessage::ShadowEnumVariant("A".into()),
        );
    }

    #[test]
    fn enum_with_argument() {
        ok("
            enum Foo { A(Int32), B(Float32), C}
            fun give_me_a(): Foo { Foo::A(1i32) }
            fun give_me_b(): Foo { Foo::B(2.0f32) }
            fun give_me_c(): Foo { Foo::C }

        ");
    }

    #[test]
    fn enum_wrong_type() {
        err(
            "
            enum Foo { A(Int32), B(Float32), C}
            fun give_me_a(): Foo { Foo::A(2.0f32) }

        ",
            pos(3, 42),
            ErrorMessage::EnumArgsIncompatible(
                "Foo".into(),
                "A".into(),
                vec!["Int32".into()],
                vec!["Float32".into()],
            ),
        );
    }

    #[test]
    fn enum_missing_args() {
        err(
            "
            enum Foo { A(Int32), B(Float32), C}
            fun give_me_a(): Foo { Foo::A }

        ",
            pos(3, 39),
            ErrorMessage::EnumArgsIncompatible(
                "Foo".into(),
                "A".into(),
                vec!["Int32".into()],
                Vec::new(),
            ),
        );
    }

    #[test]
    fn enum_unexpected_args() {
        err(
            "
            enum Foo { A(Int32), B(Float32), C}
            fun give_me_c(): Foo { Foo::C(12.0f32) }

        ",
            pos(3, 42),
            ErrorMessage::EnumArgsIncompatible(
                "Foo".into(),
                "C".into(),
                Vec::new(),
                vec!["Float32".into()],
            ),
        );
    }

    #[test]
    fn enum_parens_but_no_args() {
        err(
            "
            enum Foo { A(Int32), B(Float32), C}
            fun give_me_c(): Foo { Foo::C() }
        ",
            pos(3, 42),
            ErrorMessage::EnumArgsNoParens("Foo".into(), "C".into()),
        );
    }

    #[test]
    fn enum_copy() {
        ok("
            enum Foo { A(Int32), B(Float32), C}
            fun foo_test(y: Foo): Foo { let x: Foo = y; x }
        ");
    }

    #[test]
    fn enum_generic() {
        ok("
            enum Foo[T] { One(T), Two }
        ");
    }

    #[test]
    fn enum_with_type_param() {
        ok("trait SomeTrait {} enum MyOption[T: SomeTrait] { None, Some(T) }");
    }

    #[test]
    fn enum_generic_with_failures() {
        err(
            "enum MyOption[] { A, B }",
            pos(1, 1),
            ErrorMessage::TypeParamsExpected,
        );

        err(
            "enum MyOption[X, X] { A, B }",
            pos(1, 18),
            ErrorMessage::TypeParamNameNotUnique("X".into()),
        );

        err(
            "enum MyOption[X: NonExistingTrait] { A, B }",
            pos(1, 18),
            ErrorMessage::UnknownIdentifier("NonExistingTrait".into()),
        );
    }

    #[test]
    fn check_enum_type() {
        err(
            "
                enum MyOption[X] { A, B }
                fun foo(v: MyOption): Unit {}
            ",
            pos(3, 28),
            ErrorMessage::WrongNumberTypeParams(1, 0),
        );
    }

    #[test]
    fn check_enum_value() {
        ok("
            enum Foo { A(Int32), B }
            fun foo(): Foo { Foo::A(1i32) }
            fun bar(): Foo { Foo::B }
        ");

        err(
            "
            enum Foo { A(Int32), B }
            fun foo(): Foo { Foo::A(true) }
        ",
            pos(3, 36),
            ErrorMessage::EnumArgsIncompatible(
                "Foo".into(),
                "A".into(),
                vec!["Int32".into()],
                vec!["Bool".into()],
            ),
        );
    }

    #[test]
    fn check_enum_value_generic() {
        ok("
            enum Foo[T] { A, B }
            fun foo(): Unit { let tmp = Foo[String]::B; }
        ");

        err(
            "
            trait SomeTrait {}
            enum Foo[T: SomeTrait] { A, B }
            fun foo(): Unit { let tmp = Foo[String]::B; }
        ",
            pos(4, 52),
            ErrorMessage::TypeNotImplementingTrait("String".into(), "SomeTrait".into()),
        );
    }

    #[test]
    fn enum_with_generic_argument() {
        ok("
            enum Foo[T] { A(T), B }
            fun foo(): Unit { let tmp = Foo[Int32]::A(0i32); }
        ");

        err(
            "
            enum Foo[T] { A(T), B }
            fun foo(): Unit { let tmp = Foo[Int32]::A(true); }
        ",
            pos(3, 54),
            ErrorMessage::EnumArgsIncompatible(
                "Foo".into(),
                "A".into(),
                vec!["T".into()],
                vec!["Bool".into()],
            ),
        );
    }

    #[test]
    fn enum_move_generic() {
        ok("
            enum Foo[T] { A(T), B }
            fun foo(x: Foo[Int32]): Foo[Int32] { x }
        ");

        err(
            "
            enum Foo[T] { A(T), B }
            fun foo(x: Foo[Int32]): Foo[Float32] { x }
        ",
            pos(3, 50),
            ErrorMessage::ReturnType("Foo[Float32]".into(), "Foo[Int32]".into()),
        );
    }

    #[test]
    fn enum_nested() {
        ok("
            enum Foo { A(Foo), B }
        ");
    }
}
