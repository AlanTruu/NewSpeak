use crate::grammar::{Expr, Value, is_equal, is_truthy};
use crate::lex::Token;
use crate::interpreter_utils::{NewError, NewError::Runtime};
use crate::lex::TokenType;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct Interpreter {
    locals : HashMap<String, usize>,
}

#[derive(Debug)]
pub struct Scope {
    pub environments : Vec<Environment>
}

impl Scope {
    pub fn exit_scope(&mut self) {
        self.environments.pop();
    }

    pub fn get_current_scope(&self) -> &Environment {
        self.environments.last().unwrap()
    }

    pub fn get_current_mut_scope(&mut self) ->  &mut Environment {
        self.environments.last_mut().unwrap()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            locals : HashMap::new()
        }
    }

    fn get(&self, name : &Token, index : usize, scope : &Scope) -> Result<Value, NewError> {
        let val_or_none = scope.environments[index].values.get(&name.lexeme);

        match val_or_none {
            Some(val) => Ok(val.clone()),
            None => {
                    if index > 0 {
                        return self.get(name, index - 1, scope)
                    } else {
                        return Err(NewError::Runtime("Undefined variable", name.clone()))
                    }

            }
        }
    }

    fn assign(&mut self, name : &Token, value : Value, index : usize, scope : &mut Scope) -> Result<(), NewError> {
        if scope.environments[index].values.contains_key(&name.lexeme as &str) {
            scope.environments[index].values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if index > 0 {
            self.assign(name, value, index - 1, scope)
        } else {
            Err(NewError::Runtime("Undefined variable", name.clone()))
        }
    }

    // pub fn find(&self, name : &Token) -> Option<&Value> {
    //     self.get_current_scope().values.get(&name.lexeme)
    // }

    pub fn resolve(&mut self, name : &str, depth : usize) {
        self.locals.insert(name.to_string(), depth);
    }

    pub fn look_up_var(&self, name : &str) ->  Option<&usize>{
        self.locals.get(name)
    }

    pub fn execute(&mut self, stmt : &Stmt, scope : &mut Scope, globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Value {
        match stmt {
            Stmt::Print(expr) => {
                let val_or_err = scope.evaluate(expr, self, globals);

                match val_or_err {
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
                let val_or_err = scope.evaluate(expr, self, globals);

                match val_or_err {
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
                let val = scope.evaluate(initializer, self, globals);

                match val {
                    Ok(val) => scope.define(&name.lexeme, val),
                    Err(e) => eprintln!("{:?}", e)
                }
                Value::Nil
            },
            Stmt::Var { name, initializer : None } => {
                scope.define(&name.lexeme, Value::Nil);
                Value::Nil
            },
            Stmt::Block(statements) => {

                let inner_env = Environment {
                    values : HashMap::new(),
                    index : scope.get_current_scope().index + 1
                };

                scope.environments.push(inner_env);

                if let Value::ReturnVal(val) = self.interpret(statements, scope, globals) {
                    scope.environments.pop();
                    return Value::ReturnVal(val)
                } else {
                    scope.environments.pop();
                    Value::Nil
                }
            },
            Stmt::If { condition, on, otherwise } => {

                let Ok(val) = scope.evaluate(condition, self, globals) else {
                    panic!("Could not evaluate on condition")
                };

                if is_truthy(&val) {
                    if let Value::ReturnVal(val) = self.execute(on, scope, globals) {
                        return Value::ReturnVal(val)
                    }
                } else if let Some(otherwise_expr) = otherwise {
                    if let Value::ReturnVal(val) = self.execute(otherwise_expr, scope, globals) {
                        return Value::ReturnVal(val)
                    }
                }

                Value::Nil
            },
            Stmt::While { condition, body } => {
                loop {

                    if let Ok(val) = scope.evaluate(&condition, self, globals) {
                        if !is_truthy(&val) {
                            break
                        }
                    } else {
                        break
                    }

                     if let Value::ReturnVal(val) = self.execute(body, scope, globals) {
                         return Value::ReturnVal(val)
                     }
                }

                Value::Nil
            },
            Stmt::Function { name, params, body } => {
                let func = Value::Function{params: params.clone(), body : body.clone(), closure_index : scope.get_current_scope().index};

                scope.define(&name.lexeme, func);

                Value::Nil
            },
            Stmt::Return { keyword : _, value } => {
                if let Some(value) = value {
                    let value = scope.evaluate(value, self, globals);

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

    pub fn interpret(&mut self, statements : &[Stmt], scope : &mut Scope, globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Value {
        for statement in statements {
            if let Value::ReturnVal(inner) =  self.execute(statement, scope, globals) {
                return Value::ReturnVal(inner)
            }
        }

        Value::Nil
    }
}

#[derive(Debug, Clone)]
/// Encapsulates environment, contains values table for user defined values/functions, and globals table for developer defined functions.
pub struct Environment {
    pub values : HashMap<String, Value>,
    pub index : usize

}

impl Scope {
    pub fn new() -> Self {
        Self {
            environments : Vec::new()
        }
    }

    pub fn from(source : &[Environment]) -> Self {
        Self {
            environments : source.into()
        }
    }

    pub fn evaluate(&mut self, e : &Expr, interpreter : &mut Interpreter, globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError> {
        match e {
            Expr::Literal(Value::Nil) => Ok(Value::Nil),
            Expr::Literal(Value::Identifier(i)) => Ok(Value::Identifier(i.to_owned())),
            Expr::Literal(Value::String(s)) => Ok(Value::String(s.to_owned())),
            Expr::Literal(Value::Number(n)) => Ok(Value::Number(n.to_owned())),
            Expr::Literal(Value::Bool(true)) => Ok(Value::Bool(true)),
            Expr::Literal(Value::Bool(false)) => Ok(Value::Bool(false)),
            Expr::Grouping(expr) => self.evaluate(expr, interpreter, globals),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right, interpreter, globals)?;

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
                let left = self.evaluate(left, interpreter, globals)?;
                let right = self.evaluate(right, interpreter, globals)?;

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
            Expr::Variable(v) => {
//                 let dist_or_none = interpreter.look_up_var(&v.lexeme);
//
//                 if let Some(dist) = dist_or_none {
//                     // find var
//                 } else {
//                     // check in globals
//                     todo!()
//                 }

                interpreter.get(v, self.get_current_scope().index, self)

                // Ok(interpreter.get(v, self.get_current_scope().index, &self)?)
            },
            Expr::Assign { name, val } => {
                let val = self.evaluate(&val, interpreter, globals)?;

                interpreter.assign(name, val, self.get_current_scope().index, self)?;

                Ok(Value::Nil)
            },
            Expr::Logical { left, operator, right } => {
                let left = self.evaluate(left, interpreter, globals)?;

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

                self.evaluate(right, interpreter, globals)
            },
            Expr::Call { callee, paren, args } => {
                let res = self.evaluate(callee, interpreter, globals);

                match res {
                    Ok(callee) => {
                        let mut arguments : Vec<Value> = Vec::new();

                        for arg in args {
                            arguments.push(self.evaluate(arg, interpreter, globals)?);
                        }

                        let Value::Function {
                            params : _, body : _, closure_index : _
                        } = callee else {
                            return Err(NewError::Runtime("Callee is not a function", paren.clone()))
                        };

                        if arguments.len() != callee.arity() {
                            return Err(Runtime("Incorrect number of arguments were passed to this function", paren.clone()))
                        }

                        callee.call(self, interpreter, &arguments, globals)
                    },
                    Err(_) => {
                        if let Expr::Variable(v) = &**callee {
                            let Some(callable) = globals.get(&v.lexeme) else {
                                return Err(Runtime("Incorrect number of arguments were passed to this function", paren.clone()))
                            };

                            return callable.call(self, interpreter, &vec![], globals)
                        }

                        Ok(Value::Nil)
                    }
                }
            },
            _=> panic!("Unknown error occured")
        }
    }

    pub fn define(&mut self, name : &str, value : Value) {
        self.get_current_mut_scope().values.insert(name.into(), value);
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
    fn call(&self, env : &mut Scope, i : &mut Interpreter, arguments : &[Value], globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError>;
    fn arity(&self) -> usize;
}

impl Debug for dyn NewSpeakCallable {
    fn fmt(&self, f : &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "yada")
    }
}
