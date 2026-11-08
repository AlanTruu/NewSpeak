use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::class::{ClassInstance, NsClass};
use crate::grammar::Value::Instance;
use crate::statement::{Scope, Stmt};
use crate::{interpreter_utils::NewError, lex::Token};
use crate::statement::{NewSpeakCallable, Interpreter};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name : Token,
        val : Box<Expr>,
        index : usize
    },
    Binary {
        left : Box<Expr>,
        operator : Token,
        right : Box<Expr>,
        index : usize
    },
    Unary {
        operator : Token,
        right : Box<Expr>,
        index : usize
    },
    Grouping{group : Box<Expr>, index : usize},
    Literal {
        value : Value,
        index : usize
    },
    Variable {
        token : Token,
        index : usize
    },
    Logical {
        left : Box<Expr>,
        operator : Token,
        right : Box<Expr>,
        index : usize
    },
    Call {
        callee : Box<Expr>,
        paren : Token,
        args : Vec<Expr>,
        index : usize
    },
    Get {
        object : Box<Expr>,
        name : Token,
        index : usize
    },
    Set {
        object : Box<Expr>,
        name : Token,
        value : Box<Expr>,
        index : usize
    },
    This {
        keyword : Token,
        index : usize
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(OrderedFloat<f64>),
    String(String),
    Bool(bool),
    Nil,
    Identifier(String),
    Function {
        params : Vec<Token>,
        body : Vec<Stmt>,
        closure_index : usize,
        this : Option<Rc<RefCell<ClassInstance>>>,
        is_init : bool
    },
    ReturnVal(Box<Value>),
    Class(NsClass),
    Instance(Rc<RefCell<ClassInstance>>)
}

impl Value {
    pub fn bind(&self, instance : &Rc<RefCell<ClassInstance>>) -> Value {
        match self {
            Value::Function { params, body, closure_index, this : None, is_init } => {
                Value::Function { params : params.clone(), body : body.clone(), closure_index : *closure_index, this: Some(Rc::clone(instance)), is_init : *is_init }
            },
            _ => panic!("Cannot call bind on a non-function value")
        }
    }

    pub fn get(&self, name : &Token) -> Result<Value, NewError> {
        match self {
            Instance(instance) => {
                if let Some(value) = instance.borrow().fields.get(&name.lexeme) {
                    return Ok(value.clone())
                } else if let Some(func) = instance.borrow().class.find_method(&name.lexeme) {
                    let method = func.bind(instance);
                    Ok(method)
                }
                else {
                    Err(NewError::Runtime("Undefined property" , name.clone()))
                }
            }
            _=> panic!("Cannot call get on a non class instance")
        }
    }

    pub fn set(&self, name : &Token, value : &Value) {
        match self {
            Instance(instance) => {
                instance.borrow_mut().set(name, value);
            }
            _=> panic!("Cannot call get on a non class instance")
        }
    }
}

impl NewSpeakCallable for Value {
    fn call(&self, env : &mut Scope, i : &mut Interpreter, arguments : &[Value], globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError> {
        match self {
            Value::Function { params, body, closure_index, this, is_init } => {
                let mut new_scope = Scope::from(&env.environments[..=*closure_index]);

                if let Some(instance) = this {
                    new_scope.define("this", Value::Instance(Rc::clone(instance)));
                }

                for i in 0..params.len() {
                    new_scope.define(&params[i].lexeme, arguments[i].clone());
                }

                if let Value::ReturnVal(value) = i.interpret(body, &mut new_scope, globals) {
                    if *is_init {
                        return Ok(new_scope.get_at(*closure_index, "this").unwrap().clone());
                    }

                    return Ok(*value)
                }

                if *is_init {
                    return Ok(new_scope.get_at(*closure_index, "this").unwrap().clone())
                }

                Ok(Value::Nil)
            },
            Value::Class(class) => {
                let instance = Rc::new(RefCell::new(ClassInstance::new(class)));

                if let Some(val) = class.find_method("init") {
                    return Ok(val.bind(&instance).call(env, i, arguments, globals)?)
                }

                Ok(Value::Instance(instance))
            }
            _ => panic!()
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::Function { params, body : _, closure_index: _, this : _, is_init : _ } => params.len(),
            Value::Class(class) => {
                let Some(init) = class.find_method("init") else {
                    return 0
                };

                init.arity()
            }
            _=> 0
        }
    }
}

pub fn print(e : Expr) -> String {
    match e {
        Expr::Binary {left, operator, right, index : _index} => format!("( {} {} {} )", operator.lexeme, print(*left), print(*right)),
        Expr::Unary { operator, right, index : _index } => format!("( {} {} )", operator.lexeme, print(*right)),
        Expr::Grouping { group, index : _index } => format!("( group {} )", print(*group)),
        Expr::Literal{value : Value::Number(n), index : _index} => format!(" {} ", n),
        Expr::Literal{value : Value::String(s), index : _index} => format!(" {} ", s),
        Expr::Literal{value: Value::Identifier(i), index : _index} => format!(" {} ", i),
        Expr::Literal{value : Value::Bool(true), index : _index} => format!(" truth "),
        Expr::Literal{value : Value::Bool(false), index : _index} => format!(" thoughtCrime "),
        Expr::Literal{value : Value::Nil, index : _index} => format!(" nil "),
        _ => panic!("Unexpected token")
    }
}

pub fn is_equal(left : &Value, right : &Value) -> bool {
    if let Value::Nil = left && let Value::Nil = right {
        true
    } else if let Value::Nil = left {
        false
    } else {
        false
    }
}

pub fn is_truthy(val : &Value) -> bool {
    match val {
        Value::Nil | Value::Bool(false) => false,
        _ => true
    }
}
