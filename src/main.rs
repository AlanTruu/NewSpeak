/*
    Author: Alan Truong
    Start date: 19/10/26
    Project: NewSpeak Interpreter (Treewalk)
*/


use std::{cell::RefCell, env, rc::Rc};
use NewSpeak::{grammar::print, interpreter_utils, lex::TokenType, parser_g::Parser, statement::{Environment, Interpreter}};
use std::collections::HashMap;


fn main() {

    let args : Vec<String>  = env::args().collect();

    if args.len() == 1 {
        println!("Usage: new_speak [file_path]");
    } else if args.len() > 1 {
        println!("Running file");

        let path = &args[1];

        let mut reserved : HashMap<&str, TokenType> = HashMap::new();
        reserved.insert("and", TokenType::And);
        reserved.insert("class", TokenType::Class);
        reserved.insert("otherwise", TokenType::Otherwise);
        reserved.insert("thoughtCrime", TokenType::ThoughtCrime);
        reserved.insert("for", TokenType::For);
        reserved.insert("func", TokenType::Func);
        reserved.insert("on", TokenType::On);
        reserved.insert("nil", TokenType::Nil);
        reserved.insert("or", TokenType::Or);
        reserved.insert("redact", TokenType::Redact);
        reserved.insert("return", TokenType::Return);
        reserved.insert("super", TokenType::Super);
        reserved.insert("this", TokenType::This);
        reserved.insert("truth", TokenType::Truth);
        reserved.insert("var", TokenType::Var);
        reserved.insert("while", TokenType::While);
        reserved.insert("break", TokenType::Break);

        let (tokens, err) = interpreter_utils::run_file(path.into(), &mut reserved);

        for token in &tokens {
            println!("{}", token)
        }

        let mut p = Parser {tokens : tokens.into_iter().peekable()};

        let statements = p.parse().unwrap();

        let env = Environment {values : HashMap::new(), enclosing : None};

        let mut interpreter = Interpreter {env: Rc::new(RefCell::new(env))};

        interpreter.interpret(&statements);

        for e in err {
            println!("Found error {:?}", e);
        }
    }
}
