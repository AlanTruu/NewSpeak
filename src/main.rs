/*
    Author: Alan Truong
    Start date: 19/10/26
    Project: NewSpeak Interpreter (Treewalk)
*/


use std::env;
use NewSpeak::{grammar::print, interpreter, lex::TokenType, parser_g::Parser};
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

        let (tokens, err) = interpreter::run_file(path.into(), &mut reserved);

        for token in &tokens {
            println!("{}", token)
        }

        let mut p = Parser {tokens : tokens.into_iter().peekable(), current : 0};

        match p.parse() {
            Ok(expr) => {
                let tree = print(expr);

                println!("{}", tree)
            }
            Err(e) => eprintln!("{:?}", e)
        }

        match p.parse() {
            Ok(expr) => {
                let tree = print(expr);

                println!("{}", tree)
            }
            Err(e) => eprintln!("{:?}", e)
        }

        for e in err {
            println!("Found error {:?}", e);
        }
    }
}
