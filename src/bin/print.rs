// use NewSpeak::{grammar::Expr, lex::{Token, TokenType}};
// use NewSpeak::grammar::print;
use NewSpeak::{lex::{Token, TokenType}, syntax::{self, Binary, Unary, Grouping, Literal, Printer}};

fn main() {
    // let expression = Expr::Binary { left: Box::new(Expr::Unary { operator: Token {t: TokenType::Minus , lexeme : "-".to_string(), line: 1}, right: Box::new(Expr::Literal(TokenType::Number(123.0))) }), operator: Token { t: TokenType::Star, lexeme: "*".to_string(), line: 1 }, right: Box::new(Expr::Grouping(Box::new(Expr::Literal(TokenType::Number(45.66))))) };

    // let str = print(expression);

    let expression = Binary {
        left : Unary {
            operator : Token::new(TokenType::Minus, "-".to_string(), 1),
            right : Literal {
                literal : TokenType::Number(123.0)
            }
        },
        operator : Token::new(TokenType::Star, "*".to_string(), 1),
        right : Grouping {
            expression : Literal {literal : TokenType::Number(45.67)}
        }

    };

    let p = Printer {};

    let res = p.print(&expression);
    println!("{}", res)
}
