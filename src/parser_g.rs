use crate::grammar::*;
use crate::interpreter_utils::{NewError, error};
use crate::lex::{self, Token, TokenType};
use crate::statement::Stmt;
use std::iter::Peekable;

pub struct Parser {
    pub tokens :  Peekable<std::vec::IntoIter<lex::Token>>,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Stmt>,NewError> {
        let mut statements : Vec<Stmt> = Vec::new();

        while let Some(token) = self.tokens.peek() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) ->  Result<Stmt, NewError> {
        if let Some(tok) = self.tokens.peek() {
            let token = self.tokens.next().unwrap();

            match token.t {
                TokenType::Redact => {
                    let res = self.print_statement();

                    res
                },
                _ => self.expression_statement()
            }
        } else {
            todo!()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, NewError> {
        let expr = self.expression()?;

        if let Some(tok) = self.tokens.peek() && let TokenType::Semicolon = tok.t {
            self.tokens.next();
            Ok(Stmt::Print(expr))
        } else if let Some(tok) = self.tokens.peek() {
            Err(NewError::Syntax(tok.line, "Missing semicolon"))
        } else {
            Err(NewError::MissingSemicolon)
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, NewError> {
        let expr = self.expression()?;

        if let Some(tok) = self.tokens.peek() && let TokenType::Semicolon = tok.t {
            self.tokens.next();
            Ok(Stmt::Expression(expr))
        } else if let Some(tok) = self.tokens.peek() {
            Err(NewError::Syntax(tok.line, "Missing semicolon"))
        } else {
            Err(NewError::MissingSemicolon)
        }
    }

    fn expression(&mut self) -> Result<Expr, NewError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.comparison().inspect_err(|e| {
            if let NewError::Parse(msg, t) = e {
                error(&t, msg);
            }
        })?;

        // peek next token to see if its != or ==
        while let Some(token) = self.tokens.peek() {
            let op = match token.t {
                TokenType::BangEqual | TokenType::EqualEqual => self.tokens.next().unwrap(),
                _ => break
            };

            let right = self.comparison()?;

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.term()?;

        while let Some(token) = self.tokens.peek() {
            let op = match token.t {
                TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => self.tokens.next().unwrap(),
                _ => break
            };

            let right = self.term()?;

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right)};
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.factor()?;

        while let Some(token) = self.tokens.peek() {
            let op = match token.t {
                TokenType::Minus | TokenType::Plus => self.tokens.next().unwrap(),
                _ => break
            };

            let right = self.factor()?;

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right)};
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.unary()?;

        while let Some(token) = self.tokens.peek() {
            let op = match token.t {
                TokenType::Star | TokenType::Slash => self.tokens.next().unwrap(),
                _ => break
            };

            let right = self.unary()?;

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right)};
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, NewError> {
        if let Some(token) = self.tokens.peek()  {
            match token.t {
                TokenType::Minus | TokenType::Bang => {
                    let op = self.tokens.next().unwrap();
                    let expr = self.unary();

                    let right = expr.inspect_err(|e| {
                        if let NewError::Parse(msg, t) = e {
                            error(&t, msg);
                        }
                    })?;

                    return Ok(Expr::Unary { operator: op, right: Box::new(right) })
                }
                _=> (),
            }
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, NewError> {
        if let Some(token) = self.tokens.peek() {
            let token = self.tokens.next().unwrap();

            let res = match token.t {
                TokenType::ThoughtCrime => Expr::Literal(Value::Bool(false)),
                TokenType::Truth => Expr::Literal(Value::Bool(true)),
                TokenType::Nil => Expr::Literal(Value::Nil),
                TokenType::Number(n) => Expr::Literal(Value::Number(n)),
                TokenType::String(s) => Expr::Literal(Value::String(s)),
                TokenType::LeftParen => {
                    let expr = self.expression().inspect_err(|e| {
                        if let NewError::Parse(msg, t) = e {
                            error(&t, msg);
                        }
                    })?;

                    let Some(tok) = self.tokens.next() else {
                        return Err(NewError::Parse("Expected succeeding token", token))
                    };

                    let TokenType::RightParen = tok.t else {
                        println!("{:?}", &expr);
                        return Err(NewError::Parse("Could not find right parantheses", token))
                    };


                    Expr::Grouping(Box::new(expr))
                }
                _ => return Err(NewError::Parse("Not a valid primary", token))
            };

            Ok(res)

        } else {
            panic!("Primary not found!")
        }
    }

    fn synchronize(&mut self) {
        while let Some(token) = self.tokens.peek() {
            if let TokenType::Semicolon = token.t {
                self.tokens.next().unwrap();
                return
            }

            match token.t {
                TokenType::Class | TokenType::Func | TokenType::Var | TokenType::For | TokenType::On | TokenType::While | TokenType::Redact | TokenType::Return =>  return,
                _ => self.tokens.next()
            };
        }
    }
}
