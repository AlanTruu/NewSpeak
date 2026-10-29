use crate::grammar::{Expr, Value, is_equal, is_truthy};
use crate::lex::Token;
use crate::interpreter_utils::{NewError, NewError::Runtime};
use crate::lex::TokenType;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Interpreter {

}

impl Interpreter {
    pub fn execute(&self, env : Rc<RefCell<Environment>>, stmt : &Stmt) {
        match stmt {
            Stmt::Print(expr) => {
                match env.borrow_mut().evaluate(expr) {
                    Ok(val) => {
                        match val {
                            Value::Bool(false) => println!("thoughtCrime"),
                            Value::Bool(true) => println!("truth"),
                            Value::Number(n) => println!("{}", n),
                            Value::String(s) => println!("{}", s),
                            Value::Identifier(i) => println!("{}", i),
                            Value::Nil => println!("nil")
                        }
                    },
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Stmt::Expression(expr) => {
                match env.borrow_mut().evaluate(expr) {
                    Ok(v) => {

                    },
                    Err(e) => eprintln!("{:?}", e),
                    _ => todo!()
                }
            },
            Stmt::Var { name, initializer : Some(initializer) } => {
                let val = env.borrow_mut().evaluate(initializer);

                match val {
                    Ok(val) => env.borrow_mut().define(&name.lexeme, val),
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Stmt::Var { name, initializer : None } => {
                env.borrow_mut().define(&name.lexeme, Value::Nil);
            },
            Stmt::Block(statements) => {
                let outer = Some(Rc::clone(&env));

                let inner_env = Environment {
                    values : HashMap::new(),
                    enclosing : outer
                };

                self.interpret(Rc::new(RefCell::new(inner_env)), statements);
            },
            _ => todo!()
        }
    }

    pub fn interpret(&self, env : Rc<RefCell<Environment>>, statements : &[Stmt]) {
        for statement in statements {
            self.execute(env.clone(), statement);
        }
    }
}

pub struct Environment {
    pub values : HashMap<String, Value>,
    pub enclosing :  Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn evaluate(&mut self, e : &Expr) -> Result<Value, NewError> {
        match e {
            Expr::Literal(Value::Nil) => Ok(Value::Nil),
            Expr::Literal(Value::Identifier(i)) => Ok(Value::Identifier(i.to_owned())),
            Expr::Literal(Value::String(s)) => Ok(Value::String(s.to_owned())),
            Expr::Literal(Value::Number(n)) => Ok(Value::Number(n.to_owned())),
            Expr::Literal(Value::Bool(true)) => Ok(Value::Bool(true)),
            Expr::Literal(Value::Bool(false)) => Ok(Value::Bool(false)),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

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
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

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
            },
            Expr::Variable(v) => Ok(self.get(v)?.clone()),
            Expr::Assign { name, val } => {
                let val = self.evaluate(&val)?;

                self.assign(name, val.clone())?;

                Ok(val)
            }
        }
    }

    fn define(&mut self, name : &str, value : Value) {
        self.values.insert(name.into(), value);
    }

    fn assign(&mut self, name : &Token, value : Value) -> Result<(), NewError> {
        if self.values.contains_key(&name.lexeme as &str) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(NewError::Runtime("Undefined variable", name.clone()))
        }
    }

    fn get(&self, name : &Token) -> Result<Value, NewError> {
        let val_or_none = self.values.get(&name.lexeme as &str);

        match val_or_none {
            Some(val) => Ok(val.clone()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                   let Ok(val) = enclosing.borrow_mut().get(name) else {
                       return Err(NewError::Runtime("Undefined variable", name.clone()))
                   };

                   Ok(val)
                } else {
                    Err(NewError::Runtime("Undefined variable", name.clone()))
                }
            }
        }
    }

//     pub fn execute(&mut self, stmt : &Stmt) {
//         match stmt {
//             Stmt::Print(expr) => {
//                 match self.evaluate(expr) {
//                     Ok(val) => {
//                         match val {
//                             Value::Bool(false) => println!("thoughtCrime"),
//                             Value::Bool(true) => println!("truth"),
//                             Value::Number(n) => println!("{}", n),
//                             Value::String(s) => println!("{}", s),
//                             Value::Identifier(i) => println!("{}", i),
//                             Value::Nil => println!("nil")
//                         }
//                     },
//                     Err(e) => eprintln!("{:?}", e)
//                 }
//             },
//             Stmt::Expression(expr) => {
//                 match self.evaluate(expr) {
//                     Ok(v) => {
// 
//                     },
//                     Err(e) => eprintln!("{:?}", e),
//                     _ => todo!()
//                 }
//             },
//             Stmt::Var { name, initializer : Some(initializer) } => {
//                 let val = self.evaluate(initializer);
// 
//                 match val {
//                     Ok(val) => self.define(&name.lexeme, val),
//                     Err(e) => eprintln!("{:?}", e)
//                 }
//             },
//             Stmt::Var { name, initializer : None } => {
//                 self.define(&name.lexeme, Value::Nil);
//             },
//             Stmt::Block(statements) => {
// 
//             }
//             _ => todo!()
//         }
//     }
// 
//     pub fn interpret(&mut self, statements : &[Stmt]) {
//         for statement in statements {
//             self.execute(statement);
//         }
//     }
}

pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Block(Vec<Stmt>),
    If(Expr),
    Var {
        name : Token,
        initializer : Option<Expr>
    }
}
