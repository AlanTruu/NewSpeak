use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::class::{ClassInstance, NsClass};
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
        closure_index : usize
    },
    ReturnVal(Box<Value>),
    Class(NsClass),
    Instance(Rc<RefCell<ClassInstance>>)
}

impl NewSpeakCallable for Value {
    fn call(&self, env : &mut Scope, i : &mut Interpreter, arguments : &[Value], globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError> {
        match self {
            Value::Function { params, body, closure_index } => {
                let mut new_scope = Scope::from(&env.environments[..=*closure_index]);

                for i in 0..params.len() {
                    new_scope.define(&params[i].lexeme, arguments[i].clone());
                }

                match i.interpret(body, &mut new_scope, globals) {
                    Value::ReturnVal(value) => {
                        return Ok(*value)
                    },
                    _ => ()
                }


                Ok(Value::Nil)
            },
            Value::Class(class) => {
                let instance = ClassInstance::new(class);

                Ok(Value::Instance(Rc::new(RefCell::new(instance))))
            }
            _ => panic!()
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::Function { params, body : _, closure_index: _ } => params.len(),
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
