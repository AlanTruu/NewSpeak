use crate::lex::{Token, TokenType};

#[derive(Debug)]
pub enum Expr {
    Binary {
        left : Box<Expr>,
        operator : Token,
        right : Box<Expr>
    },
    Unary {
        operator : Token,
        right : Box<Expr>
    },
    Grouping(Box<Expr>),
    Literal(TokenType)
}

pub fn print(e : Expr) -> String {
    match e {
        Expr::Binary {left, operator, right} => format!("( {} {} {} )", operator.lexeme, print(*left), print(*right)),
        Expr::Unary { operator, right } => format!("( {} {} )", operator.lexeme, print(*right)),
        Expr::Grouping(expr) => format!("( group {} )", print(*expr)),
        Expr::Literal(TokenType::Number(n)) => format!(" {} ", n),
        Expr::Literal(TokenType::String(s)) => format!(" {} ",s),
        Expr::Literal(TokenType::Identifier(i)) => format!(" {} ", i),
        Expr::Literal(TokenType::Truth) => format!(" truth "),
        Expr::Literal(TokenType::ThoughtCrime) => format!(" thoughtCrime "),
        Expr::Literal(TokenType::Nil) => format!(" nil "),
        _ => panic!("Unexpected token")
    }
}
