use crate::{grammar::*};
use crate::interpreter_utils::{NewError, error};
use crate::lex::{self, Token, TokenType};
use crate::statement::Stmt;
use std::iter::Peekable;

const MAXIMUM_ARGS : usize = 255;

pub struct Parser {
    pub tokens :  Peekable<std::vec::IntoIter<lex::Token>>,
    pub index : usize
}

impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Stmt>,NewError> {
        let mut statements : Vec<Stmt> = Vec::new();

        while let Some(_) = self.tokens.peek() {
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
            self.var_declaration()
        } else if let Some(token) = self.tokens.peek() && let TokenType::Func = token.t {
            self.function("function")
        } else {
            self.statement()
        }
    }

    fn function(&mut self, _kind : &str) -> Result<Stmt, NewError> {
        // Consume func keyword
        let func_token = self.tokens.next().unwrap();

        let Some(name) = self.tokens.next_if(|tok| if let TokenType::Identifier(_) = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Missing identifier for function", func_token))
        };

        let Some(left_paren) = self.tokens.next_if(|tok| if let TokenType::LeftParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Expected '(' after function name", name))
        };

        let mut params : Vec<Token> = Vec::new();

        if let Some(token) = self.tokens.peek() {
            match token.t {
                TokenType::RightParen => (),
                _ => loop {
                    if params.len() >= MAXIMUM_ARGS {
                        error(&left_paren, "Can't have more than 255 parameters");
                    }

                    // consume identifiers and add to param list
                    let Some(identifier) = self.tokens.next_if(|tok| if let TokenType::Identifier(_) = tok.t {true} else {false}) else {
                        return Err(NewError::Parse("Expect parameter name", name.clone()))
                    };

                    params.push(identifier);

                    match self.tokens.next_if(|tok| if let TokenType::Comma = tok.t {true} else {false}) {
                        Some(_) => (),
                        _=> break
                    }
                }
            }
        };

        let Some(_right_paren) = self.tokens.next_if(|tok| if let TokenType::RightParen = tok.t {true} else {false}) else {
            return Err(NewError::Parse("Missing ')' after params", name.clone()));
        };

        let body = self.block()?;

        Ok(Stmt::Function { name, params, body })

    }

    fn statement(&mut self) ->  Result<Stmt, NewError> {
        if let Some(tok) = self.tokens.peek() && let TokenType::Redact = tok.t {
            self.print_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::LeftBrace = tok.t {
            Ok(Stmt::Block(self.block()?))
        } else if let Some(tok) = self.tokens.peek() && let TokenType::On = tok.t {
            self.if_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::While = tok.t {
            self.while_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::For = tok.t {
            self.for_statement()
        } else if let Some(tok) = self.tokens.peek() && let TokenType::Return = tok.t {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, NewError>{
        let ret_token = self.tokens.next().unwrap();

        if let Some(next) = self.tokens.peek() && let TokenType::Semicolon = next.t {
            return Ok(Stmt::Return { keyword: ret_token, value: None })
        } else if let Some(_) = self.tokens.peek() {
            let value = Some(self.expression()?);

            if let Some(_) = self.tokens.next_if(|tok| if let TokenType::Semicolon = tok.t {true} else {false}) {
                return Ok(Stmt::Return { keyword: ret_token, value});
            } else {
                return Err(NewError::Parse("Expect ';' after return value", ret_token))
            }
        }

        return Err(NewError::Parse("Expect ';' after return value", ret_token))


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
            return Err(NewError::Parse("Expected ')' after loop condition", for_token))
        };

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        if let Some(condition) = condition {
            body = Stmt::While { condition, body : Box::new(body) };
        } else {
            body = Stmt::While { condition: Expr::Literal{value : Value::Bool(true), index : self.index + 1}, body : Box::new(body) };
            self.index += 1;
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
        // consume redact keyword
        let redact_token = self.tokens.next().unwrap();

        let expr = self.expression()?;

        if let Some(tok) = self.tokens.peek() && let TokenType::Semicolon = tok.t {
            self.tokens.next();
            Ok(Stmt::Print(expr))
        } else if let Some(tok) = self.tokens.peek() {
            Err(NewError::Parse("Missing semicolon", tok.clone()))
        } else {
            Err(NewError::Parse("Missing semicolon", redact_token))
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, NewError> {
        // consume var keyword
        let var_token = self.tokens.next().unwrap();

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
            _ => Err(NewError::Parse("Missing identifier for var", var_token))

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
                Expr::Variable { token, index }=> {
                    self.index += 1;
                    Ok(Expr::Assign { name : token, val : Box::new(val), index })
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

            expr = Expr::Logical { left: Box::new(expr), operator, right : Box::new(right), index: self.index };
            self.index += 1;
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, NewError> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.tokens.next_if(|tok| if let TokenType::And = tok.t {true} else {false}) {
            let right = self.equality()?;

            expr = Expr::Logical { left: Box::new(expr), operator, right: Box::new(right), index: self.index };
            self.index += 1;
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

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right), index: self.index };
            self.index += 1;
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

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right), index : self.index};
            self.index += 1;
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

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right), index: self.index};
            self.index += 1;
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

            expr = Expr::Binary { left: Box::new(expr), operator: op, right: Box::new(right), index : self.index};
            self.index += 1;
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

                    let res = Ok(Expr::Unary { operator: op, right: Box::new(right), index : self.index });
                    self.index +=1;

                    return res

                }
                _=> (),
            }
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, NewError> {
        // get callee/function name
        let mut expr = self.primary()?;

        loop {
            if let Some(_tok) = self.tokens.next_if(|tok| if let TokenType::LeftParen = tok.t {true} else {false}) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee : Expr) -> Result<Expr, NewError> {
        let mut args : Vec<Expr> = Vec::new();

        // check for right paren
        if let Some(token) = self.tokens.peek() {
               match token.t {
                   TokenType::RightParen => (),
                   _ => loop {
                            if args.len() >= MAXIMUM_ARGS {
                                if let Some(token) = self.tokens.next() {
                                    error(&token, "Cannot have more than 255 arguments to a function");
                                }
                                break;
                            }

                            args.push(self.expression()?);
                            if let Some(next) = self.tokens.peek() {
                                match next.t {
                                    TokenType::Comma => self.tokens.next().unwrap(),
                                    _ => break
                                };
                            }
                   }
           }
       }

        // consume right paren
        let Some(paren) = self.tokens.next_if(|tok| if let TokenType::RightParen = tok.t {true} else {false}) else {
            return Err(NewError::FillInStatement)
        };

        let res = Ok(Expr::Call { callee : Box::new(callee), paren, args, index : self.index });

        self.index += 1;

        res
    }

    fn primary(&mut self) -> Result<Expr, NewError> {
        if let Some(_token) = self.tokens.peek() {
            let token = self.tokens.next().unwrap();

            let res = match &token.t {
                TokenType::ThoughtCrime => Expr::Literal{value : Value::Bool(false), index: self.index},
                TokenType::Truth => Expr::Literal{value : Value::Bool(true), index: self.index},
                TokenType::Nil => Expr::Literal{value : Value::Nil, index : self.index},
                TokenType::Number(n) => Expr::Literal{value : Value::Number(n.to_owned()), index: self.index},
                TokenType::String(s) => Expr::Literal{value : Value::String(s.to_owned()), index : self.index},
                TokenType::Identifier(_) => Expr::Variable{token, index : self.index} ,
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


                    Expr::Grouping{group :Box::new(expr), index : self.index}
                },

                _ =>  return Err(NewError::Parse("Not a valid primary", token))

            };

            self.index += 1;
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
