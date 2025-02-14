use std::fmt;
use std::slice::Iter;
use std::sync::Arc;

use crate::interner::Interner;
use crate::interner::Name;
use crate::lexer::position::{Position, Span};
use crate::lexer::token::{FloatSuffix, IntBase, IntSuffix};

pub mod dump;
pub mod visit;

#[derive(Clone, Debug)]
pub struct File {
    pub elements: Vec<Elem>,
}

impl File {
    #[cfg(test)]
    pub fn fct0(&self) -> &Function {
        self.elements[0].to_function().unwrap()
    }

    #[cfg(test)]
    pub fn fct(&self, index: usize) -> &Function {
        self.elements[index].to_function().unwrap()
    }

    #[cfg(test)]
    pub fn cls0(&self) -> &Class {
        self.elements[0].to_class().unwrap()
    }

    #[cfg(test)]
    pub fn cls(&self, index: usize) -> &Class {
        self.elements[index].to_class().unwrap()
    }

    #[cfg(test)]
    pub fn value0(&self) -> &Value {
        self.elements[0].to_value().unwrap()
    }

    #[cfg(test)]
    pub fn union0(&self) -> &Union {
        self.elements[0].to_union().unwrap()
    }

    #[cfg(test)]
    pub fn enum0(&self) -> &Enum {
        self.elements[0].to_enum().unwrap()
    }

    #[cfg(test)]
    pub fn alias0(&self) -> &Alias {
        self.elements[0].to_alias().unwrap()
    }

    #[cfg(test)]
    pub fn module0(&self) -> &Module {
        self.elements[0].to_module().unwrap()
    }

    #[cfg(test)]
    pub fn trait_(&self, index: usize) -> &Trait {
        self.elements[index].to_trait().unwrap()
    }

    #[cfg(test)]
    pub fn trait0(&self) -> &Trait {
        self.elements[0].to_trait().unwrap()
    }

    #[cfg(test)]
    pub fn impl0(&self) -> &Impl {
        self.elements[0].to_impl().unwrap()
    }

    #[cfg(test)]
    pub fn ann0(&self) -> &Annotation {
        self.elements[0].to_annotation().unwrap()
    }

    #[cfg(test)]
    pub fn global0(&self) -> &Global {
        self.elements[0].to_global().unwrap()
    }

    #[cfg(test)]
    pub fn const0(&self) -> &Const {
        self.elements[0].to_const().unwrap()
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct NodeId(pub usize);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub enum Elem {
    Function(Arc<Function>),
    Class(Arc<Class>),
    Value(Arc<Value>),
    Union(Arc<Union>),
    Trait(Arc<Trait>),
    Impl(Arc<Impl>),
    Annotation(Arc<Annotation>),
    Global(Arc<Global>),
    Const(Arc<Const>),
    Enum(Arc<Enum>),
    Alias(Arc<Alias>),
    Module(Arc<Module>),
    Use(Arc<Use>),
}

impl Elem {
    pub fn id(&self) -> NodeId {
        match self {
            &Elem::Function(ref f) => f.id,
            &Elem::Class(ref c) => c.id,
            &Elem::Value(ref s) => s.id,
            &Elem::Union(ref u) => u.id,
            &Elem::Trait(ref t) => t.id,
            &Elem::Impl(ref i) => i.id,
            &Elem::Annotation(ref a) => a.id,
            &Elem::Global(ref g) => g.id,
            &Elem::Const(ref c) => c.id,
            &Elem::Enum(ref e) => e.id,
            &Elem::Alias(ref e) => e.id,
            &Elem::Module(ref e) => e.id,
            &Elem::Use(ref e) => e.id,
        }
    }

    pub fn to_function(&self) -> Option<&Function> {
        match self {
            &Elem::Function(ref fct) => Some(fct),
            _ => None,
        }
    }

    pub fn to_class(&self) -> Option<&Class> {
        match self {
            &Elem::Class(ref class) => Some(class),
            _ => None,
        }
    }

    pub fn to_union(&self) -> Option<&Union> {
        match self {
            &Elem::Union(ref union_) => Some(union_),
            _ => None,
        }
    }

    pub fn to_enum(&self) -> Option<&Enum> {
        match self {
            &Elem::Enum(ref enum_) => Some(enum_),
            _ => None,
        }
    }

    pub fn to_alias(&self) -> Option<&Alias> {
        match self {
            &Elem::Alias(ref alias) => Some(alias),
            _ => None,
        }
    }

    pub fn to_module(&self) -> Option<&Module> {
        match self {
            &Elem::Module(ref module) => Some(module),
            _ => None,
        }
    }

    pub fn to_value(&self) -> Option<&Value> {
        match self {
            &Elem::Value(ref value) => Some(value),
            _ => None,
        }
    }

    pub fn to_trait(&self) -> Option<&Trait> {
        match self {
            &Elem::Trait(ref trait_) => Some(trait_),
            _ => None,
        }
    }

    pub fn to_impl(&self) -> Option<&Impl> {
        match self {
            &Elem::Impl(ref impl_) => Some(impl_),
            _ => None,
        }
    }

    pub fn to_annotation(&self) -> Option<&Annotation> {
        match self {
            &Elem::Annotation(ref annotation) => Some(annotation),
            _ => None,
        }
    }

    pub fn to_global(&self) -> Option<&Global> {
        match self {
            &Elem::Global(ref global) => Some(global),
            _ => None,
        }
    }

    pub fn to_const(&self) -> Option<&Const> {
        match self {
            &Elem::Const(ref konst) => Some(konst),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Global {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub mutable: bool,
    pub data_type: Type,
    pub initializer: Option<Arc<Function>>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub elements: Option<Vec<Elem>>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Use {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub common_path: Vec<UsePathComponent>,
    pub target: UseTargetDescriptor,
}

#[derive(Clone, Debug)]
pub enum UseTargetDescriptor {
    Default,
    As(UseTargetName),
    Group(UseTargetGroup),
}

#[derive(Clone, Debug)]
pub struct UseTargetGroup {
    pub pos: Position,
    pub span: Span,
    pub targets: Vec<Arc<Use>>,
}

#[derive(Clone, Debug)]
pub struct UseTargetName {
    pub pos: Position,
    pub span: Span,
    pub name: Option<Name>,
}

#[derive(Clone, Debug)]
pub struct UsePathComponent {
    pub pos: Position,
    pub span: Span,
    pub value: UsePathComponentValue,
}

#[derive(Clone, Debug)]
pub enum UsePathComponentValue {
    This,
    Super,
    Package,
    Name(Name),
}

#[derive(Clone, Debug)]
pub struct Const {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub data_type: Type,
    pub expr: Box<Expr>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub type_params: Option<Vec<TypeParam>>,
    pub variants: Vec<EnumVariant>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub types: Option<Vec<Type>>,
}

#[derive(Clone, Debug)]
pub struct Alias {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub ty: Type,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Value {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub fields: Vec<ValueField>,
    pub visibility: Visibility,
    pub internal: bool,
    pub type_params: Option<Vec<TypeParam>>,
}

#[derive(Clone, Debug)]
pub struct ValueField {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub data_type: Type,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Union {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub type_params: Option<Vec<TypeParam>>,
    pub variants: Vec<UnionVariant>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
    pub types: Option<Vec<Type>>,
}

#[derive(Clone, Debug)]
pub enum Type {
    This(TypeSelfType),
    Basic(TypeBasicType),
    Tuple(TypeTupleType),
    Lambda(TypeLambdaType),
}

#[derive(Clone, Debug)]
pub struct TypeSelfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeTupleType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub subtypes: Vec<Box<Type>>,
}

#[derive(Clone, Debug)]
pub struct TypeLambdaType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub params: Vec<Box<Type>>,
    pub ret: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct TypeBasicType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub path: Path,
    pub params: Vec<Box<Type>>,
}

impl TypeBasicType {
    pub fn name(&self) -> Name {
        assert_eq!(self.path.names.len(), 1);
        self.path.names.last().cloned().unwrap()
    }
}

impl Type {
    pub fn create_self(id: NodeId, pos: Position, span: Span) -> Type {
        Type::This(TypeSelfType { id, pos, span })
    }

    pub fn create_basic(
        id: NodeId,
        pos: Position,
        span: Span,
        path: Path,
        params: Vec<Box<Type>>,
    ) -> Type {
        Type::Basic(TypeBasicType {
            id,
            pos,
            span,
            path,
            params,
        })
    }

    pub fn create_fct(
        id: NodeId,
        pos: Position,
        span: Span,
        params: Vec<Box<Type>>,
        ret: Box<Type>,
    ) -> Type {
        Type::Lambda(TypeLambdaType {
            id,
            pos,
            span,
            params,
            ret,
        })
    }

    pub fn create_tuple(id: NodeId, pos: Position, span: Span, subtypes: Vec<Box<Type>>) -> Type {
        Type::Tuple(TypeTupleType {
            id,
            pos,
            span,
            subtypes,
        })
    }

    pub fn to_basic(&self) -> Option<&TypeBasicType> {
        match *self {
            Type::Basic(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn to_tuple(&self) -> Option<&TypeTupleType> {
        match *self {
            Type::Tuple(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn to_fct(&self) -> Option<&TypeLambdaType> {
        match *self {
            Type::Lambda(ref val) => Some(val),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn is_unit(&self) -> bool {
        match self {
            &Type::Tuple(ref val) if val.subtypes.len() == 0 => true,
            _ => false,
        }
    }

    pub fn to_string(&self, interner: &Interner) -> String {
        match *self {
            Type::This(_) => "Self".into(),
            Type::Basic(ref val) => format!("{}", *interner.str(val.name())),

            Type::Tuple(ref val) => {
                let types: Vec<String> =
                    val.subtypes.iter().map(|t| t.to_string(interner)).collect();

                format!("({})", types.join(", "))
            }

            Type::Lambda(ref val) => {
                let types: Vec<String> = val.params.iter().map(|t| t.to_string(interner)).collect();
                let ret = val.ret.to_string(interner);

                format!("({}) -> {}", types.join(", "), ret)
            }
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Type::This(ref val) => val.pos,
            Type::Basic(ref val) => val.pos,
            Type::Tuple(ref val) => val.pos,
            Type::Lambda(ref val) => val.pos,
        }
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Type::This(ref val) => val.id,
            Type::Basic(ref val) => val.id,
            Type::Tuple(ref val) => val.id,
            Type::Lambda(ref val) => val.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Impl {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub type_params: Option<Vec<TypeParam>>,
    pub trait_type: Option<Type>,
    pub extended_type: Type,
    pub methods: Vec<Arc<Function>>,
}

#[derive(Clone, Debug)]
pub struct Trait {
    pub id: NodeId,
    pub name: Name,
    pub type_params: Option<Vec<TypeParam>>,
    pub pos: Position,
    pub span: Span,
    pub methods: Vec<Arc<Function>>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Class {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub internal: bool,
    pub visibility: Visibility,

    pub fields: Vec<Field>,
    pub type_params: Option<Vec<TypeParam>>,
}

#[derive(Clone, Debug)]
pub struct Annotation {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub annotation_usages: AnnotationUsages,
    pub internal: Option<Modifier>,

    pub type_params: Option<Vec<TypeParam>>,
    pub term_params: Option<Vec<AnnotationParam>>,
}

#[derive(Clone, Debug)]
pub struct TypeParam {
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub bounds: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct ConstructorParam {
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub data_type: Type,
    pub field: bool,
    pub mutable: bool,
    pub variadic: bool,
}

#[derive(Clone, Debug)]
pub struct AnnotationParam {
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub data_type: Type,
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub name: Option<Name>,
    pub pos: Position,
    pub span: Span,
    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub id: NodeId,
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub data_type: Type,
    pub primary_ctor: bool,
    pub expr: Option<Box<Expr>>,
    pub mutable: bool,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub enum FunctionKind {
    Function,
    Lambda,
}

impl FunctionKind {
    pub fn is_lambda(&self) -> bool {
        match self {
            &FunctionKind::Lambda => true,
            &FunctionKind::Function => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: NodeId,
    pub kind: FunctionKind,
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub method: bool,
    pub is_optimize_immediately: bool,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_test: bool,
    pub internal: bool,
    pub is_constructor: bool,

    pub params: Vec<Param>,
    pub is_nullary: bool,

    pub return_type: Option<Type>,
    pub block: Option<Box<ExprBlockType>>,
    pub type_params: Option<Vec<TypeParam>>,
}

impl Function {
    pub fn block(&self) -> &ExprBlockType {
        self.block.as_ref().unwrap()
    }
}

// remove in next step
#[derive(Clone, Debug)]
pub struct Modifiers(Vec<ModifierElement>);

// remove in next step
impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers(Vec::new())
    }

    pub fn contains(&self, modifier: Modifier) -> bool {
        self.0.iter().find(|el| el.value == modifier).is_some()
    }

    pub fn add(&mut self, modifier: Modifier, pos: Position, span: Span) {
        self.0.push(ModifierElement {
            value: modifier,
            pos,
            span,
        });
    }

    pub fn iter(&self) -> Iter<ModifierElement> {
        self.0.iter()
    }
}

// remove in next step
#[derive(Clone, Debug)]
pub struct ModifierElement {
    pub value: Modifier,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct AnnotationUsages(Vec<AnnotationUsage>);

impl AnnotationUsages {
    pub fn new() -> AnnotationUsages {
        AnnotationUsages(Vec::new())
    }

    pub fn contains(&self, name: Name) -> bool {
        self.0.iter().find(|el| el.name == name).is_some()
    }

    pub fn add(&mut self, annotation_usage: AnnotationUsage) {
        self.0.push(annotation_usage);
    }

    pub fn iter(&self) -> Iter<AnnotationUsage> {
        self.0.iter()
    }
}

#[derive(Clone, Debug)]
pub struct AnnotationUsage {
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub type_args: Vec<Type>,
    pub term_args: Vec<Box<Expr>>,
}

// rename to InternalAnnotation in next step
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Modifier {
    Internal,
    Pub,
    Static,
    Test,
    OptimizeImmediately,
}

impl Modifier {
    pub fn find(name: &str) -> Option<Modifier> {
        match name {
            "internal" => Some(Modifier::Internal),
            "pub" => Some(Modifier::Pub),
            "static" => Some(Modifier::Static),
            "test" => Some(Modifier::Test),
            "optimizeImmediately" => Some(Modifier::OptimizeImmediately),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match *self {
            Modifier::Internal => "internal",
            Modifier::Pub => "pub",
            Modifier::Static => "static",
            Modifier::Test => "test",
            Modifier::OptimizeImmediately => "optimizeImmediately",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub id: NodeId,
    pub idx: u32,
    pub name: Name,
    pub pos: Position,
    pub span: Span,
    pub data_type: Type,
    pub variadic: bool,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let(StmtLetType),
    While(StmtWhileType),
    Expr(StmtExprType),
    Return(StmtReturnType),
    For(StmtForType),
}

impl Stmt {
    pub fn create_let(
        id: NodeId,
        pos: Position,
        span: Span,
        pattern: Box<LetPattern>,
        mutable: bool,
        data_type: Option<Type>,
        expr: Option<Box<Expr>>,
    ) -> Stmt {
        Stmt::Let(StmtLetType {
            id,
            pos,
            span,

            pattern,
            mutable,
            data_type,
            expr,
        })
    }

    pub fn create_for(
        id: NodeId,
        pos: Position,
        span: Span,
        pattern: Box<LetPattern>,
        expr: Box<Expr>,
        block: Box<Stmt>,
    ) -> Stmt {
        Stmt::For(StmtForType {
            id,
            pos,
            span,

            pattern,
            expr,
            block,
        })
    }

    pub fn create_while(
        id: NodeId,
        pos: Position,
        span: Span,
        cond: Box<Expr>,
        block: Box<Stmt>,
    ) -> Stmt {
        Stmt::While(StmtWhileType {
            id,
            pos,
            span,

            cond,
            block,
        })
    }

    pub fn create_expr(id: NodeId, pos: Position, span: Span, expr: Box<Expr>) -> Stmt {
        Stmt::Expr(StmtExprType {
            id,
            pos,
            span,

            expr,
        })
    }

    pub fn create_return(id: NodeId, pos: Position, span: Span, expr: Option<Box<Expr>>) -> Stmt {
        Stmt::Return(StmtReturnType {
            id,
            pos,
            span,

            expr,
        })
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Stmt::Let(ref stmt) => stmt.id,
            Stmt::While(ref stmt) => stmt.id,
            Stmt::For(ref stmt) => stmt.id,
            Stmt::Expr(ref stmt) => stmt.id,
            Stmt::Return(ref stmt) => stmt.id,
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Stmt::Let(ref stmt) => stmt.pos,
            Stmt::While(ref stmt) => stmt.pos,
            Stmt::For(ref stmt) => stmt.pos,
            Stmt::Expr(ref stmt) => stmt.pos,
            Stmt::Return(ref stmt) => stmt.pos,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            Stmt::Let(ref stmt) => stmt.span,
            Stmt::While(ref stmt) => stmt.span,
            Stmt::For(ref stmt) => stmt.span,
            Stmt::Expr(ref stmt) => stmt.span,
            Stmt::Return(ref stmt) => stmt.span,
        }
    }

    pub fn to_let(&self) -> Option<&StmtLetType> {
        match *self {
            Stmt::Let(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_let(&self) -> bool {
        match *self {
            Stmt::Let(_) => true,
            _ => false,
        }
    }

    pub fn to_while(&self) -> Option<&StmtWhileType> {
        match *self {
            Stmt::While(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_while(&self) -> bool {
        match *self {
            Stmt::While(_) => true,
            _ => false,
        }
    }

    pub fn to_for(&self) -> Option<&StmtForType> {
        match *self {
            Stmt::For(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_for(&self) -> bool {
        match *self {
            Stmt::For(_) => true,
            _ => false,
        }
    }

    pub fn to_expr(&self) -> Option<&StmtExprType> {
        match *self {
            Stmt::Expr(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_expr(&self) -> bool {
        match *self {
            Stmt::Expr(_) => true,
            _ => false,
        }
    }

    pub fn to_return(&self) -> Option<&StmtReturnType> {
        match *self {
            Stmt::Return(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_return(&self) -> bool {
        match *self {
            Stmt::Return(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StmtLetType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub pattern: Box<LetPattern>,
    pub mutable: bool,

    pub data_type: Option<Type>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub enum LetPattern {
    Ident(LetIdentType),
    Tuple(LetTupleType),
    Underscore(LetUnderscoreType),
}

impl LetPattern {
    pub fn is_ident(&self) -> bool {
        match self {
            LetPattern::Ident(_) => true,
            _ => false,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match self {
            LetPattern::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn is_underscore(&self) -> bool {
        match self {
            LetPattern::Underscore(_) => true,
            _ => false,
        }
    }

    pub fn to_name(&self) -> Option<Name> {
        match self {
            LetPattern::Ident(ref ident) => Some(ident.name),
            _ => None,
        }
    }

    pub fn to_ident(&self) -> Option<&LetIdentType> {
        match self {
            LetPattern::Ident(ref ident) => Some(ident),
            _ => None,
        }
    }

    pub fn to_tuple(&self) -> Option<&LetTupleType> {
        match self {
            LetPattern::Tuple(ref tuple) => Some(tuple),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LetUnderscoreType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LetIdentType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Name,
}

#[derive(Clone, Debug)]
pub struct LetTupleType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub parts: Vec<Box<LetPattern>>,
}

#[derive(Clone, Debug)]
pub struct StmtForType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub pattern: Box<LetPattern>,
    pub expr: Box<Expr>,
    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct StmtWhileType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub cond: Box<Expr>,
    pub block: Box<Stmt>,
}

#[derive(Clone, Debug)]
pub struct StmtExprType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct StmtReturnType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Option<Box<Expr>>,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum UnOp {
    Plus,
    Neg,
}

impl UnOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UnOp::Plus => "+",
            UnOp::Neg => "-",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot,
}

impl CmpOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
            CmpOp::Is => "===",
            CmpOp::IsNot => "!==",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BinOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Cmp(CmpOp),
    Or,
    And,
    BitOr,
    BitAnd,
    BitXor,
}

impl BinOp {
    pub fn as_str(&self) -> &'static str {
        match *self {
            BinOp::Assign => "=",
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Cmp(op) => op.as_str(),
            BinOp::Or => "||",
            BinOp::And => "&&",
            BinOp::BitOr => "|",
            BinOp::BitAnd => "&",
            BinOp::BitXor => "^",
        }
    }

    pub fn is_any_assign(&self) -> bool {
        match *self {
            BinOp::Assign => true,
            _ => false,
        }
    }

    pub fn is_compare(&self) -> bool {
        match *self {
            BinOp::Cmp(cmp) if cmp != CmpOp::Is && cmp != CmpOp::IsNot => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Un(ExprUnType),
    Bin(ExprBinType),
    LitChar(ExprLitCharType),
    LitInt(ExprLitIntType),
    LitFloat(ExprLitFloatType),
    LitStr(ExprLitStrType),
    Template(ExprTemplateType),
    LitBool(ExprLitBoolType),
    Ident(ExprIdentType),
    Call(ExprCallType),
    TypeParam(ExprTypeParamType),
    Path(ExprPathType),
    Dot(ExprDotType),
    This(ExprSelfType),
    Conv(ExprConvType),
    Lambda(Arc<Function>),
    Block(ExprBlockType),
    If(ExprIfType),
    Tuple(ExprTupleType),
    Paren(ExprParenType),
}

impl Expr {
    pub fn create_block(
        id: NodeId,
        pos: Position,
        span: Span,
        stmts: Vec<Box<Stmt>>,
        expr: Option<Box<Expr>>,
    ) -> Expr {
        Expr::Block(ExprBlockType {
            id,
            pos,
            span,

            stmts,
            expr,
        })
    }

    pub fn create_if(
        id: NodeId,
        pos: Position,
        span: Span,
        cond: Box<StmtLetType>,
        cases: Vec<IfCaseType>,
        else_block: Option<Box<Expr>>,
    ) -> Expr {
        Expr::If(ExprIfType {
            id,
            pos,
            span,
            cond,
            cases,
            else_block,
        })
    }

    pub fn create_un(id: NodeId, pos: Position, span: Span, op: UnOp, opnd: Box<Expr>) -> Expr {
        Expr::Un(ExprUnType {
            id,
            pos,
            span,

            op,
            opnd,
        })
    }

    pub fn create_bin(
        id: NodeId,
        pos: Position,
        span: Span,
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    ) -> Expr {
        Expr::Bin(ExprBinType {
            id,
            pos,
            span,

            op,
            initializer: false,
            lhs,
            rhs,
        })
    }

    pub fn create_conv(
        id: NodeId,
        pos: Position,
        span: Span,
        object: Box<Expr>,
        data_type: Box<Type>,
    ) -> Expr {
        Expr::Conv(ExprConvType {
            id,
            pos,
            span,

            object,
            data_type,
        })
    }

    pub fn create_lit_char(id: NodeId, pos: Position, span: Span, value: char) -> Expr {
        Expr::LitChar(ExprLitCharType {
            id,
            pos,
            span,

            value,
        })
    }

    pub fn create_lit_int(
        id: NodeId,
        pos: Position,
        span: Span,
        value: u64,
        base: IntBase,
        suffix: IntSuffix,
    ) -> Expr {
        Expr::LitInt(ExprLitIntType {
            id,
            pos,
            span,

            value,
            base,
            suffix,
        })
    }

    pub fn create_lit_float(
        id: NodeId,
        pos: Position,
        span: Span,
        value: f64,
        suffix: FloatSuffix,
    ) -> Expr {
        Expr::LitFloat(ExprLitFloatType {
            id,
            pos,
            span,
            value,
            suffix,
        })
    }

    pub fn create_lit_str(id: NodeId, pos: Position, span: Span, value: String) -> Expr {
        Expr::LitStr(ExprLitStrType {
            id,
            pos,
            span,

            value,
        })
    }

    pub fn create_template(id: NodeId, pos: Position, span: Span, parts: Vec<Box<Expr>>) -> Expr {
        Expr::Template(ExprTemplateType {
            id,
            pos,
            span,

            parts,
        })
    }

    pub fn create_lit_bool(id: NodeId, pos: Position, span: Span, value: bool) -> Expr {
        Expr::LitBool(ExprLitBoolType {
            id,
            pos,
            span,

            value,
        })
    }

    pub fn create_this(id: NodeId, pos: Position, span: Span) -> Expr {
        Expr::This(ExprSelfType { id, pos, span })
    }

    pub fn create_ident(
        id: NodeId,
        pos: Position,
        span: Span,
        name: Name,
        type_params: Option<Vec<Type>>,
    ) -> Expr {
        Expr::Ident(ExprIdentType {
            id,
            pos,
            span,

            name,
            type_params,
        })
    }

    pub fn create_paren(id: NodeId, pos: Position, span: Span, expr: Box<Expr>) -> Expr {
        Expr::Paren(ExprParenType {
            id,
            pos,
            span,

            expr,
        })
    }

    pub fn create_call(
        id: NodeId,
        pos: Position,
        span: Span,
        callee: Box<Expr>,
        args: Vec<Box<Arg>>,
    ) -> Expr {
        Expr::Call(ExprCallType {
            id,
            pos,
            span,

            callee,
            args,
        })
    }

    pub fn create_type_param(
        id: NodeId,
        pos: Position,
        span: Span,
        callee: Box<Expr>,
        args: Vec<Type>,
    ) -> Expr {
        Expr::TypeParam(ExprTypeParamType {
            id,
            pos,
            span,

            callee,
            args,
        })
    }

    pub fn create_path(
        id: NodeId,
        pos: Position,
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    ) -> Expr {
        Expr::Path(ExprPathType {
            id,
            pos,
            span,

            lhs,
            rhs,
        })
    }

    pub fn create_dot(
        id: NodeId,
        pos: Position,
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    ) -> Expr {
        Expr::Dot(ExprDotType {
            id,
            pos,
            span,

            lhs,
            rhs,
        })
    }

    pub fn create_lambda(fct: Arc<Function>) -> Expr {
        Expr::Lambda(fct)
    }

    pub fn create_tuple(id: NodeId, pos: Position, span: Span, values: Vec<Box<Expr>>) -> Expr {
        Expr::Tuple(ExprTupleType {
            id,
            pos,
            span,
            values,
        })
    }

    pub fn to_un(&self) -> Option<&ExprUnType> {
        match *self {
            Expr::Un(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_un(&self) -> bool {
        match *self {
            Expr::Un(_) => true,
            _ => false,
        }
    }

    pub fn to_bin(&self) -> Option<&ExprBinType> {
        match *self {
            Expr::Bin(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_bin(&self) -> bool {
        match *self {
            Expr::Bin(_) => true,
            _ => false,
        }
    }

    pub fn to_paren(&self) -> Option<&ExprParenType> {
        match *self {
            Expr::Paren(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_paren(&self) -> bool {
        match *self {
            Expr::Paren(_) => true,
            _ => false,
        }
    }

    pub fn to_ident(&self) -> Option<&ExprIdentType> {
        match *self {
            Expr::Ident(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_ident(&self) -> bool {
        match *self {
            Expr::Ident(_) => true,
            _ => false,
        }
    }

    pub fn to_call(&self) -> Option<&ExprCallType> {
        match *self {
            Expr::Call(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_call(&self) -> bool {
        match *self {
            Expr::Call(_) => true,
            _ => false,
        }
    }

    pub fn to_path(&self) -> Option<&ExprPathType> {
        match *self {
            Expr::Path(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_path(&self) -> bool {
        match *self {
            Expr::Path(_) => true,
            _ => false,
        }
    }

    pub fn to_type_param(&self) -> Option<&ExprTypeParamType> {
        match *self {
            Expr::TypeParam(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_type_param(&self) -> bool {
        match *self {
            Expr::TypeParam(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_char(&self) -> Option<&ExprLitCharType> {
        match *self {
            Expr::LitChar(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_char(&self) -> bool {
        match *self {
            Expr::LitChar(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_int(&self) -> Option<&ExprLitIntType> {
        match *self {
            Expr::LitInt(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_int(&self) -> bool {
        match *self {
            Expr::LitInt(_) => true,
            _ => false,
        }
    }

    pub fn to_template(&self) -> Option<&ExprTemplateType> {
        match *self {
            Expr::Template(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_template(&self) -> bool {
        match *self {
            Expr::Template(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_float(&self) -> Option<&ExprLitFloatType> {
        match *self {
            Expr::LitFloat(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_float(&self) -> bool {
        match *self {
            Expr::LitFloat(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_str(&self) -> Option<&ExprLitStrType> {
        match *self {
            Expr::LitStr(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_str(&self) -> bool {
        match *self {
            Expr::LitStr(_) => true,
            _ => false,
        }
    }

    pub fn to_lit_bool(&self) -> Option<&ExprLitBoolType> {
        match *self {
            Expr::LitBool(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_lit_bool(&self) -> bool {
        match *self {
            Expr::LitBool(_) => true,
            _ => false,
        }
    }

    pub fn is_lit_true(&self) -> bool {
        match *self {
            Expr::LitBool(ref lit) if lit.value => true,
            _ => false,
        }
    }

    pub fn to_dot(&self) -> Option<&ExprDotType> {
        match *self {
            Expr::Dot(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_dot(&self) -> bool {
        match *self {
            Expr::Dot(_) => true,
            _ => false,
        }
    }

    pub fn is_this(&self) -> bool {
        match *self {
            Expr::This(_) => true,
            _ => false,
        }
    }

    pub fn to_conv(&self) -> Option<&ExprConvType> {
        match *self {
            Expr::Conv(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_conv(&self) -> bool {
        match *self {
            Expr::Conv(_) => true,
            _ => false,
        }
    }

    pub fn to_lambda(&self) -> Option<Arc<Function>> {
        match *self {
            Expr::Lambda(ref val) => Some(val.clone()),
            _ => None,
        }
    }

    pub fn is_lambda(&self) -> bool {
        match self {
            &Expr::Lambda(_) => true,
            _ => false,
        }
    }

    pub fn to_tuple(&self) -> Option<&ExprTupleType> {
        match *self {
            Expr::Tuple(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match *self {
            Expr::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn to_block(&self) -> Option<&ExprBlockType> {
        match *self {
            Expr::Block(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            &Expr::Block(_) => true,
            _ => false,
        }
    }

    pub fn to_if(&self) -> Option<&ExprIfType> {
        match *self {
            Expr::If(ref val) => Some(val),
            _ => None,
        }
    }

    pub fn is_if(&self) -> bool {
        match *self {
            Expr::If(_) => true,
            _ => false,
        }
    }

    pub fn pos(&self) -> Position {
        match *self {
            Expr::Un(ref val) => val.pos,
            Expr::Bin(ref val) => val.pos,
            Expr::LitChar(ref val) => val.pos,
            Expr::LitInt(ref val) => val.pos,
            Expr::LitFloat(ref val) => val.pos,
            Expr::LitStr(ref val) => val.pos,
            Expr::Template(ref val) => val.pos,
            Expr::LitBool(ref val) => val.pos,
            Expr::Ident(ref val) => val.pos,
            Expr::Call(ref val) => val.pos,
            Expr::TypeParam(ref val) => val.pos,
            Expr::Path(ref val) => val.pos,
            Expr::Dot(ref val) => val.pos,
            Expr::This(ref val) => val.pos,
            Expr::Conv(ref val) => val.pos,
            Expr::Lambda(ref val) => val.pos,
            Expr::Block(ref val) => val.pos,
            Expr::If(ref val) => val.pos,
            Expr::Tuple(ref val) => val.pos,
            Expr::Paren(ref val) => val.pos,
        }
    }

    pub fn span(&self) -> Span {
        match *self {
            Expr::Un(ref val) => val.span,
            Expr::Bin(ref val) => val.span,
            Expr::LitChar(ref val) => val.span,
            Expr::LitInt(ref val) => val.span,
            Expr::LitFloat(ref val) => val.span,
            Expr::LitStr(ref val) => val.span,
            Expr::Template(ref val) => val.span,
            Expr::LitBool(ref val) => val.span,
            Expr::Ident(ref val) => val.span,
            Expr::Call(ref val) => val.span,
            Expr::TypeParam(ref val) => val.span,
            Expr::Path(ref val) => val.span,
            Expr::Dot(ref val) => val.span,
            Expr::This(ref val) => val.span,
            Expr::Conv(ref val) => val.span,
            Expr::Lambda(ref val) => val.span,
            Expr::Block(ref val) => val.span,
            Expr::If(ref val) => val.span,
            Expr::Tuple(ref val) => val.span,
            Expr::Paren(ref val) => val.span,
        }
    }

    pub fn id(&self) -> NodeId {
        match *self {
            Expr::Un(ref val) => val.id,
            Expr::Bin(ref val) => val.id,
            Expr::LitChar(ref val) => val.id,
            Expr::LitInt(ref val) => val.id,
            Expr::LitFloat(ref val) => val.id,
            Expr::LitStr(ref val) => val.id,
            Expr::Template(ref val) => val.id,
            Expr::LitBool(ref val) => val.id,
            Expr::Ident(ref val) => val.id,
            Expr::Call(ref val) => val.id,
            Expr::TypeParam(ref val) => val.id,
            Expr::Path(ref val) => val.id,
            Expr::Dot(ref val) => val.id,
            Expr::This(ref val) => val.id,
            Expr::Conv(ref val) => val.id,
            Expr::Lambda(ref val) => val.id,
            Expr::Block(ref val) => val.id,
            Expr::If(ref val) => val.id,
            Expr::Tuple(ref val) => val.id,
            Expr::Paren(ref val) => val.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Branch {
    pub cond_tail: Option<Box<Expr>>,
    pub then_block: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprTupleType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub values: Vec<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct ExprConvType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub object: Box<Expr>,
    pub data_type: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct ExprUnType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub op: UnOp,
    pub opnd: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprBinType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub op: BinOp,
    pub initializer: bool,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprLitCharType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: char,
}

#[derive(Clone, Debug)]
pub struct ExprLitIntType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: u64,
    pub base: IntBase,
    pub suffix: IntSuffix,
}

#[derive(Clone, Debug)]
pub struct ExprLitFloatType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: f64,
    pub suffix: FloatSuffix,
}

#[derive(Clone, Debug)]
pub struct ExprLitStrType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ExprTemplateType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub parts: Vec<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct ExprLitBoolType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub value: bool,
}

#[derive(Clone, Debug)]
pub struct ExprBlockType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub stmts: Vec<Box<Stmt>>,
    pub expr: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct ExprSelfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprIdentType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub name: Name,
    pub type_params: Option<Vec<Type>>,
}

#[derive(Clone, Debug)]
pub struct ExprCallType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub callee: Box<Expr>,
    pub args: Vec<Box<Arg>>,
}

impl ExprCallType {
    pub fn object(&self) -> Option<&Expr> {
        if let Some(type_param) = self.callee.to_type_param() {
            if let Some(dot) = type_param.callee.to_dot() {
                Some(&dot.lhs)
            } else {
                None
            }
        } else if let Some(dot) = self.callee.to_dot() {
            Some(&dot.lhs)
        } else {
            None
        }
    }

    pub fn object_or_callee(&self) -> &Expr {
        self.object().unwrap_or(&self.callee)
    }

    pub fn arg_names(&self) -> Vec<&Option<Name>> {
        self.args.iter().map(|arg| &arg.name).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ExprParenType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub expr: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprIfType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub cond: Box<StmtLetType>,
    pub cases: Vec<IfCaseType>,
    pub else_block: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
pub struct IfCaseType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub data: IfCaseData,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug)]
pub enum IfCaseData {
    Simple,
    Continuation(Box<Expr>),
    Patterns(Vec<IfPattern>),
}

#[derive(Clone, Debug)]
pub struct IfPattern {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub path: Path,
    pub params: Option<Vec<IfPatternParam>>,
}

#[derive(Clone, Debug)]
pub struct IfPatternParam {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub name: Option<Name>,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,
    pub names: Vec<Name>,
}

#[derive(Clone, Debug)]
pub struct ExprTypeParamType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub callee: Box<Expr>,
    pub args: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct ExprPathType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct ExprDotType {
    pub id: NodeId,
    pub pos: Position,
    pub span: Span,

    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Copy, Clone, Debug)]
pub enum Visibility {
    Public,
    Default,
}

impl Visibility {
    pub fn from_modifiers(modifiers: &Modifiers) -> Visibility {
        if modifiers.contains(Modifier::Pub) {
            Visibility::Public
        } else {
            Visibility::Default
        }
    }
    pub fn is_public(self) -> bool {
        match self {
            Visibility::Public => true,
            Visibility::Default => false,
        }
    }

    pub fn is_default(self) -> bool {
        match self {
            Visibility::Public => false,
            Visibility::Default => true,
        }
    }
}
