/*
    Author: Alan Truong
    Start date: 19/10/26
    Project: NewSpeak Interpreter (Treewalk)
*/


use std::{env};
use new_speak::{interpreter_utils::{self, define_reserved, get_globals}, lex::TokenType, parser::Parser, statement::{Environment, Interpreter, Scope}};
use std::collections::HashMap;


fn main() {

    let args : Vec<String>  = env::args().collect();

    if args.len() == 1 {
        println!("Usage: new_speak [file_path]");
    } else if args.len() > 1 {
        println!("Running file");

        let path = &args[1];

        let mut reserved : HashMap<&str, TokenType> = define_reserved();

        let (tokens, err) = interpreter_utils::run_file(path.into(), &mut reserved);

        for token in &tokens {
            println!("{}", token)
        }

        let mut p = Parser {tokens : tokens.into_iter().peekable()};

        let statements = p.parse().unwrap();

        let env = Environment {values : HashMap::new(), index : 0};

        let mut interpreter = Interpreter::new();

        let mut scope = Scope {
            environments : vec![env]
        };

        interpreter.interpret(&statements, &mut scope, &get_globals());

        for e in err {
            println!("Found error {:?}", e);
        }
    }
}
