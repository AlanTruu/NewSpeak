use std::any::Any;

use crate::{grammar::Expr::Literal, interpreter_utils::NewError::{self, Runtime}, lex::{Token, TokenType}};

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
    Literal(Value)
}

#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Identifier(String)
}

pub fn print(e : Expr) -> String {
    match e {
        Expr::Binary {left, operator, right} => format!("( {} {} {} )", operator.lexeme, print(*left), print(*right)),
        Expr::Unary { operator, right } => format!("( {} {} )", operator.lexeme, print(*right)),
        Expr::Grouping(expr) => format!("( group {} )", print(*expr)),
        Expr::Literal(Value::Number(n)) => format!(" {} ", n),
        Expr::Literal(Value::String(s)) => format!(" {} ", s),
        Expr::Literal(Value::Identifier(i)) => format!(" {} ", i),
        Expr::Literal(Value::Bool(true)) => format!(" truth "),
        Expr::Literal(Value::Bool(false)) => format!(" thoughtCrime "),
        Expr::Literal(Value::Nil) => format!(" nil "),
        _ => panic!("Unexpected token")
    }
}

pub fn evaluate(e : &Expr) -> Result<Value, NewError> {
    match e {
        Expr::Literal(Value::Nil) => Ok(Value::Nil),
        Expr::Literal(Value::Identifier(i)) => Ok(Value::Identifier(i.to_owned())),
        Expr::Literal(Value::String(s)) => Ok(Value::String(s.to_owned())),
        Expr::Literal(Value::Number(n)) => Ok(Value::Number(n.to_owned())),
        Expr::Literal(Value::Bool(true)) => Ok(Value::Bool(true)),
        Expr::Literal(Value::Bool(false)) => Ok(Value::Bool(false)),
        Expr::Grouping(expr) => evaluate(expr),
        Expr::Unary { operator, right } => {
            let right = evaluate(right)?;

            if let TokenType::Minus = operator.t && let Value::Number(n) = right {
                Ok(Value::Number(-n))
            } else if let TokenType::Minus = operator.t {
                Err(NewError::Runtime("Operand must be a number", operator.clone()))
            } else if let TokenType::Bang = operator.t {
                Ok(Value::Bool(!is_truthy(right)))
            } else {
                Ok(right)
            }
        },
        Expr::Binary { left, operator, right } => {
            let left = evaluate(left)?;
            let right = evaluate(right)?;

            if let Value::Number(n_1) = left && let Value::Number(n_2) = right {
                match operator.t {
                    TokenType::Minus => Ok(Value::Number(n_1 - n_2)),
                    TokenType::Slash => {
                        if n_2 == 0.0 {
                            return Err(Runtime("Cannot divide by 0", operator.clone()))
                        }

                        Ok(Value::Number(n_1 / n_2))
                    },
                    TokenType::Star => Ok(Value::Number(n_1 * n_2)),
                    TokenType::Plus => Ok(Value::Number(n_1 + n_2)),
                    TokenType::Greater => Ok(Value::Bool(n_1 > n_2)),
                    TokenType::GreaterEqual => Ok(Value::Bool(n_1 >= n_2)),
                    TokenType::Less => Ok(Value::Bool(n_1 < n_2)),
                    TokenType::LessEqual => Ok(Value::Bool(n_1 <= n_2)),
                    TokenType::EqualEqual => Ok(Value::Bool(n_1 == n_2)),
                    TokenType::BangEqual => Ok(Value::Bool(n_1 != n_2)),
                    _ => Err(Runtime("Invalid operator", operator.clone()))

                }
            } else if let Value::String(s_1) = &left && let Value::String(s_2) = &right {
                match operator.t {
                    TokenType::Plus => Ok(Value::String(s_1.to_string() + s_2)),
                    TokenType::EqualEqual => Ok(Value::Bool(s_1 == s_2)),
                    TokenType::BangEqual => Ok(Value::Bool(s_1 != s_2)),
                    _ => Err(Runtime("Operator is incompatible with strings", operator.clone()))

                }
            } else if let Value::Bool(b_1) = &left && let Value::Bool(b_2) = &right {
                match operator.t {
                    TokenType::EqualEqual => Ok(Value::Bool(b_1 == b_2)),
                    TokenType::BangEqual => Ok(Value::Bool(b_1 != b_2)),
                    _ => Err(Runtime("Operator is incompatible with booleans", operator.clone()))
                }
            }
            else {
                // for operands of differing types
                match operator.t {
                    TokenType::EqualEqual => Ok(Value::Bool(is_equal(&left, &right))),
                    TokenType::BangEqual =>  Ok(Value::Bool(!is_equal(&left, &right))),
                    _ => Err(Runtime("Incompatible operands", operator.clone()))
                }
            }


        }
    }
}

fn is_equal(left : &Value, right : &Value) -> bool {
    if let Value::Nil = left && let Value::Nil = right {
        true
    } else if let Value::Nil = left {
        false
    } else {
        false
    }
}

fn is_truthy(val : Value) -> bool {
    match val {
        Value::Nil | Value::Bool(false) => false,
        _ => true
    }
}
