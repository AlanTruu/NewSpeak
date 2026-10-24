// use crate::{lex::{self, Token, TokenType}, syntax::{Binary, Expr}};
// use std::iter::Peekable;
// 
// struct Parser {
//     tokens :  Peekable<std::vec::IntoIter<lex::Token>>,
//     current : usize
// }
// 
// impl Parser {
//     fn new(tokens : Vec<Token>) -> Self {
//         Self {
//             tokens : tokens.into_iter().peekable(),
//             current : 0
//         }
//     }
// 
//     fn expression(&self) -> impl Expr {
//         self.equality()
//     }
// 
//     fn equality(&self) -> impl Expr {
//         let mut expr = self.comparison();
// 
//         while let Some(tok) = self.tokens.peek() {
//             let operator = match tok.t {
//                 TokenType::BangEqual | TokenType::EqualEqual => self.tokens.next().unwrap(),
//                 _ => break
//             };
// 
//             let right = self.comparison();
// 
//             expr = Binary::new(expr, operator, right);
//         }
// 
//         expr
//     }
// 
//     fn comparison(&self) -> impl Expr {
//         
//     }
// 
// }
