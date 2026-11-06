/*
    Author: Alan Truong
    Start date: 19/10/26
    Project: NewSpeak Interpreter (Treewalk)
*/


use std::{env};
use new_speak::{interpreter_utils::{self, NewError, error, get_globals, get_reserved, report}, lex::TokenType, parser::Parser, resolver::Resolver, statement::{Environment, Interpreter, Scope}};
use std::collections::HashMap;
use new_speak::resolver;


fn main() {

    let args : Vec<String>  = env::args().collect();

    if args.len() == 1 {
        println!("Usage: new_speak [file_path]");
    } else if args.len() > 1 {
        println!("Running file");

        let path = &args[1];

        let mut reserved : HashMap<&str, TokenType> = get_reserved();

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

        let mut resolver = Resolver::new(&mut interpreter);

        if let Err(NewError::Runtime(message, token)) = resolver.read(&statements) {
            error(&token, message);
        }

        interpreter.interpret(&statements, &mut scope, &get_globals());

        // println!("Interpreter locals: {:?}", interpreter.locals);


        for e in err {
            println!("Found error {:?}", e);
        }
    }
}
