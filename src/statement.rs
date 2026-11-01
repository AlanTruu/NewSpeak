use crate::grammar::{Expr, Value, is_equal, is_truthy};
use crate::lex::Token;
use crate::interpreter_utils::{NewError, NewError::Runtime};
use crate::lex::TokenType;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Interpreter {}

impl Interpreter {
    pub fn execute(&mut self, stmt : &Stmt, env: Rc<RefCell<Environment>>) -> Value {
        match stmt {
            Stmt::Print(expr) => {
                match env.borrow_mut().evaluate(expr, self) {
                    Ok(val) => {
                        match val {
                            Value::Bool(false) => {
                                println!("thoughtCrime");
                                Value::Nil
                            },
                            Value::Bool(true) =>{
                                println!("truth");
                                Value::Nil
                            },
                            Value::Number(n) =>{
                                println!("{}", n);
                                Value::Nil
                            },
                            Value::String(s) =>{
                                println!("{}", s);
                                Value::Nil
                            },
                            Value::Identifier(i) =>{
                                println!("{}", i);
                                Value::Nil
                            },
                            Value::Nil =>{
                                println!("nil");
                                Value::Nil
                            },
                            _=> todo!()
                        }
                    },
                    Err(e) =>{
                        eprintln!("{:?}", e);
                        Value::Nil
                    }
                }
            },
            Stmt::Expression(expr) => {
                match env.borrow_mut().evaluate(expr, self) {
                    Ok(val) => {
                        if let Value::ReturnVal(val) = val {
                            return Value::ReturnVal(val)
                        }

                        Value::Nil
                    },
                    Err(e) =>{
                        eprintln!("{:?}", e);
                        Value::Nil
                    },
                }
            },
            Stmt::Var { name, initializer : Some(initializer) } => {
                let val = env.borrow_mut().evaluate(initializer, self);

                match val {
                    Ok(val) => env.borrow_mut().define(&name.lexeme, val),
                    Err(e) => eprintln!("{:?}", e)
                }
                Value::Nil
            },
            Stmt::Var { name, initializer : None } => {
                env.borrow_mut().define(&name.lexeme, Value::Nil);
                Value::Nil
            },
            Stmt::Block(statements) => {
                let outer = Some(Rc::clone(&env));

                let inner_env = Environment {
                    values : HashMap::new(),
                    globals : Rc::clone(&env.borrow().globals),
                    enclosing : outer
                };

                let inner = Rc::new(RefCell::new(inner_env));

                if let Value::ReturnVal(val) = self.interpret(statements, inner) {
                    return Value::ReturnVal(val)
                }

                Value::Nil

            },
            Stmt::If { condition, on, otherwise } => {

                let Ok(val) = env.borrow_mut().evaluate(condition, self) else {
                    panic!("Could not evaluate on condition")
                };

                if is_truthy(&val) {
                    if let Value::ReturnVal(val) = self.execute(on, env) {
                        return Value::ReturnVal(val)
                    }
                } else if let Some(otherwise_expr) = otherwise {
                    if let Value::ReturnVal(val) = self.execute(otherwise_expr, env) {
                        return Value::ReturnVal(val)
                    }
                }

                Value::Nil
            },
            Stmt::While { condition, body } => {
                loop {

                    if let Ok(val) = env.borrow_mut().evaluate(&condition, self) {
                        if !is_truthy(&val) {
                            break
                        }
                    } else {
                        break
                    }

                     if let Value::ReturnVal(val) = self.execute(body, Rc::clone(&env)) {
                         return Value::ReturnVal(val)
                     }
                }

                Value::Nil
            },
            Stmt::Function { name, params, body } => {
                let func = Value::Function{params: params.clone(), body : body.clone(), closure : Rc::clone(&env)};

                env.borrow_mut().define(&name.lexeme, func);

                Value::Nil
            },
            Stmt::Return { keyword, value } => {
                if let Some(value) = value {
                    let value = env.borrow_mut().evaluate(value, self);

                    match value {
                        Ok(value) =>{
                            return Value::ReturnVal(Box::new(value));
                        }
                        Err(e) => {
                            eprintln!("Error with return value: {:?}", e)
                        }
                    }
                }

                Value::Nil
            }
        }
    }

    pub fn interpret(&mut self, statements : &[Stmt], env : Rc<RefCell<Environment>>) -> Value {
        for statement in statements {
            if let Value::ReturnVal(inner) =  self.execute(statement, Rc::clone(&env)) {
                return Value::ReturnVal(inner)
            }
        }

        Value::Nil
    }
}

#[derive(Debug)]
/// Encapsulates environment, contains values table for user defined values/functions, and globals table for developer defined functions.
pub struct Environment {
    pub values : HashMap<String, Value>,
    pub enclosing :  Option<Rc<RefCell<Environment>>>,
    pub globals : Rc<HashMap<String, Box<dyn NewSpeakCallable>>>
}

impl Environment {
    fn try_globals(&self, name : &str) -> Option<&Box<dyn NewSpeakCallable>> {
        self.globals.get(name)
    }

    pub fn evaluate(&mut self, e : &Expr, interpreter : &mut Interpreter) -> Result<Value, NewError> {
        match e {
            Expr::Literal(Value::Nil) => Ok(Value::Nil),
            Expr::Literal(Value::Identifier(i)) => Ok(Value::Identifier(i.to_owned())),
            Expr::Literal(Value::String(s)) => Ok(Value::String(s.to_owned())),
            Expr::Literal(Value::Number(n)) => Ok(Value::Number(n.to_owned())),
            Expr::Literal(Value::Bool(true)) => Ok(Value::Bool(true)),
            Expr::Literal(Value::Bool(false)) => Ok(Value::Bool(false)),
            Expr::Grouping(expr) => self.evaluate(expr, interpreter),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right, interpreter)?;

                if let TokenType::Minus = operator.t && let Value::Number(n) = right {
                    Ok(Value::Number(-n))
                } else if let TokenType::Minus = operator.t {
                    Err(NewError::Runtime("Operand must be a number", operator.clone()))
                } else if let TokenType::Bang = operator.t {
                    Ok(Value::Bool(!is_truthy(&right)))
                } else {
                    Ok(right)
                }
            },
            Expr::Binary { left, operator, right } => {
                let left = self.evaluate(left, interpreter)?;
                let right = self.evaluate(right, interpreter)?;

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
                let val = self.evaluate(&val, interpreter)?;

                self.assign(name, val.clone())?;

                Ok(val)
            },
            Expr::Logical { left, operator, right } => {
                let left = self.evaluate(left, interpreter)?;

                // short circuiting
                if let TokenType::Or = operator.t {
                    if is_truthy(&left) {
                        return Ok(left)
                    }
                } else {
                    if !is_truthy(&left) {
                        return Ok(left)
                    }
                }

                self.evaluate(right, interpreter)
            },
            Expr::Call { callee, paren, args } => {
                let res = self.evaluate(callee, interpreter);

                match res {
                    Ok(callee) => {
                        let mut arguments : Vec<Value> = Vec::new();

                        for arg in args {
                            arguments.push(self.evaluate(arg, interpreter)?);
                        }

                        let Value::Function {
                            params : _, body : _, closure : _
                        } = callee else {
                            return Err(NewError::Runtime("Callee is not a function", paren.clone()))
                        };

                        if arguments.len() != callee.arity() {
                            return Err(Runtime("Incorrect number of arguments were passed to this function", paren.clone()))
                        }

                        callee.call(self, interpreter, &arguments)
                    },
                    Err(_) => {
                        if let Expr::Variable(v) = &**callee {
                            let Some(global) = self.try_globals(&v.lexeme) else {
                                return Err(Runtime("Undefined function", paren.clone()))
                            };

                            global.call(self, interpreter, &vec![])
                        } else {
                            return Err(Runtime("Undefined function", paren.clone()))
                        }
                    }
                }


            },
            _=> panic!("Unknown error occured")
        }
    }

    pub fn define(&mut self, name : &str, value : Value) {
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
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Block(Vec<Stmt>),
    If {
        condition : Expr,
        on : Box<Stmt>,
        otherwise : Option<Box<Stmt>>,
    },
    Var {
        name : Token,
        initializer : Option<Expr>
    },
    While {
        condition : Expr,
        body : Box<Stmt>
    },
    Function {
        name : Token,
        params : Vec<Token>,
        body : Vec<Stmt>
    },
    Return {
        keyword : Token,
        value : Option<Expr>
    }
}

pub trait NewSpeakCallable {
    fn call(&self, env : &Environment, i : &mut Interpreter, arguments : &[Value]) -> Result<Value, NewError>;
    fn arity(&self) -> usize;
}

impl Debug for dyn NewSpeakCallable {
    fn fmt(&self, f : &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "yada")
    }
}
