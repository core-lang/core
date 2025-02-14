use crate::language::error::msg::ErrorMessage;
use crate::language::sem_analysis::ConstValue;
use crate::language::tests::*;

#[test]
fn type_method_size() {
    ok("fun f(a: String): Int64 = a.size");
    ok("fun f(a: String): Int64 = \"abc\".size");
}

#[test]
fn type_object_field() {
    ok("class Foo(a:Int32) fun f(x: Foo): Int32 = x.a");
    ok("class Foo(a:String) fun f(x: Foo): String = x.a");
    err(
        "class Foo(a:Int32) fun f(x: Foo): Bool = x.a",
        pos(1, 43),
        ErrorMessage::ReturnType("Bool".into(), "Int32".into()),
    );
    err(
        "class Foo(a:Int32) fun f(x: Foo): Int32 = x.b",
        pos(1, 44),
        ErrorMessage::UnknownField("b".into(), "Foo".into()),
    );
}

#[test]
fn type_object_set_field() {
    ok("class Foo(a: Int32) fun f(x: Foo): Unit { x.a = 1 }");
    err(
        "class Foo(a: Int32) fun f(x: Foo): Unit { x.a = false }",
        pos(1, 47),
        ErrorMessage::AssignField("a".into(), "Foo".into(), "Int32".into(), "Bool".into()),
    );
}

#[test]
fn type_object_field_without_self() {
    err(
        "class Foo(a: Int32) impl Foo { fun f(): Int32 = a }",
        pos(1, 49),
        ErrorMessage::UnknownIdentifier("a".into()),
    );
    err(
        "class Foo(a: Int32) impl Foo { fun set(x: Int32): Unit { a = x } }",
        pos(1, 58),
        ErrorMessage::UnknownIdentifier("a".into()),
    );
}

#[test]
fn type_class_method_call() {
    ok("
        class Foo
        impl Foo {
            fun bar(): Unit {}
            fun baz(): Int32 = 1
        }

        fun f(x: Foo): Unit = x.bar()
        fun g(x: Foo): Int32 = x.baz()");

    err(
        "
        class Foo
        impl Foo {
            fun bar(): Int32 = 0
        }

        fun f(x: Foo): String = x.bar()",
        pos(7, 38),
        ErrorMessage::ReturnType("String".into(), "Int32".into()),
    );
}

#[test]
fn return_type() {
    err(
        "
        class Foo[T]
        fun f(): Foo[Int32] { Foo[Int64]() }
    ",
        pos(3, 29),
        ErrorMessage::ReturnType("Foo[Int32]".into(), "Foo[Int64]".into()),
    );
}

#[test]
fn type_method_defined_twice() {
    err(
        "class Foo
        impl Foo {
                 fun bar(): Unit {}
                 fun bar(): Unit {}
             }",
        pos(4, 18),
        ErrorMessage::MethodExists("bar".into(), pos(3, 18)),
    );

    err(
        "class Foo
        impl Foo{
                 fun bar(): Unit {}
                 fun bar(): Int32 {}
             }",
        pos(4, 18),
        ErrorMessage::MethodExists("bar".into(), pos(3, 18)),
    );

    err(
        "class Foo
        impl Foo {
                 fun bar(a: Int32): Unit {}
                 fun bar(a: Int32): Int32 {}
             }",
        pos(4, 18),
        ErrorMessage::MethodExists("bar".into(), pos(3, 18)),
    );

    err(
        "class Foo
        impl Foo {
                fun bar(a: Int32): Unit {}
                fun bar(a: String): Unit {}
            }",
        pos(4, 17),
        ErrorMessage::MethodExists("bar".into(), pos(3, 17)),
    );
}

#[test]
fn type_self() {
    ok("class Foo impl Foo { fun me(): Foo = self }");
    err(
        "class Foo fun me(): Unit = self",
        pos(1, 28),
        ErrorMessage::ThisUnavailable,
    );

    ok("class Foo(a: Int32, b: Int32)
        impl Foo {
            fun bar(): Int32 = self.a + self.b
        }");

    ok("class Foo(a: Int32)
        impl Foo {
            fun setA(a: Int32): Unit { self.a = a }
        }");

    ok("class Foo
        impl Foo {
            fun zero(): Int32 = 0i32
            fun other(): Int32 = self.zero()
        }");

    ok("class Foo
        impl Foo {
            fun bar(): Unit = self.bar()
        }");
}

#[test]
fn type_unknown_method() {
    err(
        "class Foo
            impl Foo {
                 fun bar(a: Int32): Unit { }
            }

            fun f(x: Foo): Unit = x.bar()",
        pos(6, 40),
        ErrorMessage::ParamTypesIncompatible("bar".into(), vec!["Int32".into()], Vec::new()),
    );

    err(
        "class Foo
              fun f(x: Foo): Unit = x.bar(1i32)",
        pos(2, 42),
        ErrorMessage::UnknownMethod("Foo".into(), "bar".into(), vec!["Int32".into()]),
    );
}

#[test]
fn type_ctor() {
    ok("class Foo fun f(): Foo = Foo()");
    ok("class Foo(a: Int32) fun f(): Foo = Foo(1i32)");
    err(
        "class Foo fun f(): Foo = 1i32",
        pos(1, 26),
        ErrorMessage::ReturnType("Foo".into(), "Int32".into()),
    );
}

#[test]
fn type_def_for_return_type() {
    ok("fun a(): Int32 { return 1i32 }");
    err(
        "fun a(): unknown {}",
        pos(1, 10),
        ErrorMessage::UnknownIdentifier("unknown".into()),
    );
}

#[test]
fn type_def_for_param() {
    ok("fun a(b: Int32): Unit {}");
    err(
        "fun a(b: foo): Unit {}",
        pos(1, 10),
        ErrorMessage::UnknownIdentifier("foo".into()),
    );
}

#[test]
fn type_def_for_var() {
    ok("fun a(): Unit { let a: Int32 = 1i32 }");
    err(
        "fun a(): Unit { let a: test = 1 }",
        pos(1, 24),
        ErrorMessage::UnknownIdentifier("test".into()),
    );
}

#[test]
fn type_var_wrong_type_defined() {
    ok("fun f(): Unit { let a: Int32 = 1i32 }");
    ok("fun f(): Unit { let a: Bool = false }");
    ok("fun f(): Unit { let a: String = \"f\" }");

    err(
        "fun f(): Unit { let a: Int32 = true }",
        pos(1, 17),
        ErrorMessage::AssignType("a".into(), "Int32".into(), "Bool".into()),
    );
    err(
        "fun f(): Unit { let b: Bool = 2i32 }",
        pos(1, 17),
        ErrorMessage::AssignType("b".into(), "Bool".into(), "Int32".into()),
    );
}

#[test]
fn type_while() {
    ok("fun x(): Unit { while true { } }");
    ok("fun x(): Unit { while false { } }");
    err(
        "fun x(): Unit { while 2i32 { } }",
        pos(1, 17),
        ErrorMessage::WhileCondType("Int32".into()),
    );
}

#[test]
fn type_if() {
    ok("fun x(): Unit { if true { } }");
    ok("fun x(): Unit { if false { } }");
    err(
        "fun x(): Unit { if 4i32 { } }",
        pos(1, 20),
        ErrorMessage::IfCondType("Int32".into()),
    );
}

#[test]
fn type_return_unit() {
    ok("fun f(): Unit { return }");
    ok("fun f(): Unit { return; }");
    err(
        "fun f(): Unit { return 1i32 }",
        pos(1, 17),
        ErrorMessage::ReturnType("()".into(), "Int32".into()),
    );
}

#[test]
fn type_return() {
    ok("fun f(): Int32 { let a = 1i32; return a }");
    ok("fun f(): Int32 { return 1i32 }");
    err(
        "fun f(): Int32 { return }",
        pos(1, 18),
        ErrorMessage::ReturnType("Int32".into(), "()".into()),
    );

    ok("fun f(): Int32 { return 0i32 }
            fun g(): Int32 { return f() }");
    err(
        "fun f(): Unit { }
             fun g(): Int32 { return f() }",
        pos(2, 31),
        ErrorMessage::ReturnType("Int32".into(), "()".into()),
    );
}

#[test]
fn type_variable() {
    ok("fun f(a: Int32): Unit { let b: Int32 = a }");
}

#[test]
fn type_let() {
    ok("fun f(value: (Int32, Int32)): Int32 { let (a, b) = value; a+b }");
    err(
        "fun f(): Unit { let (a, b) = true }",
        pos(1, 21),
        ErrorMessage::LetPatternExpectedTuple("Bool".into()),
    );

    ok("fun f(value: ()): Unit { let () = value }");
    err(
        "fun f(): Unit { let () = true }",
        pos(1, 21),
        ErrorMessage::LetPatternExpectedTuple("Bool".into()),
    );
    err(
        "fun f(): Unit { let (a, b) = () }",
        pos(1, 21),
        ErrorMessage::LetPatternShouldBeUnit,
    );

    err(
        "fun f(): Unit { let (a, b) = (true,) }",
        pos(1, 21),
        ErrorMessage::LetPatternExpectedTupleWithLength("(Bool)".into(), 1, 2),
    );
    err(
        "fun f(): Unit { let () = (true,) }",
        pos(1, 21),
        ErrorMessage::LetPatternExpectedTupleWithLength("(Bool)".into(), 1, 0),
    );

    ok("fun f(val: (Int32, (Int32, Int32))): Int32 { let (a, (b, c)) = val; a+b+c }");
}

#[test]
fn type_assign_lvalue() {
    err(
        "fun f(): Unit { 1 = 3 }",
        pos(1, 19),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn type_un_op() {
    ok("fun f(a: Int32): Unit { -a; +a; () }");
    err(
        "fun f(a: Bool): Unit { -a; () }",
        pos(1, 24),
        ErrorMessage::UnOpType("-".into(), "Bool".into()),
    );
    err(
        "fun f(a: Bool): Unit { +a; () }",
        pos(1, 24),
        ErrorMessage::UnOpType("+".into(), "Bool".into()),
    );
}

#[test]
fn type_bin_op() {
    ok("fun f(a: Int32): Unit { a+a; a-a; a*a; a/a; () }");
    ok("fun f(a: Int32): Unit { a<a; a<=a; a==a; a!=a; a>a; a>=a; () }");
    ok("fun f(a: String): Unit { a<a; a<=a; a==a; a!=a; a>a; a>=a; () }");
    ok("fun f(a: String): Unit { a===a; a!==a; a+a; () }");
    ok("class Foo fun f(a: Foo): Unit { a===a; a!==a; () }");
    ok("fun f(a: Int32): Unit { a|a; a&a; a^a; () }");
    ok("fun f(a: Bool): Unit { a||a; a&&a; () }");

    err(
        "class A class B fun f(a: A, b: B): Unit { a === b; () }",
        pos(1, 45),
        ErrorMessage::TypesIncompatible("A".into(), "B".into()),
    );
    err(
        "class A class B fun f(a: A, b: B): Unit { b !== a; () }",
        pos(1, 45),
        ErrorMessage::TypesIncompatible("B".into(), "A".into()),
    );
    err(
        "fun f(a: Bool): Unit { a+a; () }",
        pos(1, 25),
        ErrorMessage::BinOpType("+".into(), "Bool".into(), "Bool".into()),
    );
    err(
        "fun f(a: Bool): Unit { a^a; () }",
        pos(1, 25),
        ErrorMessage::BinOpType("^".into(), "Bool".into(), "Bool".into()),
    );
    err(
        "fun f(a: Int32): Unit { a||a; () }",
        pos(1, 26),
        ErrorMessage::BinOpType("||".into(), "Int32".into(), "Int32".into()),
    );
    err(
        "fun f(a: Int32): Unit { a&&a; () }",
        pos(1, 26),
        ErrorMessage::BinOpType("&&".into(), "Int32".into(), "Int32".into()),
    );
    err(
        "fun f(a: String): Unit { a-a; () }",
        pos(1, 27),
        ErrorMessage::BinOpType("-".into(), "String".into(), "String".into()),
    );
    err(
        "fun f(a: String): Unit { a*a; () }",
        pos(1, 27),
        ErrorMessage::BinOpType("*".into(), "String".into(), "String".into()),
    );
}

#[test]
fn type_function_return_type() {
    ok("fun foo(): Int32 = 1i32; fun f(): Unit { let i: Int32 = foo() }");
    err(
        "
        fun foo(): Int32 = 1i32
        fun f(): Unit { let i: Bool = foo() }",
        pos(3, 25),
        ErrorMessage::AssignType("i".into(), "Bool".into(), "Int32".into()),
    );
}

#[test]
fn type_ident_in_function_params() {
    ok("
    fun f(a: Int32): Unit {}
    fun g(): Unit { let a = 1i32; f(a) }");
}

#[test]
fn type_recursive_function_call() {
    ok("fun f(a: Int32): Unit = f(a)");
}

#[test]
fn type_function_params() {
    ok("fun foo(): Unit {} fun f(): Unit = foo()");
    ok("fun foo(a: Int32): Unit {} fun f(): Unit = foo(1i32)");
    ok("fun foo(a: Int32, b: Bool): Unit {} fun f(): Unit = foo(1i32, true)");

    err(
        "
        fun foo(): Unit {}
        fun f(): Unit = foo(1i32)",
        pos(3, 28),
        ErrorMessage::ParamTypesIncompatible("foo".into(), vec![], vec!["Int32".into()]),
    );
    err(
        "
        fun foo(a: Int32): Unit {}
        fun f(): Unit = foo(true)",
        pos(3, 28),
        ErrorMessage::ParamTypesIncompatible(
            "foo".into(),
            vec!["Int32".into()],
            vec!["Bool".into()],
        ),
    );
    err(
        "
        fun foo(a: Int32, b: Bool): Unit {}
        fun f(): Unit = foo(1i32, 2i32)",
        pos(3, 28),
        ErrorMessage::ParamTypesIncompatible(
            "foo".into(),
            vec!["Int32".into(), "Bool".into()],
            vec!["Int32".into(), "Int32".into()],
        ),
    );
}

#[test]
fn type_array() {
    ok("fun f(a: Array[Int32]): Int32 = a(1i64)");
    err(
        "fun f(a: Array[Int32]): String = a(1i64)",
        pos(1, 35),
        ErrorMessage::ReturnType("String".into(), "Int32".into()),
    );
}

#[test]
fn type_array_assign() {
    err(
        "fun f(a: Array[Int32]): Int32 { return a(3) = 4i32 }",
        pos(1, 33),
        ErrorMessage::ReturnType("Int32".into(), "()".into()),
    );
    err(
        "fun f(a: Array[Int32]): Unit { a(3) = \"b\" }",
        pos(1, 37),
        ErrorMessage::UnknownMethod(
            "Array[Int32]".into(),
            "set".into(),
            vec!["Int64".into(), "String".into()],
        ),
    );
}

#[test]
fn type_array_field() {
    ok("
        class Foo(x: Array[Int32])
        fun f(a: Foo): Int32 = a.x(1i64)
    ");
}

#[test]
fn wrong_type_params_for_primitive() {
    err(
        "
        fun f(): Unit { let a: Int32[Bool, Char] = 10 }
    ",
        pos(2, 32),
        ErrorMessage::WrongNumberTypeParams(0, 2),
    );
}

#[test]
fn let_without_initialization() {
    err(
        "fun f(): Unit { let x: Int32 }",
        pos(1, 17),
        ErrorMessage::LetMissingInitialization,
    );
}

#[test]
fn reassign_param() {
    err(
        "fun f(a: Int32): Unit { a = 1 }",
        pos(1, 27),
        ErrorMessage::LetReassigned,
    );
}

#[test]
fn reassign_field() {
    ok("class Foo(x: Int32) fun foo(f: Foo): Unit { f.x = 1 }");
}

#[test]
fn reassign_var() {
    ok("fun f(): Unit { var a=1; a=2 }");
}

#[test]
fn reassign_let() {
    err(
        "fun f(): Unit { let a=1; a=2 }",
        pos(1, 27),
        ErrorMessage::LetReassigned,
    );
}

#[test]
fn reassign_self() {
    err(
        "class Foo
        impl Foo { fun f(): Unit { self = Foo() } }",
        pos(2, 41),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn same_names() {
    ok("class Foo { Foo: Foo }");
    ok("class Foo fun foo(): Unit { let Foo: Int32 = 1i32 }");
}

#[test]
fn lit_int64() {
    ok("fun f(): Int64 = 1i64");
    ok("fun f(): Int32 = 1i32");

    let ret = ErrorMessage::ReturnType("Int32".into(), "Int64".into());
    err("fun f(): Int32 = 1i64", pos(1, 18), ret);

    ok("fun f(): Int64 = 1");
}

#[test]
fn lit_int64_as_default() {
    ok("fun f(): Int64 = 1");
    ok("fun f(): Int64 {
        let x = 1
        x
    }");
}

#[test]
fn overload_plus() {
    ok("class A impl A { fun plus(rhs: A): Int32 = 0 }
            fun f(): Int32 = A() + A()");
}

#[test]
fn overload_minus() {
    ok("class A impl A { fun minus(rhs: A): Int32 = 0 }
            fun f(): Int32 = A() - A()");
}

#[test]
fn overload_times() {
    ok("class A impl A { fun times(rhs: A): Int32 = 0 }
            fun f(): Int32 = A() * A()");
}

#[test]
fn overload_div() {
    ok("class A impl A { fun div(rhs: A): Int32 = 0 }
            fun f(): Int32 = A() / A()");
}

#[test]
fn overload_bitwise_or() {
    ok("class A impl A { fun bitwiseOr(rhs: A): Int32 = 0 }
            fun f(): Int32 = A() | A()");
}

#[test]
fn overload_bitwise_and() {
    ok("class A impl A { fun bitwiseAnd(rhs: A): Int32 = 0i32 }
            fun f(): Int32 = A() & A()");
}

#[test]
fn overload_bitwise_xor() {
    ok("class A impl A { fun bitwiseXor(rhs: A): Int32 = 0i32 }
            fun f(): Int32 = A() ^ A()");
}

#[test]
fn overload_equals() {
    ok("class A impl A { fun equals(rhs: A): Bool = true }
            fun f1(): Bool = A() == A()
            fun f2(): Bool = A() != A()");
}

#[test]
fn overload_compare_to() {
    ok("class A impl A { fun compareTo(rhs: A): Int32 = 0 }
            fun f1(): Bool = A() < A()
            fun f2(): Bool = A() <= A()
            fun f3(): Bool = A() > A()
            fun f4(): Bool = A() >= A()");
}

#[test]
fn int64_operations() {
    ok("fun f(a: Int64, b: Int64): Int64 = a + b");
    ok("fun f(a: Int64, b: Int64): Int64 = a - b");
    ok("fun f(a: Int64, b: Int64): Int64 = a * b");
    ok("fun f(a: Int64, b: Int64): Int64 = a / b");
    ok("fun f(a: Int64, b: Int64): Int64 = a | b");
    ok("fun f(a: Int64, b: Int64): Int64 = a & b");
    ok("fun f(a: Int64, b: Int64): Int64 = a ^ b");
    ok("fun f(a: Int64, b: Int64): Bool = a == b");
    ok("fun f(a: Int64, b: Int64): Bool = a != b");
    ok("fun f(a: Int64, b: Int64): Bool = a === b");
    ok("fun f(a: Int64, b: Int64): Bool = a !== b");
    ok("fun f(a: Int64, b: Int64): Bool = a < b");
    ok("fun f(a: Int64, b: Int64): Bool = a <= b");
    ok("fun f(a: Int64, b: Int64): Bool = a > b");
    ok("fun f(a: Int64, b: Int64): Bool = a >= b");
    ok("fun f(a: Int64): Int64 = -a");
    ok("fun f(a: Int64): Int64 = +a");
}

#[test]
fn test_literal_int_overflow() {
    err(
        "fun f(): Unit { let x = 2147483648i32 }",
        pos(1, 25),
        ErrorMessage::NumberOverflow("Int32".into()),
    );
    ok("fun f(): Unit { let x = 2147483647i32 }");
    err(
        "fun f(): Unit { let x = -2147483649i32 }",
        pos(1, 26),
        ErrorMessage::NumberOverflow("Int32".into()),
    );
    ok("fun f(): Unit { let x = -2147483648i32 }");
}

#[test]
fn test_literal_hex_int_overflow() {
    err(
        "fun f(): Unit { let x = 0x1'FF'FF'FF'FFi32 }",
        pos(1, 25),
        ErrorMessage::NumberOverflow("Int32".into()),
    );
    ok("fun f(): Unit { let x: Int32 = 0xFF'FF'FF'FFi32 }");
}

#[test]
fn test_literal_bin_int_overflow() {
    err(
        "fun f(): Unit { let x = 0b1'11111111'11111111'11111111'11111111i32 }",
        pos(1, 25),
        ErrorMessage::NumberOverflow("Int32".into()),
    );
    ok("fun f(): Unit { let x: Int32 = 0b11111111'11111111'11111111'11111111i32 }");
}

#[test]
fn test_literal_int64_overflow() {
    err(
        "fun f(): Unit { let x = 9223372036854775808i64 }",
        pos(1, 25),
        ErrorMessage::NumberOverflow("Int64".into()),
    );
    ok("fun f(): Unit { let x = 9223372036854775807i64 }");
    err(
        "fun f(): Unit { let x = -9223372036854775809i64 }",
        pos(1, 26),
        ErrorMessage::NumberOverflow("Int64".into()),
    );
    ok("fun f(): Unit { let x = -9223372036854775808i64 }");
}

#[test]
fn test_literal_float_overflow() {
    err(
        "fun f(): Unit { let x = -340282350000000000000000000000000000000f32 }",
        pos(1, 26),
        ErrorMessage::NumberOverflow("Float32".into()),
    );
    ok("fun f(): Unit { let x = -340282340000000000000000000000000000000f32 }");
    err(
        "fun f(): Unit { let x = 340282350000000000000000000000000000001f32 }",
        pos(1, 25),
        ErrorMessage::NumberOverflow("Float32".into()),
    );
    ok("fun f(): Unit { let x = 340282340000000000000000000000000000000f32 }");
}

#[test]
fn test_char() {
    ok("fun foo(): Char = 'c'");
    ok("fun foo(a: Char): Char = a");
    err(
        "fun foo(): Char = false",
        pos(1, 19),
        ErrorMessage::ReturnType("Char".into(), "Bool".into()),
    );
    err(
        "fun foo(): Char = 10i32",
        pos(1, 19),
        ErrorMessage::ReturnType("Char".into(), "Int32".into()),
    );
}

#[test]
fn test_generic_arguments_mismatch() {
    err(
        "class A[T]
            fun foo(): Unit {
                let a = A[Int32, Int32]()
            }",
        pos(3, 40),
        ErrorMessage::WrongNumberTypeParams(1, 2),
    );

    err(
        "class A[T]
            fun foo(): Unit {
                let a = A()
            }",
        pos(3, 26),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );

    err(
        "class A
            fun foo(): Unit {
                let a = A[Int32]()
            }",
        pos(3, 33),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
}

#[test]
fn test_invoke_static_method_as_instance_method() {
    err(
        "class A
        impl A {
            @static fun foo(): Unit {}
            fun test(): Unit = self.foo()
        }",
        pos(4, 40),
        ErrorMessage::UnknownMethod("A".into(), "foo".into(), vec![]),
    );
}

#[test]
fn test_invoke_method_as_static() {
    err(
        "class A
        impl A {
            fun foo(): Unit {}
            @static fun test(): Unit = A::foo()
        }",
        pos(4, 46),
        ErrorMessage::UnknownStaticMethod("A".into(), "foo".into(), vec![]),
    );
}

#[test]
fn test_fct_with_type_params() {
    err(
        "fun f(): Unit {} fun g(): Unit = f[Int32]()",
        pos(1, 42),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
    err(
        "fun f[T](): Unit {} fun g(): Unit = f()",
        pos(1, 38),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );
    ok("fun f[T](): Unit {} fun g(): Unit = f[Int32]()");
    ok("fun f[T1, T2](): Unit {} fun g(): Unit = f[Int32, String]()");
}

#[test]
fn test_type_param_bounds_in_definition() {
    err(
        "
            trait MyTrait {}
            class Foo[T: MyTrait]
            fun bar[T](arg: Foo[T]): Unit {}
        ",
        pos(4, 29),
        ErrorMessage::TypeNotImplementingTrait("T".into(), "MyTrait".into()),
    );

    err(
        "
            trait MyTraitA {}
            trait MyTraitB {}
            class Foo[T: MyTraitA + MyTraitB]
            fun bar[T: MyTraitA](arg: Foo[T]): Unit {}
        ",
        pos(5, 39),
        ErrorMessage::TypeNotImplementingTrait("T".into(), "MyTraitB".into()),
    );

    err(
        "
            trait MyTraitA {}
            trait MyTraitB {}
            class Foo[T: MyTraitA + MyTraitB]
            class Baz[X]
            impl[X] Baz[X] {
                fun bar[T: MyTraitA](arg: Foo[T]): Unit {}
            }
        ",
        pos(7, 43),
        ErrorMessage::TypeNotImplementingTrait("T".into(), "MyTraitB".into()),
    );
}

#[test]
fn test_const_check() {
    err(
        "const one: Int32 = 1i32;
            fun f(): Int64 = one",
        pos(2, 30),
        ErrorMessage::ReturnType("Int64".into(), "Int32".into()),
    );

    err(
        "const one: Int32 = 1i32;
            fun f(): Unit { let x: String = one }",
        pos(2, 29),
        ErrorMessage::AssignType("x".into(), "String".into(), "Int32".into()),
    );
}

#[test]
fn test_const_values() {
    ok_with_test(
        "  const yes: Bool = true
                        const x: UInt8 = 255u8
                        const a: Int32 = 100i32
                        const b: Int64 = 200i64
                        const c: Char = 'A'
                        const d: Float32 = 3.0f32
                        const e: Float64 = 6.0",
        |sa| {
            {
                let id = sa.const_by_name("yes");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Bool(true), const_.value);
            }

            {
                let id = sa.const_by_name("x");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Int(255), const_.value);
            }

            {
                let id = sa.const_by_name("a");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Int(100), const_.value);
            }

            {
                let id = sa.const_by_name("b");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Int(200), const_.value);
            }

            {
                let id = sa.const_by_name("c");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Char('A'), const_.value);
            }

            {
                let id = sa.const_by_name("d");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Float(3.0), const_.value);
            }

            {
                let id = sa.const_by_name("e");
                let const_ = sa.consts.idx(id);
                let const_ = const_.read();
                assert_eq!(ConstValue::Float(6.0), const_.value);
            }
        },
    );
}

#[test]
fn test_assignment_to_const() {
    err(
        "const one: Int32 = 1i32
            fun f(): Unit { one = 2i32 }",
        pos(2, 29),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn test_unary_minus_byte() {
    err(
        "const m1: UInt8 = -1u8",
        pos(1, 19),
        ErrorMessage::UnOpType("-".into(), "UInt8".into()),
    );
    ok("const m1: Int32 = -1i32");
    ok("const m1: Int64 = -1i64");
}

#[test]
fn test_generic_trait_bounds() {
    ok("trait Foo {}
            class X
            impl Foo for X {}
            class A[T: Foo]
            fun f(): A[X] { A[X]() }");

    err(
        "trait Foo {}
            class X
            class A[T: Foo]
            fun f(): A[X] { A[X]() }",
        pos(4, 22),
        ErrorMessage::TypeNotImplementingTrait("X".into(), "Foo".into()),
    );

    err(
        "trait Foo {}
            fun f[T: Foo](): Unit {}
            fun t(): Unit = f[Int32]()",
        pos(3, 37),
        ErrorMessage::TypeNotImplementingTrait("Int32".into(), "Foo".into()),
    );
}

#[test]
fn test_operator_on_generic_type() {
    err(
        "fun f[T](a: T, b: T): Unit = a + b",
        pos(1, 32),
        ErrorMessage::BinOpType("+".into(), "T".into(), "T".into()),
    );
}

#[test]
fn test_find_class_method_precedence() {
    // finding class method should have precedence over
    // trait methods
    ok("class A
            impl A { fun foo(): Unit {} }
            trait Foo { fun foo(): Unit }
            impl Foo for A { fun foo(): Unit {} }
            fun test(a: A): Unit = a.foo()");

    err(
        "class A
            impl A { fun foo(): Unit {} }
            trait Foo { fun foo(a: Int32): Unit }
            impl Foo for A { fun foo(a: Int32): Unit {} }
            fun test(a: A): Unit = a.foo(1i32)",
        pos(5, 41),
        ErrorMessage::ParamTypesIncompatible("foo".into(), Vec::new(), vec!["Int32".into()]),
    );

    ok("class A
            impl A { @static fun foo(): Unit {} }
            trait Foo { fun foo(a: Int32): Unit }
            impl Foo for A { fun foo(a: Int32): Unit {} }
            fun test(a: A): Unit = a.foo(1i32)");
}

#[test]
fn test_global_get() {
    ok("var x: Int32 = 0i32; fun foo(): Int32 = x");
}

#[test]
fn test_global_set() {
    ok("var x: Int32 = 0i32; fun foo(a: Int32): Unit { x = a }");
    err(
        "let x: Int32 = 0i32; fun foo(a: Int32): Unit { x = a }",
        pos(1, 50),
        ErrorMessage::LetReassigned,
    );
}

#[test]
fn lambda_assignment() {
    ok("fun f(): Unit { let x = ||: Int32 { 2i32 } }");
    ok("fun f(): Unit { let x: (): () = ||: Unit {} }");
    ok("fun f(): Unit { let x: (): Unit = ||: () {} }");
    ok("fun f(): Unit { let x: (): Int32 = ||: Int32 { 2i32 } }");
    err(
        "fun f(): Unit { let x: (): Int32 = ||: Unit {} }",
        pos(1, 17),
        ErrorMessage::AssignType("x".into(), "() -> Int32".into(), "() -> ()".into()),
    );
}

#[test]
fn method_call_with_multiple_matching_traits() {
    err(
        "class A
            trait X { fun f(): Unit }
            trait Y { fun f(): Unit }

            impl X for A { fun f(): Unit {} }
            impl Y for A { fun f(): Unit {} }

            fun g(a: A): Unit = a.f()",
        pos(8, 36),
        ErrorMessage::MultipleCandidatesForMethod("A".into(), "f".into(), Vec::new()),
    );

    err(
        "class A
            trait X { fun f(x: Int64): Unit }
            trait Y { fun f(x: Int64): Unit }

            impl X for A { fun f(x: Int64): Unit {} }
            impl Y for A { fun f(x: Int64): Unit {} }

            fun g(a: A): Unit = a.f(123)",
        pos(8, 36),
        ErrorMessage::MultipleCandidatesForMethod(
            "A".into(),
            "f".into(),
            vec!["Int64".to_string()],
        ),
    );
}

#[test]
fn generic_trait_method_call() {
    ok("trait Foo { fun bar(): Unit }
            fun f[T: Foo](t: T): Unit = t.bar()");
    ok("trait Foo { fun bar(): Unit }
            class A[T: Foo](t: T)
            impl[T: Foo] A[T] {
                fun baz(): Unit = self.t.bar()
            }");
}

#[test]
fn test_generic_ctor_without_type_params() {
    err(
        "class Foo[A, B]
            fun test(): Unit = Foo()",
        pos(2, 35),
        ErrorMessage::WrongNumberTypeParams(2, 0),
    );
}

#[test]
fn test_generic_argument_with_trait_bound() {
    err(
        "fun f[X: std::Comparable](x: X): Unit {}
            fun g[T](t: T): Unit = f[T](t)",
        pos(2, 40),
        ErrorMessage::TypeNotImplementingTrait("T".into(), "Comparable".into()),
    );
}

#[test]
fn test_for_supports_make_iterator() {
    err(
        "fun f(): Unit { for i in 1i32 {} }",
        pos(1, 26),
        ErrorMessage::TypeNotUsableInForIn("Int32".into()),
    );

    err(
        "
            class Foo
            impl Foo { fun iterator(): Bool = true }
            fun f(): Unit { for i in Foo() {} }",
        pos(4, 41),
        ErrorMessage::TypeNotUsableInForIn("Foo".into()),
    );

    ok("
            class Foo
            impl Foo { fun iterator(): FooIter = FooIter() }
            class FooIter
            impl std::Iterator for FooIter {
                fun next(): Option[Int32] = Some[Int32](0i32)
            }
            fun f(): Int32 { for i in Foo() { return i; } return 0i32; }");
}

#[test]
fn test_ctor_with_type_param() {
    err(
        "
            class Foo[T]
            impl[T] Foo[T] {
                fun foo(a: Int32): Unit { Bar[T](a); () }
            }

            class Bar[T](a: T)
            ",
        pos(4, 49),
        ErrorMessage::ParamTypesIncompatible("Bar".into(), vec!["T".into()], vec!["Int32".into()]),
    );
}

#[test]
fn test_fct_used_as_identifier() {
    err(
        "fun foo(): Unit {} fun bar(): Unit = foo",
        pos(1, 38),
        ErrorMessage::ValueExpected,
    );
}

#[test]
fn test_cls_used_as_identifier() {
    err(
        "class X fun f(): Unit = X",
        pos(1, 25),
        ErrorMessage::ValueExpected,
    );
}

#[test]
fn test_assign_fct() {
    err(
        "fun foo(): Unit {} fun bar(): Unit { foo = 1i32 }",
        pos(1, 38),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn test_assign_class() {
    err(
        "
            class X
            fun foo(): Unit { X = 2i32 }
        ",
        pos(3, 31),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn test_new_call_fct() {
    ok("fun g(): Unit {} fun f(): Unit = g()");
}

#[test]
fn test_new_call_fct_wrong_params() {
    err(
        "fun g(): Unit {} fun f(): Unit = g(1i32)",
        pos(1, 35),
        ErrorMessage::ParamTypesIncompatible("g".into(), Vec::new(), vec!["Int32".into()]),
    );
}

#[test]
fn test_new_call_fct_with_type_params() {
    ok("fun g[T](): Unit {} fun f(): Unit = g[Int32]()");
}

#[test]
fn test_new_call_fct_with_wrong_type_params() {
    err(
        "fun g(): Unit {} fun f(): Unit = g[Int32]()",
        pos(1, 42),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
}

#[test]
fn test_new_call_static_method() {
    ok("class Foo impl Foo { @static fun bar(): Unit {} }
            fun f(): Unit = Foo::bar()");
}

#[test]
fn test_new_call_static_method_wrong_params() {
    err(
        "class Foo impl Foo { @static fun bar(): Unit {} }
            fun f(): Unit = Foo::bar(1i32)",
        pos(2, 37),
        ErrorMessage::ParamTypesIncompatible("bar".into(), Vec::new(), vec!["Int32".into()]),
    );
}

#[test]
fn test_new_call_static_method_type_params() {
    ok("class Foo impl Foo { @static fun bar[T](): Unit {} }
            fun f(): Unit = Foo::bar[Int32]()");
}

#[test]
fn test_new_call_class() {
    ok("
        class X
        fun f(): Unit { X(); () }
    ");
}

#[test]
fn test_new_call_class_wrong_params() {
    err(
        "
        class X
        fun f(): Unit { X(1i32); () }
    ",
        pos(3, 26),
        ErrorMessage::ParamTypesIncompatible("X".into(), Vec::new(), vec!["Int32".into()]),
    );
}

#[test]
fn test_new_call_class_with_type_params() {
    ok("
        class X[T]
        fun f(): Unit { X[Int32](); () }
    ");
}

#[test]
fn test_new_call_class_with_wrong_type_params() {
    err(
        "
            class X
            fun f(): Unit { X[Int32](); () }
        ",
        pos(3, 37),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
}

#[test]
fn test_new_call_method() {
    ok("
        class X
        impl X { fun f(): Unit {} }
        fun f(x: X): Unit = x.f()
    ");
}

#[test]
fn test_new_call_method_type_param() {
    ok("
        class X
        impl X { fun f[T](): Unit {} }
        fun f(x: X): Unit = x.f[Int32]()
    ");
}

#[test]
fn test_new_call_method_wrong_params() {
    err(
        "
        class X
        impl X { fun f(): Unit {} }
        fun f(x: X): Unit = x.f(1i32)",
        pos(4, 32),
        ErrorMessage::ParamTypesIncompatible("f".into(), Vec::new(), vec!["Int32".into()]),
    );
}

#[test]
fn test_new_call_method_generic() {
    ok("fun f[T: std::Hash](t: T): Unit { t.hash; () }");
}

#[test]
fn test_new_call_method_generic_error() {
    err(
        "fun f[T](t: T): Unit = t.hash()",
        pos(1, 30),
        ErrorMessage::UnknownMethodForTypeParam("T".into(), "hash".into(), Vec::new()),
    );
}

#[test]
fn test_new_call_method_generic_error_multiple() {
    err(
        "
            trait TraitA { fun id(): Unit }
            trait TraitB { fun id(): Unit }
            fun f[T: TraitA + TraitB](t: T): Unit = t.id()",
        pos(4, 57),
        ErrorMessage::MultipleCandidatesForTypeParam("T".into(), "id".into(), Vec::new()),
    );
}

#[test]
fn test_named_arguments_fail_method() {
    err(
        "
            class Foo()
            impl Foo { fun foo(x: Int64, y: Bool): Unit {} }
            fun x(): Unit = Foo().foo(z = 23, y = true)",
        pos(4, 38),
        ErrorMessage::ArgumentNameMismatch(
            "foo".into(),
            vec!["x: Int64".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_named_arguments_fail_method_static() {
    err(
        "
            class Foo()
            impl Foo { @static fun foo(x: Int64, y: Bool): Unit {} }
            fun x(): Unit { Foo::foo(z = 23, y = true); }",
        pos(4, 37),
        ErrorMessage::ArgumentNameMismatch(
            "foo".into(),
            vec!["x: Int64".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_named_arguments_fail_method_static_generic() {
    err(
        "
            class Foo()
            impl Foo { @static fun foo[T](x: T, y: Bool): Unit {} }
            fun x(): Unit = Foo::foo[Int64](z = 23, y = true)",
        pos(4, 44),
        ErrorMessage::ArgumentNameMismatch(
            "foo".into(),
            vec!["x: T".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_named_arguments_fail_function() {
    err(
        "
            fun foo(x: Int64, y: Bool): Unit {}
            fun x(): Unit = foo(z = 23, y = true)",
        pos(3, 32),
        ErrorMessage::ArgumentNameMismatch(
            "foo".into(),
            vec!["x: Int64".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_named_arguments_fail_class() {
    err(
        "
            class Foo(x: Int64, y: Bool)
            fun x(): Unit { Foo(z = 23, y = true); () }",
        pos(3, 32),
        ErrorMessage::ArgumentNameMismatch(
            "Foo".into(),
            vec!["x: Int64".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_named_arguments_fail_value() {
    err(
        "
            value Foo(x: Int64, y: Bool)
            fun x(): Unit { Foo(z = 23, y = true); () }",
        pos(3, 32),
        ErrorMessage::ArgumentNameMismatch(
            "Foo".into(),
            vec!["x: Int64".into(), "y: Bool".into()],
            vec!["z: Int64".into(), "y: Bool".into()],
        ),
    );
}

#[test]
fn test_array_syntax_get() {
    ok("fun f(t: Array[Int32]): Int32 = t(0)");
}

#[test]
fn test_array_syntax_set() {
    ok("fun f(t: Array[Int32]): Unit { t(0) = 10i32 }");
}

#[test]
fn test_array_syntax_set_wrong_value() {
    err(
        "fun f(t: Array[Int32]): Unit { t(0) = true }",
        pos(1, 37),
        ErrorMessage::UnknownMethod(
            "Array[Int32]".into(),
            "set".into(),
            vec!["Int64".into(), "Bool".into()],
        ),
    );
}

#[test]
fn test_array_syntax_set_wrong_index() {
    err(
        "fun f(t: Array[Int32]): Unit { t(\"bla\") = 9i32 }",
        pos(1, 41),
        ErrorMessage::UnknownMethod(
            "Array[Int32]".into(),
            "set".into(),
            vec!["String".into(), "Int32".into()],
        ),
    );
}

#[test]
fn test_template() {
    ok("fun f(x: Int32): String = \"x = ${x}\"");
    err(
        "
            class Foo
            fun f(x: Foo): String = \"x = ${x}\"
        ",
        pos(3, 44),
        ErrorMessage::ExpectedStringable("Foo".into()),
    );
    ok("fun f[T: std::Stringable](x: T): String = \"${x}\"");
}

#[test]
fn test_trait_object_as_argument() {
    ok("trait Foo { fun bar(): Int32 }
        fun f(x: Foo): Int32 = x.bar()");
    err(
        "trait Foo { fun baz(): Unit }
        fun f(x: Foo): String = x.baz()",
        pos(2, 38),
        ErrorMessage::ReturnType("String".into(), "()".into()),
    );
}

#[test]
fn test_type_param_used_as_value() {
    err(
        "fun f[T](): Int32 = T",
        pos(1, 21),
        ErrorMessage::ValueExpected,
    );

    err(
        "class SomeClass[T]
        impl[T] SomeClass[T] {
            fun f(): Int32 = T
        }",
        pos(3, 30),
        ErrorMessage::ValueExpected,
    );
}

#[test]
fn test_assign_to_type_param() {
    err(
        "fun f[T](): Unit { T = 10 }",
        pos(1, 20),
        ErrorMessage::LvalueExpected,
    );

    err(
        "
        class SomeClass[T]
        impl[T] SomeClass[T] {
            fun f(): Unit { T = 10; }
        }",
        pos(4, 29),
        ErrorMessage::LvalueExpected,
    );
}

#[test]
fn test_type_param_with_name_but_no_call() {
    err(
        "trait X { fun foo(): Int32; }
        fun f[T: X](): Unit { T::foo; }",
        pos(2, 32),
        ErrorMessage::UnknownStaticMethodWithTypeParam,
    );

    err(
        "trait X { fun foo(): Int32; }
        class SomeClass[T: X]
        impl[T: X] SomeClass[T] {
            fun f(): Unit { T::foo; }
        }",
        pos(4, 30),
        ErrorMessage::UnknownStaticMethodWithTypeParam,
    );
}

#[test]
fn test_type_param_call() {
    err(
        "trait X { fun foo(): Int32; }
        fun f[T: X](): Unit { T(); }",
        pos(2, 31),
        ErrorMessage::ValueExpected,
    );

    err(
        "trait X { fun foo(): Int32; }
        class SomeClass[T: X]
        impl[T: X] SomeClass[T] {
            fun f(): Unit { T(); }
        }",
        pos(4, 29),
        ErrorMessage::ValueExpected,
    );
}

#[test]
fn test_static_method_call_with_type_param() {
    err(
        "trait X { @static fun bar(): Int32; }
        fun f[T: X](): Unit { T::foo(); }",
        pos(2, 37),
        ErrorMessage::UnknownStaticMethodWithTypeParam,
    );

    err(
        "trait X { @static fun foo(): Int32; }
        trait Y { @static fun foo(): String; }
        fun f[T: X + Y](): Unit { T::foo(); }",
        pos(3, 41),
        ErrorMessage::MultipleCandidatesForStaticMethodWithTypeParam,
    );

    err(
        "trait X { @static fun foo(): Int32; }
        fun f[T: X](): Int32 { return T::foo(1i32); }",
        pos(2, 45),
        ErrorMessage::ParamTypesIncompatible("foo".into(), Vec::new(), vec!["Int32".into()]),
    );

    ok("trait X { @static fun foo(): Int32; }
        fun f[T: X](): Int32 { return T::foo(); }");
}

#[test]
fn test_type_param_with_let() {
    ok("fun myid[T](val: T): T {
        let tmp: T = val;
        return tmp;
    }");
}

#[test]
fn test_fct_and_class_type_params() {
    ok("
    class A[X]
    impl[X] A[X] {
        fun test[Y](): Unit {}
    }");

    ok("
    class A[X]
    impl[X] A[X] {
        fun t1[Y](x: X, y: Y): Y { return y; }
        fun t2[Y](x: X, y: Y): X { return x; }
    }

    fun t1(a: A[Int32]): String {
        return a.t1[String](1i32, \"bla\");
    }

    fun t2(a: A[Int32]): Int32 {
        return a.t2[String](1i32, \"bla\");
    }
    ");
}

#[test]
fn test_value() {
    ok("
        value Foo(f1: Int32)
        fun f(): Foo { Foo(1i32) }
    ");
    err(
        "
        value Foo(f1: Int32)
        fun f(): Foo { Foo() }",
        pos(3, 27),
        ErrorMessage::ValueArgsIncompatible("Foo".into(), vec!["Int32".into()], Vec::new()),
    );
    err(
        "
        value Foo(f1: Int32)
        fun f(): Foo { Foo(true) }",
        pos(3, 27),
        ErrorMessage::ValueArgsIncompatible(
            "Foo".into(),
            vec!["Int32".into()],
            vec!["Bool".into()],
        ),
    );
}

#[test]
fn test_value_field() {
    ok("
        value Foo(f1: Int32)
        fun f(x: Foo): Int32 { x.f1 }
    ");

    err(
        "
        value Foo(f1: Bool)
        fun f(x: Foo): Int32 { x.f1 }
    ",
        pos(3, 30),
        ErrorMessage::ReturnType("Int32".into(), "Bool".into()),
    );

    err(
        "
        value Foo(f1: Bool)
        fun f(x: Foo): Int32 { x.unknown }
    ",
        pos(3, 33),
        ErrorMessage::UnknownField("unknown".into(), "Foo".into()),
    );
}

#[test]
fn test_value_field_array() {
    ok("
        value Foo(f1: Array[Int32])
        fun f(x: Foo): Int32 { x.f1(0) }
    ");
}

#[test]
fn test_value_with_type_params() {
    ok("
        value Foo[T](f1: Int32)
        fun f(): Foo[Int32] { Foo[Int32](1i32) }
    ");
    err(
        "
        value Foo[T](f1: Int32)
        fun f(): Foo[Int32] { Foo(1i32) }
    ",
        pos(3, 34),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );
    err(
        "
        value Foo[T](f1: Int32)
        fun f(): Foo[Int32] { Foo[Int32, Bool](1i32) }
    ",
        pos(3, 47),
        ErrorMessage::WrongNumberTypeParams(1, 2),
    );
    err(
        "
        trait MyTrait {}
        value Foo[T: MyTrait](f1: Int32)
        fun f(): Foo[Int32] { Foo[Int32](1i32) }
    ",
        pos(4, 18),
        ErrorMessage::TypeNotImplementingTrait("Int32".into(), "MyTrait".into()),
    );
    ok("
        trait MyTrait {}
        class Bar
        impl MyTrait for Bar {}
        value Foo[T: MyTrait](f1: Int32)
        fun f(): Foo[Bar] { Foo[Bar](1i32) }
    ");
    err(
        "
        value Foo[T](f1: Int32)
        fun f(): Foo[Int32] { Foo[Bool](1i32) }
    ",
        pos(3, 29),
        ErrorMessage::ReturnType("Foo[Int32]".into(), "Foo[Bool]".into()),
    );
    err(
        "
        value Foo[T](f1: T, f2: Bool)
        fun f[T](val: T): Foo[T] { Foo(val, false) }",
        pos(3, 39),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );
}

#[test]
fn test_value_mod() {
    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo { value Foo(f1: Int32) }
        ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );
}

#[test]
fn test_value_with_static_method() {
    ok("
        value Foo(value: Int32)
        impl Foo {
            @static fun bar(): Unit {}
        }
        fun f(): Unit = Foo::bar()
        ");

    ok("
        value Foo[T](value: Int32)
        impl[T] Foo[T] {
            @static fun bar(): Unit {}
        }
        fun f(): Unit = Foo[Int32]::bar()
        ");

    err(
        "
            value Foo(value: Int32)
            fun f(): Unit = Foo[Int32]::bar()
            ",
        pos(3, 44),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
}

#[test]
fn test_enum_with_static_method() {
    ok("
        enum Foo { A, B }
        impl Foo {
            @static fun bar(): Unit {}
        }
        fun f(): Unit = Foo::bar()
        ");

    err(
        "
        enum Foo { A, B }
        fun f(): Unit = Foo[Int32]::bar()
        ",
        pos(3, 40),
        ErrorMessage::WrongNumberTypeParams(0, 1),
    );
}

#[test]
fn test_enum() {
    ok("enum A { V1, V2 }");
    ok("enum A { V1, V2 } fun f(a: A): A = a");
    ok("enum A { V1, V2 } fun f(): A = A::V1");

    ok("enum A { V1, V2 } fun f(): Bool = A::V1 == A::V2");
    ok("enum A { V1, V2 } fun f(): Bool = A::V1 != A::V2");

    err(
        "enum A { V1 } fun f(): A = A ",
        pos(1, 28),
        ErrorMessage::ValueExpected,
    );

    err(
        "enum A { V1 } fun f(): Unit { A = 1; }",
        pos(1, 31),
        ErrorMessage::LvalueExpected,
    );

    err(
        "enum A { V1, V2 } fun f(): A = A::V3",
        pos(1, 33),
        ErrorMessage::UnknownEnumVariant("V3".into()),
    );

    err(
        "enum A[T] { V1, V2 } fun f(): A[Int32] = A::V1",
        pos(1, 43),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );

    err(
        "enum A[T] { V1(T), V2 } fun f(): A[Int32] = A[Int32]::V1",
        pos(1, 53),
        ErrorMessage::EnumArgsIncompatible("A".into(), "V1".into(), vec!["T".into()], Vec::new()),
    );

    err(
        "
        enum Foo[T] { A(T, Bool), B }
        fun f[T](val: T): Foo[T] { Foo::A[T, String](val, false) }",
        pos(3, 53),
        ErrorMessage::WrongNumberTypeParams(1, 2),
    );

    ok("
        enum Foo[T] { A(T, Bool), B }
        fun f[T](val: T): Foo[T] { Foo::A(val, false) }");
}

#[test]
fn test_enum_if_pattern() {
    ok("
        enum A { V1, V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1 { 0i32 }
          ... is A::V2 { 1i32 }
    ");

    err(
        "
        enum A { V1, V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1 { 0i32 }
          ... is A::V2 { \"foo\" }
    ",
        pos(5, 24),
        ErrorMessage::IfBranchTypesIncompatible("Int32".into(), "String".into()),
    );
}

#[test]
fn test_enum_if_pattern_with_parens() {
    err(
        "
        enum A { V1, V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1() { 0i32 }
          ... is A::V2   { 1i32 }
    ",
        pos(4, 18),
        ErrorMessage::IfPatternNoParens,
    );
}

#[test]
fn test_enum_if_pattern_wrong_number_params() {
    err(
        "
        enum A { V1(Int32), V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1 { 0i32 }
          ... is A::V2 { 1i32 }
    ",
        pos(4, 18),
        ErrorMessage::IfPatternWrongNumberOfParams(0, 1),
    );

    err(
        "
        enum A { V1(Int32, Float32, Bool), V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, b, c, d) { 0i32 }
          ... is A::V2             { 1i32 }
    ",
        pos(4, 18),
        ErrorMessage::IfPatternWrongNumberOfParams(4, 3),
    );
}

#[test]
fn test_enum_if_pattern_params() {
    ok("
        enum A { V1(Int32, Int32, Int32), V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, _, c) { a + c }
          ... is A::V2          { 1i32  }
    ");

    err(
        "
        enum A { V1(Int32, Int32, Int32), V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, _, c) { a + c }
          ... is A::V2          { a     }
    ",
        pos(5, 35),
        ErrorMessage::UnknownIdentifier("a".into()),
    );

    err(
        "
        enum A { V1(Int32, Int32), V2 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, a) { a + a }
          ... is A::V2       { 1i32  }
    ",
        pos(4, 27),
        ErrorMessage::IfPatternBindingAlreadyUsed,
    );
}

#[test]
fn test_enum_if_pattern_missing_variants() {
    err(
        "
        enum A { V1(Int32, Int32, Int32), V2, V3 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, _, c) { a + c }
          ... is A::V2          { 1i32  }
    ",
        pos(3, 30),
        ErrorMessage::IfPatternVariantUncovered,
    );
}

#[test]
fn test_enum_if_pattern_unreachable_variants() {
    err(
        "
        enum A { V1(Int32, Int32, Int32), V2, V3 }
        fun f(x: A): Int32 = if  x
          ... is A::V1(a, _, c) { a + c }
          ... is A::V2          { 1i32  }
          ... is A::V3          { 2i32  }
          ... is A::V2          { 4i32  }
    ",
        pos(7, 18),
        ErrorMessage::IfPatternUnreachable,
    );

    err(
        "
        enum A { V1(Int32, Int32, Int32), V2, V3 }
        fun f(x: A): Int32 = if x
          ... is A::V1(a, _, c) { a + c }
          ... is A::V2          { 1i32  }
          ... is A::V3          { 2i32  }
          else                  { 4i32  }
    ",
        pos(7, 33),
        ErrorMessage::IfPatternUnreachable,
    );
}

#[test]
fn test_enum_if_else() {
    ok("
        enum A { V1, V2, V3 }
        fun f(x: A): Bool = if x
          ... is A::V1 { true  }
          else         { false }
    ");
}

#[test]
fn test_enum_equals() {
    ok("
        enum A { V1, V2 }
        fun f(x: A, y: A): Bool = x == y
    ");

    err(
        "
        enum A { V1(Int32), V2 }
        fun f(x: A, y: A): Bool = x == y
    ",
        pos(3, 37),
        ErrorMessage::BinOpType("==".into(), "A".into(), "A".into()),
    );
}

#[test]
fn test_use_enum_value() {
    ok("enum A { V1(Int32), V2 } use A.V1; fun f(): A = V1(1i32)");
    ok("enum A[T] { V1(Int32), V2 } use A.V1; fun f(): A[Int32] = V1[Int32](1i32)");
    ok("enum A[T] { V1(Int32), V2 } use A.V1; fun f(): A[Int32] = V1(1i32)");

    ok("enum A { V1, V2 } use A.V2; fun f(): A = V2");

    err(
        "enum A { V1(Int32), V2 } use A.V1; fun f(): A = V1",
        pos(1, 49),
        ErrorMessage::EnumArgsIncompatible(
            "A".into(),
            "V1".into(),
            vec!["Int32".into()],
            Vec::new(),
        ),
    );

    err(
        "enum A { V1(Int32), V2 } use A.V2; fun f(): A = V2(0i32)",
        pos(1, 51),
        ErrorMessage::EnumArgsIncompatible(
            "A".into(),
            "V2".into(),
            Vec::new(),
            vec!["Int32".into()],
        ),
    );

    ok("enum A[T] { V1(Int32), V2 } use A.V2; fun f(): A[Int32] = V2");

    ok("enum A[T] { V1, V2 } use A.V2; fun f(): A[Int32] = V2[Int32]");

    err(
        "enum A[T] { V1, V2 } use A.V2; fun f(): A[Int32] = V2[Int32, Float32]",
        pos(1, 54),
        ErrorMessage::WrongNumberTypeParams(1, 2),
    );
}

#[test]
fn test_enum_value_with_type_param() {
    ok("enum A[T] { V1, V2 } fun f(): A[Int32] = A::V2[Int32]");
    ok("enum A[T] { V1, V2 } fun f(): A[Int32] = A[Int32]::V2");
    err(
        "enum A[T] { V1, V2 } fun f(): A[Int32] = A[Int32]::V2[Int32]",
        pos(1, 43),
        ErrorMessage::ExpectedSomeIdentifier,
    );
}

#[test]
fn test_block_value() {
    ok("fun f(): Int32 = 1i32");
    ok("fun f(): Unit { let x = { 1i32 } }");
    ok("fun g(): Int32 = 1i32; fun f(): Unit { let x: Int32 = { g() } }");
    ok("fun g(): Int32 = 1i32; fun f(): Unit { let x: Int32 = { g(); 1i32 } }");
}

#[test]
fn test_if_expression() {
    ok("fun f(): Int32 = if true { 1i32 } else { 2i32 }");
    ok("fun f(): Float32 = if true { 1.0f32 } else { 2.0f32 }");
    ok("fun f(): Float64 = if true { 1.0 } else { 2.0 }");
    ok("fun f(): Int32 = 4i32 * if true { 1i32 } else { 2i32 }");
}

#[test]
fn test_tuple() {
    ok("fun f(a: (Int32, Bool)): Unit {}");
    ok("fun f(a: (Int32, Bool)): (Int32, Bool) = a");
    ok("fun f(a: (Int32, Bool)): (Int32, Bool) {
            let tmp = a;
            return tmp;
        }");
    err(
        "fun f(a: (Int32, Bool)): (Int32) = a",
        pos(1, 36),
        ErrorMessage::ReturnType("(Int32)".into(), "(Int32, Bool)".into()),
    );
    err(
        "fun f(a: (Int32, Bool)): (Int32, Float32) = a",
        pos(1, 45),
        ErrorMessage::ReturnType("(Int32, Float32)".into(), "(Int32, Bool)".into()),
    );
}

#[test]
fn test_tuple_literal() {
    ok("fun f(): (Int32, Bool) = (1i32, false)");

    err(
        "fun f(): (Int32) = (1i32)",
        pos(1, 20),
        ErrorMessage::ReturnType("(Int32)".into(), "Int32".into()),
    );

    err(
        "fun f(): (Int32, Int32) = (1i32, false)",
        pos(1, 27),
        ErrorMessage::ReturnType("(Int32, Int32)".into(), "(Int32, Bool)".into()),
    );
}

#[test]
fn test_tuple_in_call() {
    ok("
        fun f(a: (Int32, Bool)): Unit {}
        fun g(): Unit = f((1i32, true))
    ")
}

#[test]
fn test_tuple_element() {
    ok("fun f(a: (Int32, Bool)): Int32 = a.0");

    ok("fun f(a: (Int32, Bool)): Bool = a.1");

    err(
        "fun f(a: (Int32, Bool)): String = a.1",
        pos(1, 36),
        ErrorMessage::ReturnType("String".into(), "Bool".into()),
    );
}

#[test]
fn test_type_without_make_iterator() {
    err(
        "
        class Foo
        fun bar(x: Foo): Unit {
            for i in x {
                let x: Foo = i
            }
        }
    ",
        pos(4, 22),
        ErrorMessage::TypeNotUsableInForIn("Foo".into()),
    );
}

#[test]
fn test_type_make_iterator_not_implementing_iterator() {
    err(
        "
        class Foo
        impl Foo {
            fun iterator(): Int32 = 0i32
        }
        fun bar(x: Foo): Unit {
            for i in x {
                let x: Foo = i
            }
        }
    ",
        pos(7, 22),
        ErrorMessage::TypeNotUsableInForIn("Foo".into()),
    );
}

#[test]
fn zero_trait_ok() {
    ok("fun f(): Unit { Array[Int32]::zero(12i64); () }");
}

#[test]
fn zero_trait_err() {
    err(
        "fun f(): Unit { Array[String]::zero(12i64); }",
        pos(1, 36),
        ErrorMessage::UnknownStaticMethod(
            "Array[String]".into(),
            "zero".into(),
            vec!["Int64".into()],
        ),
    );
}

#[test]
fn extension_method_call() {
    ok("
        class Foo(val: Int32)
        impl Foo { fun foo(): Int32 = self.val }
        fun bar(x: Foo): Int32 = x.foo()
    ");
}

#[test]
fn extension_class_with_type_param() {
    ok("
        class Foo[T](val: T)
        trait MyTrait {}
        impl[T: MyTrait] Foo[T] { fun foo(): Int32 = 12i32 }
        impl MyTrait for Int32 {}
        fun bar(x: Foo[Int32]): Int32 = x.foo()
    ");

    ok("
        class Foo[T](val: T)
        impl Foo[Int32] { fun foo(): Unit {} }
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.foo() }
        fun g(x: Foo[Float32]): Unit { x.bar() }
    ");

    err(
        "
        class Foo[T](value: T)
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(4, 43),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn extension_class_tuple() {
    ok("
        class Foo[T](val: T)
        impl Foo[(Int32, Int32)] {
            fun bar(): Unit {}
        }
        fun f(x: Foo[(Int32, Int32)]): Unit {
            x.bar();
        }
    ");

    ok("
        class Foo[T]
        impl[T] Foo[(T, Int32)] {
            fun bar(): Unit {}
        }
        fun f(): Unit {
            Foo[(Int32, Int32)]().bar()
            Foo[(Float32, Int32)]().bar()
        }
    ");

    err(
        "
        class Foo[T]
        impl Foo[(Int32, Float32)] {
            fun bar(): Unit {}
        }
        fun f(x: Foo[(Int32, Int32)]): Unit {
            x.bar()
        }
    ",
        pos(7, 18),
        ErrorMessage::UnknownMethod("Foo[(Int32, Int32)]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn extension_nested() {
    err(
        "
        class Foo[T]
        impl Foo[Foo[Foo[Int32]]] {
            fun bar(): Unit {}
        }
        fun f(value: Foo[Foo[Foo[Int32]]]): Unit {
            value.bar()
        }
        fun g(value: Foo[Foo[Int32]]): Unit {
            value.bar()
        }
    ",
        pos(10, 22),
        ErrorMessage::UnknownMethod("Foo[Foo[Int32]]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn extension_bind_type_param_twice() {
    ok("
        class Foo[T]
        impl[T] Foo[(T, T)] {
            fun bar(): Unit {}
        }
        fun f(x: Foo[(Int32, Int32)]): Unit { x.bar() }
    ");

    ok("
        class Foo[T]
        impl[T] Foo[(T, T)] {
            fun bar(): Unit {}
        }
        fun f[T](x: Foo[(T, T)]): Unit { x.bar() }
    ");

    err(
        "
        class Foo[T]
        impl[T] Foo[(T, T)] {
            fun bar(): Unit {}
        }
        fun f(x: Foo[(Int32, Float32)]): Unit { x.bar() }
    ",
        pos(6, 54),
        ErrorMessage::UnknownMethod("Foo[(Int32, Float32)]".into(), "bar".into(), Vec::new()),
    );

    err(
        "
        class Foo[T]
        impl[T] Foo[(T, T)] {
            fun bar(): Unit {}
        }
        fun f[T](x: Foo[(T, Float32)]): Unit { x.bar() }
    ",
        pos(6, 53),
        ErrorMessage::UnknownMethod("Foo[(T, Float32)]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn extension_value_with_type_param() {
    ok("
        value Foo[T](val: T)
        trait MyTrait {}
        impl[T: MyTrait] Foo[T] { fun foo(): Int32 { 12i32 } }
        impl MyTrait for Int32 {}
        fun bar(x: Foo[Int32]): Int32 { x.foo() }
    ");

    ok("
        value Foo[T](val: T)
        impl Foo[Int32] { fun foo(): Unit {} }
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.foo() }
        fun g(x: Foo[Float32]): Unit { x.bar() }
    ");

    err(
        "
        value Foo[T](val: T)
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(4, 43),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn extension_enum_with_type_param() {
    ok("
        enum Foo[T] { A(T), B }
        trait MyTrait {}
        impl[T: MyTrait] Foo[T] { fun foo(): Int32 = 12i32 }
        impl MyTrait for Int32 {}
        fun bar(x: Foo[Int32]): Int32 = x.foo()
    ");

    ok("
        enum Foo[T] { A(T), B }
        impl Foo[Int32] { fun foo(): Unit {} }
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.foo() }
        fun g(x: Foo[Float32]): Unit { x.bar() }
    ");

    err(
        "
        enum Foo[T] { A(T), B }
        impl Foo[Float32] { fun bar(): Unit {} }
        fun f(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(4, 43),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );
}

#[test]
fn impl_class_type_params() {
    err(
        "
        trait MyTrait { fun bar(): Unit; }
        class Foo[T]
        impl MyTrait for Foo[String] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(5, 45),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );

    ok("
        trait MyTrait { fun bar(): Unit; }
        class Foo[T]
        impl MyTrait for Foo[Int32] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ");
}

#[test]
fn extension_with_fct_type_param() {
    ok("
        class MyClass[T]
        class Foo
        impl MyClass[Foo] {
            fun do[T](another: MyClass[T]): Unit {}
        }
        fun f(): Unit {
            MyClass[Foo]().do[Int32](MyClass[Int32]())
            MyClass[Foo]().do[Float32](MyClass[Float32]())
        }
    ");
}

#[test]
fn impl_value_type_params() {
    err(
        "
        trait MyTrait { fun bar(): Unit; }
        value Foo[T](value: T)
        impl MyTrait for Foo[String] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(5, 45),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );

    ok("
        trait MyTrait { fun bar(): Unit; }
        value Foo[T](value: T)
        impl MyTrait for Foo[Int32] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ");
}

#[test]
fn impl_value_method_with_self() {
    ok("
        value Foo(value: Int32)
        trait AsInt32 { fun value(): Int32 }
        impl AsInt32 for Foo { fun value(): Int32 = self.value }
    ");
}

#[test]
fn impl_value_with_method_overload() {
    ok("
        value Foo(val: Int32)
        impl Foo {
            fun plus(other: Foo): Foo = Foo(self.val + other.val)
        }
        fun f(a: Foo, b: Foo): Foo = a + b
    ");
}

#[test]
fn impl_enum_type_params() {
    err(
        "
        trait MyTrait { fun bar(): Unit }
        enum Foo[T] { A(T), B }
        impl MyTrait for Foo[String] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ",
        pos(5, 45),
        ErrorMessage::UnknownMethod("Foo[Int32]".into(), "bar".into(), Vec::new()),
    );

    ok("
        trait MyTrait { fun bar(): Unit }
        enum Foo[T] { A(T), B }
        impl MyTrait for Foo[Int32] { fun bar(): Unit {} }
        fun bar(x: Foo[Int32]): Unit { x.bar() }
    ");
}

#[test]
fn method_call_on_unit() {
    err(
        "fun foo(a: ()): Unit { a.foo() }",
        pos(1, 29),
        ErrorMessage::UnknownMethod("()".into(), "foo".into(), Vec::new()),
    );
}

#[test]
fn method_on_enum() {
    ok("
        enum MyEnum { A, B }
        impl MyEnum { fun foo(): Unit {} }
        fun f(x: MyEnum): Unit { x.foo() }
    ");
}

#[test]
fn literal_without_suffix_byte() {
    ok("fun f(): UInt8 = 1");
    err(
        "fun f(): UInt8 = 256",
        pos(1, 18),
        ErrorMessage::NumberOverflow("UInt8".into()),
    );
    ok("fun f(): Unit { let x: UInt8 = 1 }");
}

#[test]
fn literal_without_suffix_long() {
    ok("fun f(): Int64 = 1");
    ok("fun f(): Unit { let x: Int64 = 1 }");
}

#[test]
fn variadic_parameter() {
    ok("
        fun f(x: Int32...): Int64 = x.size
        fun g(): Unit {
            f(1i32, 2i32, 3i32, 4i32)
            f()
            f(1i32);
            ()
        }
    ");
    err(
        "
        fun f(x: Int32...): Unit {}
        fun g(): Unit = f(true)
    ",
        pos(3, 26),
        ErrorMessage::ParamTypesIncompatible("f".into(), vec!["Int32".into()], vec!["Bool".into()]),
    );
    ok("
        fun f(x: Int32, y: Int32...): Unit {}
        fun g(): Unit {
            f(1i32, 2i32, 3i32, 4i32)
            f(1i32, 2i32)
            f(1i32)
        }
    ");
    err(
        "
        fun f(x: Int32, y: Int32...): Unit {}
        fun g(): Unit { f() }
    ",
        pos(3, 26),
        ErrorMessage::ParamTypesIncompatible(
            "f".into(),
            vec!["Int32".into(), "Int32".into()],
            Vec::new(),
        ),
    );
    err(
        "fun f(x: Int32..., y: Int32): Unit {}",
        pos(1, 20),
        ErrorMessage::VariadicParameterNeedsToBeLast,
    );
}

#[test]
fn for_with_array() {
    ok("fun f(x: Array[Int32]): Int32 {
        var result = 0i32
        for i in x {
            result = result + i
        }
        result
    }");

    ok("fun f(x: Array[Float32]): Float32 {
        var result = 0.0f32
        for i in x {
            result = result + i
        }
        result
    }");
}

#[test]
fn for_with_list() {
    ok("fun f(x: List[Int32]): Int32 {
        var result = 0i32
        for i in x.iterator() {
            result = result + i
        }
        result
    }");

    ok("fun f(x: List[Int32]): Int32 {
        var result = 0i32
        for i in x {
            result = result + i
        }
        result
    }");

    ok("fun f(x: List[Float32]): Float32 {
        var result = 0.0f32
        for i in x.iteratorReverse() {
            result = result + i
        }
        result
    }");

    ok("fun f(x: List[Float32]): Float32 {
        var result = 0.0f32
        for i in x {
            result = result + i
        }
        result
    }");
}

#[test]
fn check_no_type_params_with_generic_type() {
    err(
        "
            class Bar[T]
            fun f(x: Bar): Unit {}
        ",
        pos(3, 22),
        ErrorMessage::WrongNumberTypeParams(1, 0),
    );
}

#[test]
fn check_wrong_number_type_params() {
    err(
        "
            fun foo(): Unit { bar[Int32](false); }
            fun bar[T](x: T): Unit {}
        ",
        pos(2, 41),
        ErrorMessage::ParamTypesIncompatible("bar".into(), vec!["T".into()], vec!["Bool".into()]),
    );
}

#[test]
fn multiple_functions() {
    ok("fun f(): Unit {} fun g(): Unit {}");
}

#[test]
fn redefine_function() {
    err(
        "
        fun f(): Unit {}
        fun f(): Unit {}",
        pos(3, 9),
        ErrorMessage::ShadowFunction("f".into()),
    );
}

#[test]
fn shadow_type_with_function() {
    err(
        "
        class FooBar
        fun FooBar(): Unit {}
        ",
        pos(3, 9),
        ErrorMessage::ShadowClass("FooBar".into()),
    );
}

#[test]
fn define_param_name_twice() {
    err(
        "fun test(x: String, x: Int32): Unit {}",
        pos(1, 21),
        ErrorMessage::ShadowParam("x".into()),
    );
}

#[test]
fn show_type_param_with_name() {
    err(
        "fun test[T](T: Int32): Unit {}",
        pos(1, 13),
        ErrorMessage::ShadowTypeParam("T".into()),
    );
}

#[test]
fn shadow_type_with_var() {
    ok("fun test(): Unit { let String = 3i32 }");
}

#[test]
fn shadow_function() {
    ok("fun f(): Unit { let f = 1i32 }");
    err(
        "fun f(): Unit { let f = 1i32; f() }",
        pos(1, 32),
        ErrorMessage::UnknownMethod("Int32".into(), "get".into(), Vec::new()),
    );
}

#[test]
fn shadow_var() {
    ok("fun f(): Unit { let f = 1i32; let f = 2i32 }");
}

#[test]
fn shadow_param() {
    err(
        "fun f(a: Int32, b: Int32, a: String): Unit {}",
        pos(1, 27),
        ErrorMessage::ShadowParam("a".into()),
    );
}

#[test]
fn multiple_params() {
    ok("fun f(a: Int32, b: Int32, c:String): Unit {}");
}

#[test]
fn undefined_variable() {
    err(
        "fun f(): Unit { let b = a }",
        pos(1, 25),
        ErrorMessage::UnknownIdentifier("a".into()),
    );
    err(
        "fun f(): Unit { a }",
        pos(1, 17),
        ErrorMessage::UnknownIdentifier("a".into()),
    );
}

#[test]
fn undefined_function() {
    err(
        "fun f(): Unit { foo() }",
        pos(1, 17),
        ErrorMessage::UnknownIdentifier("foo".into()),
    );
}

#[test]
fn recursive_function_call() {
    ok("fun f(): Unit { f() }");
}

#[test]
fn function_call() {
    ok("fun a(): Unit {} fun b(): Unit { a() }");

    // non-forward definition of functions
    ok("fun a(): Unit { b(); } fun b(): Unit {}");
}

#[test]
fn variable_outside_of_scope() {
    err(
        "fun f(): Int32 { { let a = 1i32 } a }",
        pos(1, 35),
        ErrorMessage::UnknownIdentifier("a".into()),
    );

    ok("fun f(): Int32 { let a = 1i32 { let a = 2i32 } a }");
}

#[test]
fn const_value() {
    ok("const one: Int32 = 1i32; fun f(): Int32 = one");
}

#[test]
fn for_var() {
    ok("fun f(): Unit { for i in std::range(0i32, 4i32) { i } }");
}

#[test]
fn mod_fct_call() {
    err(
        "
        fun f(): Unit = foo::g()
        mod foo { fun g(): Unit {} }
    ",
        pos(2, 31),
        ErrorMessage::NotAccessible("foo::g".into()),
    );

    ok("
        fun f(): Unit = foo::g()
        mod foo { @pub fun g(): Unit {} }
    ");

    ok("
        fun f(): Unit = foo::bar::baz()
        mod foo {
            @pub mod bar {
                @pub fun baz(): Unit {}
            }
        }
    ");

    err(
        "
        fun f(): Unit = foo::bar::baz()
        mod foo {
            @pub mod bar {
                fun baz(): Unit {}
            }
        }
    ",
        pos(2, 38),
        ErrorMessage::NotAccessible("foo::bar::baz".into()),
    );
}

#[test]
fn mod_ctor_call() {
    ok("
        fun f(): Unit { foo::Foo(); () }
        mod foo { @pub class Foo }
    ");

    err(
        "
        fun f(): Unit { foo::Foo(); () }
        mod foo { class Foo }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    ok("
        fun f(): Unit { foo::bar::Foo(); () }
        mod foo { @pub mod bar { @pub class Foo } }
    ");

    err(
        "
        fun f(): Unit { foo::bar::Foo(); () }
        mod foo { @pub mod bar { class Foo } }
    ",
        pos(2, 38),
        ErrorMessage::NotAccessible("foo::bar::Foo".into()),
    );
}

#[test]
fn mod_class_field() {
    err(
        "
        fun f(x: foo::Foo): Unit { let a = x.bar }
        mod foo { @pub class Foo(bar: Int32) }
    ",
        pos(2, 45),
        ErrorMessage::NotAccessible("bar".into()),
    );

    err(
        "
        fun f(x: foo::Foo): Unit { let a = x.bar(10i64) }
        mod foo { @pub class Foo(bar: Array[Int32]) }
    ",
        pos(2, 49),
        ErrorMessage::NotAccessible("bar".into()),
    );

    err(
        "
        fun f(x: foo::Foo): Unit { x.bar(10i64) = 10i32 }
        mod foo { @pub class Foo(bar: Array[Int32]) }
    ",
        pos(2, 37),
        ErrorMessage::NotAccessible("bar".into()),
    );

    ok("
        fun f(x: foo::Foo): Unit { let a = x.bar }
        mod foo { @pub class Foo(@pub bar: Int32) }
    ");
}

#[test]
fn mod_class_method() {
    ok("
        fun f(x: foo::Foo): Unit = x.bar()
        mod foo {
            @pub class Foo
            impl Foo { @pub fun bar(): Unit {} }
        }
    ");

    err(
        "
        fun f(x: foo::Foo): Unit = x.bar()
        mod foo {
            @pub class Foo
            impl Foo { fun bar(): Unit {} }
        }
    ",
        pos(2, 41),
        ErrorMessage::NotAccessible("foo::Foo#bar".into()),
    );
}

#[test]
fn mod_class_static_method() {
    ok("
        fun f(): Unit { foo::Foo::bar(); }
        mod foo {
            @pub class Foo
            impl Foo { @pub @static fun bar(): Unit {} }
        }
    ");

    err(
        "
        fun f(): Unit { foo::Foo::bar(); }
        mod foo {
            @pub class Foo
            impl Foo { @static fun bar(): Unit {} }
        }
    ",
        pos(2, 38),
        ErrorMessage::NotAccessible("foo::Foo::bar".into()),
    );
}

#[test]
fn mod_value_field() {
    err(
        "
        fun f(x: foo::Foo): Unit { let a = x.bar }
        mod foo { @pub value Foo(bar: Int32) }
    ",
        pos(2, 45),
        ErrorMessage::NotAccessible("bar".into()),
    );

    ok("
        fun f(x: foo::Foo): Unit { let a = x.bar(10i64) }
        mod foo { @pub value Foo(@pub bar: Array[Int32]) }
    ");

    err(
        "
        fun f(x: foo::Foo): Unit { let a = x.bar(10i64) }
        mod foo { @pub value Foo(bar: Array[Int32]) }
    ",
        pos(2, 49),
        ErrorMessage::NotAccessible("bar".into()),
    );

    err(
        "
        fun f(x: foo::Foo): Unit { x.bar(10i64) = 10i32 }
        mod foo { @pub value Foo(bar: Array[Int32]) }
    ",
        pos(2, 37),
        ErrorMessage::NotAccessible("bar".into()),
    );

    ok("
        fun f(x: foo::Foo): Unit { let a = x.bar }
        mod foo { @pub value Foo(@pub bar: Int32) }
    ");
}

#[test]
fn mod_path_in_type() {
    ok("
        fun f(): foo::Foo { foo::Foo() }
        mod foo { @pub class Foo }
    ");

    err(
        "fun f(): bar::Foo { 1i32 }",
        pos(1, 10),
        ErrorMessage::ExpectedModule,
    );

    err(
        "
        fun bar(): Unit {}
        fun f(): bar::Foo = 1i32
    ",
        pos(3, 18),
        ErrorMessage::ExpectedModule,
    );

    err(
        "
        fun f(): foo::bar::Foo = 1i32
        mod foo {}
    ",
        pos(2, 18),
        ErrorMessage::ExpectedModule,
    );
}

#[test]
fn mod_global() {
    ok("
        fun f(): Int32 = foo::x
        mod foo { @pub let x: Int32 = 1i32 }
    ");

    err(
        "
        fun f(): Int32 = foo::x
        mod foo { let x: Int32 = 1i32 }
    ",
        pos(2, 29),
        ErrorMessage::NotAccessible("foo::x".into()),
    );
}

#[test]
fn mod_trait() {
    ok("
        mod foo {
            class Foo
            trait Bar { fun f(x: Foo): Unit }
        }
    ");
}

#[test]
fn mod_impl() {
    ok("
        mod foo {
            class Foo
            trait Bar { fun f(x: Foo): Unit; }
            class AnotherClass
            impl Bar for AnotherClass {
                fun f(x: Foo): Unit {}
            }
        }
    ");
}

#[test]
fn mod_class() {
    ok("
        mod foo {
            class Foo(x: Bar)
            impl Foo {
                fun foo(x: Bar): Unit {}
            }
            class Bar
        }
    ");
}

#[test]
fn mod_class_new() {
    ok("
        mod foo {
            class Foo(x: Bar)
            class Bar
        }
    ");

    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            class Foo(f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            class Foo(@pub f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            @pub class Foo(f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::ClassConstructorNotAccessible("foo::Foo".into()),
    );
}

#[test]
fn mod_value() {
    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            value Foo(f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            value Foo(@pub f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    err(
        "
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            @pub value Foo(f: Int32)
        }
    ",
        pos(2, 33),
        ErrorMessage::ValueConstructorNotAccessible("foo::Foo".into()),
    );

    ok("
        fun f(): Unit { foo::Foo(1i32); () }
        mod foo {
            @pub value Foo(@pub f: Int32)
        }
    ");

    ok("
        fun f(val: foo::Foo): Unit {}
        mod foo {
            @pub value Foo(f: Int32)
        }
    ");

    err(
        "
        fun f(val: foo::Foo): Unit {}
        mod foo {
            value Foo(f: Int32)
        }
    ",
        pos(2, 20),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );
}

#[test]
fn mod_const() {
    ok("
        fun f(): Int32 = foo::x
        mod foo { @pub const x: Int32 = 1i32 }
    ");

    err(
        "
        fun f(): Int32 = foo::x
        mod foo { const x: Int32 = 1i32 }
    ",
        pos(2, 29),
        ErrorMessage::NotAccessible("foo::x".into()),
    );

    ok("
        fun f(): Int32 = foo::bar::x
        mod foo { @pub mod bar { @pub const x: Int32 = 1i32 } }
    ");
}

#[test]
fn mod_enum_value() {
    ok("
        fun f(): Unit { foo::A; () }
        mod foo { @pub enum Foo { A, B } use Foo.A; }
    ");

    err(
        "
        fun f(): Unit { foo::A; () }
        mod foo { enum Foo { A, B } use Foo.A; }
    ",
        pos(2, 28),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    ok("
        fun f(): Unit { foo::bar::A; () }
        mod foo { @pub mod bar { @pub enum Foo { A, B } use Foo.A; } }
    ");

    err(
        "
        fun f(): Unit { foo::bar::A; () }
        mod foo { @pub mod bar { enum Foo { A, B } use Foo.A; } }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::bar::Foo".into()),
    );
}

#[test]
fn mod_enum() {
    err(
        "
        fun f(): Unit { foo::Foo::B; () }
        mod foo { enum Foo { A(Bar), B } class Bar }
    ",
        pos(2, 33),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );

    ok("
        fun f(): Unit { foo::Foo::B; () }
        mod foo { @pub enum Foo { A, B } }
    ");

    ok("
        fun f(): Unit { foo::Foo::A(1i32); () }
        mod foo { @pub enum Foo { A(Int32), B } }
    ");

    err(
        "
        fun f(): Unit { foo::Foo::A(1i32); () }
        mod foo { enum Foo { A(Int32), B } }
    ",
        pos(2, 36),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );
}

#[test]
fn mod_use() {
    ok("
        use foo.bar;
        fun f(): Unit = bar()
        mod foo { @pub fun bar(): Unit {} }
    ");

    ok("
        use foo.bar.baz;
        fun f(): Unit = baz()
        mod foo { @pub mod bar {
            @pub fun baz(): Unit {}
        } }
    ");

    ok("
        use foo.bar as baz;
        fun f(): Unit = baz()
        mod foo { @pub fun bar(): Unit {} }
    ");

    ok("
        use foo.bar;
        fun f(): Int32 = bar
        mod foo { @pub let bar: Int32 = 10i32 }
    ");

    ok("
        use foo.bar.baz;
        fun f(): Int32 = baz
        mod foo { @pub mod bar {
            @pub let baz: Int32 = 10i32
        } }
    ");

    ok("
        use foo.bar;
        fun f(): Int32 = bar
        mod foo { @pub let bar: Int32 = 10i32 }
    ");
}

#[test]
fn mod_use_class() {
    ok("
        use foo.Bar
        fun f(): Unit { Bar(); () }
        mod foo { @pub class Bar }
    ");

    ok("
        use foo.Bar;
        fun f(): Unit {
            Bar()
            Bar::baz()
        }
        mod foo {
            @pub class Bar
            impl Bar {
                @pub @static fun baz(): Unit {}
            }
        }
    ");
}

#[test]
fn mod_use_trait() {
    ok("
        use foo.Bar;
        mod foo { @pub trait Bar{} }
    ");
}

#[test]
fn mod_use_std() {
    ok("
        use std.HashMap
    ");
}

#[test]
fn mod_use_package() {
    ok("
        class Foo
        mod bar {
            use package.Foo
            fun getfoo(): Foo = Foo()
        }
    ");
}

#[test]
fn mod_use_super() {
    ok("
        mod baz {
            class Foo
            mod bar {
                use super.Foo

                fun getfoo(): Foo { Foo() }
            }
        }
    ");

    err("use super.Foo", pos(1, 5), ErrorMessage::NoSuperModule);
}

#[test]
fn mod_use_self() {
    ok("
        use self.bar.Foo
        fun getfoo(): Foo = Foo()
        mod bar { @pub class Foo }
    ");
}

#[test]
fn mod_use_errors() {
    err(
        "
        use foo.bar.baz
        mod foo { @pub mod bar {} }
    ",
        pos(2, 21),
        ErrorMessage::UnknownIdentifierInModule("foo::bar".into(), "baz".into()),
    );

    err(
        "
        use foo.bar
    ",
        pos(2, 13),
        ErrorMessage::UnknownIdentifierInModule("".into(), "foo".into()),
    );

    err(
        "
        use foo.bar
        mod foo {}
    ",
        pos(2, 17),
        ErrorMessage::UnknownIdentifierInModule("foo".into(), "bar".into()),
    );

    err(
        "
        use foo.bar
        fun foo(): Unit {}
    ",
        pos(2, 13),
        ErrorMessage::ExpectedPath,
    );

    err(
        "
        use foo.bar.baz
        @pub mod foo { @pub fun bar(): Unit {} }
    ",
        pos(2, 17),
        ErrorMessage::ExpectedPath,
    );
}

#[test]
fn mod_inside() {
    ok("mod foo { fun f(): Unit { g() } fun g(): Unit {} }");

    ok("mod foo { class Foo fun g(x: Foo): Unit {} }");

    ok("
        fun f(x: foo::Foo): Unit {}
        mod foo { @pub class Foo }
    ");

    err(
        "
        fun f(x: foo::Foo): Unit {}
        mod foo { class Foo }
    ",
        pos(2, 18),
        ErrorMessage::NotAccessible("foo::Foo".into()),
    );
}

#[test]
fn different_fct_call_kinds() {
    ok("fun f(): Unit = g(); fun g(): Unit {}");
    ok("fun f(): Unit = g[Int32](); fun g[T](): Unit {}");
    ok("fun f(g: Array[Int32]): Unit { g(0); () }");
    err(
        "fun f(g: Array[Int32]): Unit { g[Float32](0); () }",
        pos(1, 33),
        ErrorMessage::NoTypeParamsExpected,
    );
    ok("class Foo fun f(): Unit { Foo(); () }");
    errors(
        "fun f(): Unit { 1i32[Int32](); () }",
        &[
            (pos(1, 21), ErrorMessage::NoTypeParamsExpected),
            (
                pos(1, 28),
                ErrorMessage::UnknownMethod("Int32".into(), "get".into(), Vec::new()),
            ),
        ],
    );
    ok("enum Foo { A(Int32), B } fun f(): Unit { Foo::A(1i32); () }");
    ok("enum Foo[T] { A(Int32), B } fun f(): Unit { Foo[Int32]::A(1i32); () }");
    ok("enum Foo[T] { A(Int32), B } fun f(): Unit { Foo::A[Int32](1i32); () }");
    err(
        "enum Foo[T] { A(Int32), B } fun f(): Unit { Foo[Int32]::A[Int32](1i32); () }",
        pos(1, 48),
        ErrorMessage::NoTypeParamsExpected,
    );
    ok("trait MyTrait { @static fun foo(): Unit; } fun f[T: MyTrait](): Unit { T::foo(); () }");
    ok("class Foo impl Foo { fun bar(): Unit {} } fun f(g: Foo): Unit { g.bar(); () }");
    ok("class Foo impl Foo { fun bar[T](): Unit {} } fun f(g: Foo): Unit { g.bar[Int32](); () }");
}

#[test]
fn trait_object_method_call() {
    ok("
        trait Foo { fun bar(): Int32; }
        fun f(x: Foo): Int32 = x.bar()
    ");
}

#[test]
fn trait_object_cast() {
    ok("
        trait Foo { fun bar(): Int32; }
        class Bar
        impl Foo for Bar {
            fun bar(): Int32 = 1i32
        }
        fun f(x: Foo): Int32 = x.bar()
        fun g(): Int32 {
            f(Bar() as Foo)
        }
    ");

    ok("
        trait Foo { fun bar(): Int32; }
        class Bar
        impl Foo for Bar {
            fun bar(): Int32 = 1i32
        }
        fun f(): Foo = Bar() as Foo
    ");

    ok("
        trait Foo { fun bar(): Int32 }
        fun f(x: Foo): Foo {
            let y = x
            y
        }
    ");

    err(
        "
        trait Foo { fun bar(): Int32 }
        class Bar
        fun f(x: Foo): Unit {}
        fun g(): Unit {
            f(Bar() as Foo)
        }
    ",
        pos(6, 21),
        ErrorMessage::TypeNotImplementingTrait("Bar".into(), "Foo".into()),
    );
}

#[test]
fn infer_enum_type() {
    ok("fun f(): Option[Int32] = None");

    ok("
        class X {
            a: Option[Int32],
            b: Option[Int32],
        }

        fun f(x: X): Unit {
            x.a = Some(10i32)
            x.b = None
        }
    ");

    ok("fun f(): Unit {
        var x: Option[Int32] = None; x = Some(10i32)
        var y: Option[Int32] = Some(10i32); y = None
    }");

    ok("fun f(): Option[Int32] = Some(10i32)");
}

#[test]
fn method_call_type_mismatch_with_type_params() {
    err(
        "
        class Foo
        impl Foo {
            fun f(a: String): Unit {}
        }
        fun g[T](foo: Foo, value: T): Unit = foo.f(value)
    ",
        pos(6, 51),
        ErrorMessage::ParamTypesIncompatible("f".into(), vec!["String".into()], vec!["T".into()]),
    );
}

#[test]
fn basic_lambda() {
    ok("fun f(foo: (Int32): Int32): Int32 = foo(1i32)");

    err(
        "fun f(foo: (Int32): Int32): Bool = foo(1i32)",
        pos(1, 39),
        ErrorMessage::ReturnType("Bool".into(), "Int32".into()),
    );

    err(
        "fun f(foo: (Int32, Int32): Int32): Int32 = foo(1i32)",
        pos(1, 47),
        ErrorMessage::LambdaParamTypesIncompatible(
            vec!["Int32".into(), "Int32".into()],
            vec!["Int32".into()],
        ),
    );
}

#[test]
fn lambda_body() {
    ok("fun f(): (Int32): Int32 = |x: Int32|: Int32 { x }");

    ok("fun f(): (Int32): Int32 = |x: Int32|: Int32 { x + 1i32 }");

    err(
        "fun f(): (Int32): Int32 = |x: Int32|: Int32 { false }",
        pos(1, 45),
        ErrorMessage::ReturnType("Int32".into(), "Bool".into()),
    );
}

#[test]
fn lambda_closure() {
    ok("fun f(): Unit {
        let x: Int32 = 10i32;
        ||: Int32 { x };
        ()
    }");

    ok("fun f(x: Int32): Unit {
        ||: Int32 { x };
        ()
    }");

    err(
        "fun f(): Unit {
        ||: Int32 { x }
        let x: Int32 = 10i32;
        ()
    }",
        pos(2, 21),
        ErrorMessage::UnknownIdentifier("x".into()),
    );
}

#[test]
fn internal_class_ctor() {
    err(
        "fun f(): Array[Int32] = Array[Int32]()",
        pos(1, 37),
        ErrorMessage::ClassConstructorNotAccessible("std::collections::Array".into()),
    );
}

#[test]
fn internal_value_ctor() {
    err(
        "fun f(): Unit { Int32(); () }",
        pos(1, 22),
        ErrorMessage::ValueConstructorNotAccessible("std::primitives::Int32".into()),
    );
}

#[test]
fn mutable_param() {
    err(
        "fun f(x: Int64): Unit { x = 10 }",
        pos(1, 27),
        ErrorMessage::LetReassigned,
    );
}

#[test]
fn self_unavailable_in_lambda() {
    err(
        "fun f(): Unit { ||: Unit { self }; () }",
        pos(1, 28),
        ErrorMessage::ThisUnavailable,
    );
}
