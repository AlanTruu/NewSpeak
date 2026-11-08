use crate::grammar::{Expr, Value, is_equal, is_truthy};
use crate::lex::Token;
use crate::interpreter_utils::{NewError, NewError::Runtime};
use crate::lex::TokenType;
use crate::class::NsClass;
use std::collections::HashMap;
use std::fmt::Debug;

/// Interpreter type runs statements and keeps local variables stored
pub struct Interpreter {
    pub locals : HashMap<usize, usize>,
}

#[derive(Debug)]
pub struct Scope {
    pub environments : Vec<Environment>
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

    pub fn resolve(&mut self, index : usize, depth : usize) {
        self.locals.insert(index, depth);
    }

    pub fn look_up_var(&self, index : &usize) ->  Option<&usize> {
        // println!("Looking up expr: {:?}", name);
        self.locals.get(index)
    }

    pub fn execute(&mut self, stmt : &Stmt, scope : &mut Scope, globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Value {
        match stmt {
            Stmt::Print(expr) => {
                let val_or_err = scope.evaluate(expr, self, globals);

                match val_or_err {
                    Ok(val) => {
                        match val {
                            Value::Bool(false) => println!("thoughtCrime"),
                            Value::Bool(true) => println!("truth"),
                            Value::Number(n) => println!("{}", n),
                            Value::String(s) => println!("{}", s),
                            Value::Identifier(i) => println!("{}", i),
                            Value::Nil =>println!("nil"),
                            Value::Class(class) => println!("{}", class.to_string()),
                            Value::Instance(instance) => println!("{}", instance.borrow().to_string()),
                            _=> todo!()
                        }
                    },
                    Err(e) =>{
                        eprintln!("{:?}", e);

                    }
                }
                Value::Nil
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
                    scope.exit_scope();
                    return Value::ReturnVal(val)
                } else {
                    scope.exit_scope();
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
                let func = Value::Function {params: params.clone(), body : body.clone(), closure_index : scope.get_current_scope().index, this : None, is_init : false};

                scope.define(&name.lexeme, func.clone());

                func
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
            },
            Stmt::Class { name, methods } => {
                scope.define(&name.lexeme, Value::Nil);

                let mut methods_map : HashMap<String, Value> = HashMap::new();

                for method in methods {
                    let Stmt::Function { name, params : _, body : _ } = method else {
                        panic!("Not a method")
                    };

                    let mut init = false;

                    if name.lexeme == "init" {
                        init = true;
                    }

                    let mut value = self.execute(method, scope, globals);

                    if let Value::Function { params : _, body : _, closure_index : _, this : _, is_init} = &mut value {
                        *is_init = init;
                    };

                    methods_map.insert(name.lexeme.clone(), value);
                }

                let class = NsClass::new(&name.lexeme, methods_map);
                let class = Value::Class(class);
                scope.define(&name.lexeme, class);
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

    pub fn exit_scope(&mut self) {
        self.environments.pop();
    }

    pub fn get_current_scope(&self) -> &Environment {
        self.environments.last().unwrap()
    }

    pub fn get_current_mut_scope(&mut self) ->  &mut Environment {
        self.environments.last_mut().unwrap()
    }

    pub fn get_at(&self, distance : usize, name : &str) -> Option<&Value> {
        // println!("Getting value at dist {} with name {}", distance, name);
        self.environments[distance].values.get(name)
    }

    pub fn assign_at(&mut self, distance : usize, name : &str, val : Value) {
        self.environments[distance].values.insert(name.to_string(), val);
    }

    pub fn from(source : &[Environment]) -> Self {
        Self {
            environments : source.into()
        }
    }

    pub fn evaluate(&mut self, e : &Expr, interpreter : &mut Interpreter, globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError> {
        match e {
            Expr::Literal{value : Value::Nil, index : _} => Ok(Value::Nil),
            Expr::Literal{value : Value::Identifier(i), index : _} => Ok(Value::Identifier(i.to_owned())),
            Expr::Literal{value : Value::String(s), index : _} => Ok(Value::String(s.to_owned())),
            Expr::Literal{value : Value::Number(n), index : _} => Ok(Value::Number(n.to_owned())),
            Expr::Literal{value : Value::Bool(true), index : _} => Ok(Value::Bool(true)),
            Expr::Literal{value: Value::Bool(false), index : _} => Ok(Value::Bool(false)),
            Expr::Grouping{group, index : _} => self.evaluate(group, interpreter, globals),
            Expr::Unary { operator, right, index : _ } => {
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
            Expr::Binary { left, operator, right, index : _ } => {
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
            Expr::Variable {token, index} => {
                // println!("Looking for variable expression: {}", v);
                let dist_or_none = interpreter.look_up_var(index);

                if let Some(dist) = dist_or_none {
                    let val_or_none = self.get_at(*dist, &token.lexeme);

                    if let Some(val) = val_or_none {
                        return Ok(val.clone())
                    }
                }

                interpreter.get(token, self.get_current_scope().index, self)
            },
            Expr::Assign { name, val, index } => {
                let val = self.evaluate(&val, interpreter, globals)?;

                let dist_or_none = interpreter.look_up_var(index);

                if let Some(dist) = dist_or_none {
                    self.assign_at(*dist, &name.lexeme, val);
                } else {
                    interpreter.assign(name, val, self.get_current_scope().index, self)?;
                }

                Ok(Value::Nil)
            },
            Expr::Logical { left, operator, right, index : _ } => {
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
            Expr::Call { callee, paren, args, index : _ } => {
                let res = self.evaluate(callee, interpreter, globals);

                match res {
                    Ok(callee) => {
                        let mut arguments : Vec<Value> = Vec::new();

                        for arg in args {
                            arguments.push(self.evaluate(arg, interpreter, globals)?);
                        }

                        match callee {
                            Value::Function { params : _, body : _, closure_index: _, this : _ , is_init : _} | Value::Class(_)=> (),
                            _ => {
                                println!("Printing callee {:?}", callee);
                                return Err(NewError::Runtime("Callee is not a function", paren.clone()))
                            }
                        };

                        if arguments.len() != callee.arity() {
                            return Err(Runtime("Incorrect number of arguments were passed to this function", paren.clone()))
                        }

                        callee.call(self, interpreter, &arguments, globals)
                    },
                    Err(_) => {
                        if let Expr::Variable{token, index : _} = &**callee {
                            let Some(callable) = globals.get(&token.lexeme) else {
                                return Err(Runtime("Incorrect number of arguments were passed to this function", paren.clone()))
                            };

                            return callable.call(self, interpreter, &vec![], globals)
                        }

                        Ok(Value::Nil)
                    }
                }
            },
            Expr::Get { object, name, index : _ } => {
                let value = self.evaluate(object, interpreter, globals)?;

                match value {
                    Value::Instance(_) => {
                        return value.get(name)
                    },
                    _ => Err(Runtime("Only instances have properties", name.clone()))
                }
            },
            Expr::Set { object, name, value, index : _ } => {
                let object = self.evaluate(object, interpreter, globals)?;

                let Value::Instance(instance) = object else {
                    return Err(Runtime("Only instances have fields", name.clone()))
                };

                let value = self.evaluate(value, interpreter, globals)?;

                instance.borrow_mut().set(name, &value);

                Ok(value)
            },
            Expr::This { keyword, index } => {
                if let Some(depth) = interpreter.look_up_var(index) && let Some(val) = self.get_at(*depth, "this") {
                    return Ok(val.clone())
                } else {
                    return Err(Runtime("Could not find 'this'", keyword.clone()))
                }


            }
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
    },
    Class {
        name : Token,
        methods : Vec<Stmt>
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
