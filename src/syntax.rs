use crate::lex::{Token, TokenType};

// pub trait Expr {
//     fn accept<V : Visitor>(&self, visitor : &V) ->  <V as Visitor>::Output;
// }
// 
// 
// pub trait Visitor {
//     type Output;
// 
//     fn visit_binary<T : Expr, R : Expr>(&self, binary : &Binary<T,R>) -> Self::Output;
//     fn visit_unary<R : Expr>(&self, unary : &Unary<R>) -> Self::Output;
//     fn visit_grouping<T : Expr>(&self, grouping : &Grouping<T>) -> Self::Output;
//     fn visit_literal(&self, literal : &Literal) -> Self::Output;
// }
// 
// pub struct Binary<T : Expr, R : Expr> {
//     pub left : T,
//     pub operator : Token,
//     pub right : R
// }
// 
// impl<T : Expr, R : Expr> Binary<T , R> {
//     pub fn new(left : T, operator : Token, right : R) -> Self {
//         Self {
//             left,
//             operator,
//             right
//         }
//     }
// }
// 
// impl<T: Expr, R: Expr> Expr for Binary<T,R> {
//     fn accept<V : Visitor>(&self, visitor : &V) -> <V as Visitor>::Output {
//         visitor.visit_binary(self)
//     }
// }
// 
// pub struct Unary<R : Expr> {
//     pub operator : Token,
//     pub right : R
// }
// 
// impl<R: Expr> Unary<R> {
//     pub fn new(operator : Token, right : R) -> Self {
//         Self {
//             operator,
//             right
//         }
//     }
// }
// 
// impl<R : Expr> Expr for Unary<R> {
//     fn accept<V : Visitor>(&self, visitor : &V) -> <V as Visitor>::Output {
//         visitor.visit_unary(self)
//     }
// }
// 
// pub struct Grouping<T : Expr> {
//     pub expression : T
// }
// 
// impl<T: Expr> Grouping<T> {
//     pub fn new(expression : T) -> Self {
//         Self {
//             expression
//         }
//     }
// }
// 
// impl<T : Expr> Expr for Grouping<T> {
//     fn accept<V : Visitor>(&self, visitor : &V) -> <V as Visitor>::Output {
//         visitor.visit_grouping(self)
//     }
// }
// 
// pub struct Literal {
//     pub literal : TokenType
// }
// 
// impl Literal {
//     pub fn new(literal : TokenType) -> Self {
//         match literal {
//             TokenType::Identifier(id) => Self {literal: TokenType::Identifier(id)},
//             TokenType::String(str) => Self {literal : TokenType::String(str)},
//             TokenType::Number(n) => Self {literal},
//             TokenType::Truth => Self {literal},
//             TokenType::ThoughtCrime => Self {literal},
//             TokenType::Nil => Self {literal},
//             _ => panic!("Not a literal")
//         }
//     }
// }
// 
// impl Expr for Literal {
//     fn accept<V : Visitor>(&self, visitor : &V) ->  <V as Visitor>::Output {
//         visitor.visit_literal(self)
//     }
// }
// 
// pub struct Printer {
// 
// }
// 
// impl Printer {
//     pub fn print(&self, expr : &impl Expr) -> String {
//         expr.accept(self)
//     }
// }
// 
// impl Visitor for Printer {
//     type Output = String;
// 
//     fn visit_binary<T : Expr, R : Expr>(&self, binary : &Binary<T,R>) -> Self::Output {
//         format!("({} {} {})", binary.operator.lexeme, binary.left.accept(self), binary.right.accept(self))
//     }
//     fn visit_unary<R : Expr>(&self, unary : &Unary<R>) -> Self::Output {
//         format!("({} {})", unary.operator.lexeme, unary.right.accept(self))
//     }
//     fn visit_grouping<T : Expr>(&self, grouping : &Grouping<T>) -> Self::Output {
//         format!("(group {})", grouping.expression.accept(self))
//     }
//     fn visit_literal(&self, literal : &Literal) -> Self::Output {
//         match &literal.literal {
//             TokenType::Number(n) => format!("{}", n),
//             TokenType::String(s) => format!("{}", s),
//             TokenType::Identifier(i) => format!("{}", i),
//             _ => panic!("Invalid literal")
//         }
//     }
// }
