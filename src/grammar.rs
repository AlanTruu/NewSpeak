use std::collections::HashMap;
use crate::statement::{Scope, Stmt};
use crate::{interpreter_utils::NewError, lex::Token};
use crate::statement::{NewSpeakCallable, Interpreter};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    ReturnVal(Box<Value>)
}

impl NewSpeakCallable for Value {
    fn call(&self, env : &mut Scope, i : &mut Interpreter, arguments : &[Value], globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<Value, NewError> {
        match self {
            Value::Function { params, body, closure_index } => {
                let mut new_scope = Scope::from(&env.environments[..=*closure_index]);

                for i in 0..params.len() {
                    new_scope.define(&params[i].lexeme, arguments[i].clone());
                }

                // println!("Interpreter locals in call() {:?}", i.locals);
                // println!("With body {:?}", body);

                match i.interpret(body, &mut new_scope, globals) {
                    Value::ReturnVal(value) => {
                        return Ok(*value)
                    },
                    _ => ()
                }


                Ok(Value::Nil)
            },
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
        Expr::Binary {left, operator, right, index} => format!("( {} {} {} )", operator.lexeme, print(*left), print(*right)),
        Expr::Unary { operator, right, index } => format!("( {} {} )", operator.lexeme, print(*right)),
        Expr::Grouping { group, index } => format!("( group {} )", print(*group)),
        Expr::Literal{value : Value::Number(n), index} => format!(" {} ", n),
        Expr::Literal{value : Value::String(s), index} => format!(" {} ", s),
        Expr::Literal{value: Value::Identifier(i), index} => format!(" {} ", i),
        Expr::Literal{value : Value::Bool(true), index} => format!(" truth "),
        Expr::Literal{value : Value::Bool(false), index} => format!(" thoughtCrime "),
        Expr::Literal{value : Value::Nil, index} => format!(" nil "),
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
