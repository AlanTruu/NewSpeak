use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::statement::{Environment, Stmt};
use crate::{interpreter_utils::NewError, lex::Token};
use crate::statement::{NewSpeakCallable, Interpreter};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name : Token,
        val : Box<Expr>
    },
    Binary {
        left : Box<Expr>,
        operator : Token,
        right : Box<Expr>
    },
    Unary {
        operator : Token,
        right : Box<Expr>
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Variable(Token),
    Logical {
        left : Box<Expr>,
        operator : Token,
        right : Box<Expr>
    },
    Call {
        callee : Box<Expr>,
        paren : Token,
        args : Vec<Expr>
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Identifier(String),
    Function {
        params : Vec<Token>,
        body : Vec<Stmt>
    },
}

impl NewSpeakCallable for Value {
    fn call(&self, env : &Environment, i : &mut Interpreter, arguments : &[Value]) -> Result<Value, NewError> {
        match self {
            Value::Function { params, body } => {
                let scope = Environment {values : HashMap::new(), globals : Rc::clone(&env.globals), enclosing : None};

                let env = Rc::new(RefCell::new(scope));

                for i in 0..params.len() {
                    env.borrow_mut().define(&params[i].lexeme, arguments[i].clone());
                }

                i.interpret(body, env);

                Ok(Value::Nil)
            },
            _ => panic!()
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::Function { params, body } => params.len(),
            _=> 0
        }
    }
}

pub fn print(e : Expr) -> String {
    match e {
        Expr::Binary {left, operator, right} => format!("( {} {} {} )", operator.lexeme, print(*left), print(*right)),
        Expr::Unary { operator, right } => format!("( {} {} )", operator.lexeme, print(*right)),
        Expr::Grouping(expr) => format!("( group {} )", print(*expr)),
        Expr::Literal(Value::Number(n)) => format!(" {} ", n),
        Expr::Literal(Value::String(s)) => format!(" {} ", s),
        Expr::Literal(Value::Identifier(i)) => format!(" {} ", i),
        Expr::Literal(Value::Bool(true)) => format!(" truth "),
        Expr::Literal(Value::Bool(false)) => format!(" thoughtCrime "),
        Expr::Literal(Value::Nil) => format!(" nil "),
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
