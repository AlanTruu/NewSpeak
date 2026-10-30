use crate::{grammar::*, statement};
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
            let statement_or_err = self.declaration();

            match statement_or_err {
                Ok(statement) => statements.push(statement),
                Err(NewError::Parse(msg, t)) => {
                    error(&t, msg);
                    self.synchronize();
                },
                _ => self.synchronize(),
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, NewError> {
        if let Some(token) = self.tokens.peek() && let TokenType::Var = token.t {

            // consume var keyword
            let token = self.tokens.next().unwrap();

            match self.var_declaration() {
                Ok(statement) => Ok(statement),
                Err(NewError::FillInStatement) => Err(NewError::Parse("Missing identifier", token)),
                Err(e) => Err(e),
            }
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) ->  Result<Stmt, NewError> {
        if let Some(tok) = self.tokens.next_if(|tok| if let TokenType::Redact = tok.t {true} else {false}) {
            let res = self.print_statement();

            match res {
                Ok(statement) => Ok(statement),
                Err(NewError::Parse(msg, token)) => Err(NewError::Parse(msg, token)),
                Err(NewError::FillInSemicolon) => Err(NewError::Parse("Missing semicolon", tok.clone())),
                _=> unreachable!()
            }
        } else if let Some(tok) = self.tokens.peek() && let TokenType::LeftBrace = tok.t {
            Ok(Stmt::Block(self.block()?))
        } else if let Some(tok) = self.tokens.peek() && let TokenType::On = tok.t {
            self.if_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::While = tok.t {
            self.while_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::For = tok.t {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, NewError> {
        // consume for keyword
        let for_token = self.tokens.next().unwrap();

        let Some(_left_paren) = self.tokens.next_if(|tok| if let TokenType::LeftParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Missing '(' after 'for'", for_token))
        };

        let Some(tok) = self.tokens.peek() else {
            return Err(NewError::Parse("Missing expression after 'for'", for_token))
        };

        let initializer = match tok.t {
            TokenType::Semicolon => {
                // consume semicolon
                self.tokens.next().unwrap();
                None
            },
            TokenType::Var => {
                // consume var keyword
                self.tokens.next().unwrap();
                Some(self.var_declaration()?)
            },
            _=> Some(self.expression_statement()?)
        };

        let Some(tok) = self.tokens.peek() else {
            return Err(NewError::Parse("Missing expression after 'for'", for_token))
        };

        let condition = match tok.t {
            TokenType::Semicolon => {
                None
            },
            _ => Some(self.expression()?)
        };

        // consume semicolon
        let Some(_) = self.tokens.next_if(|tok| if let TokenType::Semicolon = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Expected ; after loop condition", for_token))
        };

        let Some(tok) = self.tokens.peek() else {
            return Err(NewError::Parse("Incomplete 'for' statement", for_token))
        };

        let increment = match tok.t {
            TokenType::RightParen => {
                None
            },
            _ => Some(self.expression()?)
        };

        let Some(_) = self.tokens.next_if(|tok| if let TokenType::RightParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Expected ; after loop condition", for_token))
        };

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        if let Some(condition) = condition {
            body = Stmt::While { condition, body : Box::new(body) };
        } else {
            body = Stmt::While { condition: Expr::Literal(Value::Bool(true)), body : Box::new(body) };
        }

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, NewError> {
        // consume while keyword
        let while_token = self.tokens.next().unwrap();

        let Some(_left_paren) = self.tokens.next_if(|tok| if let TokenType::LeftParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Expected '(' after 'while'", while_token))
        };

        let condition = self.expression()?;

        let Some(_right_paren) = self.tokens.next_if(|tok| if let TokenType::RightParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Expected ')' after condition", while_token))
        };

        let body = self.statement()?;

        Ok(Stmt::While { condition, body : Box::new(body) })
    }

    fn if_statement(&mut self) -> Result<Stmt, NewError> {
        // consume on keyword
        self.tokens.next().unwrap();

        let Some(_left_paren) = self.tokens.next_if(|tok| if let TokenType::LeftParen = tok.t {true} else {false}) else {
            panic!("Expected '(' after on")
        };

        let cond = self.expression()?;

        let Some(_right_paren) = self.tokens.next_if(|tok| if let TokenType::RightParen = tok.t {true} else {false}) else {
            panic!("Expected ')' after on condition")
        };

        let on_branch = self.statement()?;

        if let Some(_otherwise_token) = self.tokens.next_if(|tok| if let TokenType::Otherwise = tok.t {true} else {false}) {
            let otherwise_branch = self.statement()?;

            Ok(Stmt::If { condition: cond, on: Box::new(on_branch), otherwise: Some(Box::new(otherwise_branch)) })
        } else {
            Ok(Stmt::If { condition: cond, on: Box::new(on_branch), otherwise: None })
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, NewError> {
        let expr = self.expression()?;

        if let Some(tok) = self.tokens.peek() && let TokenType::Semicolon = tok.t {
            self.tokens.next();
            Ok(Stmt::Print(expr))
        } else if let Some(tok) = self.tokens.peek() {
            Err(NewError::Parse("Missing semicolon", tok.clone()))
        } else {
            Err(NewError::FillInSemicolon)
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, NewError> {
        let (id_or_none, equal_or_none) = (self.tokens.next_if(|tok| if let TokenType::Identifier(_) = tok.t {true} else {false}), self.tokens.next_if(|tok| if let TokenType::Equal = tok.t {true} else {false} ));

        match (id_or_none, equal_or_none) {
            (Some(token_id), Some(_)) => {
                let expr = self.expression()?;

                let semicolon = self.tokens.next_if(|tok| if let TokenType::Semicolon = tok.t {true} else {false});

                if let Some(_) = semicolon {
                    Ok(Stmt::Var { name: token_id, initializer: Some(expr) })
                } else {
                    Err(NewError::Parse("Expected semicolon", token_id))
                }
            },
            (Some(token_id), None) => {
                let semicolon = self.tokens.next_if(|tok| if let TokenType::Semicolon = tok.t {true} else {false});

                if let Some(_) = semicolon {
                    Ok(Stmt::Var { name: token_id, initializer: None })
                } else {
                    Err(NewError::Parse("Expected semicolon", token_id))
                }
            },
            _ => Err(NewError::FillInStatement)

        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, NewError> {
        let expr = self.expression()?;

        if let Some(tok) = self.tokens.peek() && let TokenType::Semicolon = tok.t {
            self.tokens.next();
            Ok(Stmt::Expression(expr))
        } else if let Some(tok) = self.tokens.peek() {
            Err(NewError::Parse("Missing semicolon", tok.clone()))
        } else {
            // if there is no token available, defer error type to calling function
            Err(NewError::FillInSemicolon)
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, NewError>{
        // consume left brace
        let left_brace = self.tokens.next().unwrap();

        let mut statements : Vec<Stmt> = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if let TokenType::RightBrace = token.t {
                break
            }

            statements.push(self.declaration()?);
        }

        let Some(_right_brace) = self.tokens.next_if(|tok| if let TokenType::RightBrace = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Missing closing }", left_brace))
        };

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, NewError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, NewError> {
        let expr = self.or()?;

        if let Some(token) = self.tokens.peek() && let TokenType::Equal = token.t {
            let _equals = self.tokens.next().unwrap();

            let val = self.assignment()?;

            match expr {
                Expr::Variable(name) => {
                    Ok(Expr::Assign { name, val : Box::new(val) })
                },
                _ => Err(NewError::Assignment)
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.and()?;

        while let Some(operator) = self.tokens.next_if(|tok| if let TokenType::Or = tok.t {true} else {false}) {
            let right = self.and()?;

            expr = Expr::Logical { left: Box::new(expr), operator, right : Box::new(right) };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.tokens.next_if(|tok| if let TokenType::And = tok.t {true} else {false}) {
            let right = self.equality()?;

            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.comparison()?;

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
        if let Some(_token) = self.tokens.peek() {
            let token = self.tokens.next().unwrap();

            let res = match &token.t {
                TokenType::ThoughtCrime => Expr::Literal(Value::Bool(false)),
                TokenType::Truth => Expr::Literal(Value::Bool(true)),
                TokenType::Nil => Expr::Literal(Value::Nil),
                TokenType::Number(n) => Expr::Literal(Value::Number(n.to_owned())),
                TokenType::String(s) => Expr::Literal(Value::String(s.to_owned())),
                TokenType::Identifier(i) => Expr::Variable(token) ,
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
                        return Err(NewError::Parse("Could not find right parantheses", token))
                    };


                    Expr::Grouping(Box::new(expr))
                },

                _ =>  return Err(NewError::Parse("Not a valid primary", token))

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
