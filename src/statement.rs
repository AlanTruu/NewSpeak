use crate::grammar::Expr;
use crate::grammar::Value;
use crate::grammar::evaluate;

pub enum Stmt {
    Print(Expr),
    Expression(Expr)
}

pub fn execute(stmt : &Stmt) {
    match stmt {
        Stmt::Print(expr) => {
            match evaluate(expr) {
                Ok(val) => {
                    match val {
                        Value::Bool(false) => println!("thoughtCrime"),
                        Value::Bool(true) => println!("truth"),
                        Value::Number(n) => println!("{}", n),
                        Value::String(s) => println!("{}", s),
                        Value::Identifier(i) => println!("{}", i),
                        Value::Nil => println!("Nil")
                    }
                },
                Err(e) => eprintln!("{:?}", e)
            }
        },
        Stmt::Expression(expr) => {
            match evaluate(expr) {
                Ok(val) => (),
                Err(e) => eprintln!("{:?}", e)
            }
        }
    }
}

pub fn interpret(statements : &[Stmt]) {
    for statement in statements {
        execute(statement);
    }
}
