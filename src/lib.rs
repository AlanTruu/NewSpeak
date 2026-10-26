pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::TokenType;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn get_tokens() {
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


    }
}

pub mod lex;
pub mod syntax;
pub mod grammar;
pub mod parser;
pub mod parser_g;
pub mod statement;

pub mod interpreter_utils {
    use std::path::PathBuf;
    use std::fs;
    use std::collections::HashMap;
    use crate::lex::*;


    #[derive(Debug)]
    pub enum NewError {
        Syntax(usize, &'static str),
        Parse(&'static str, Token),
        Runtime(&'static str, Token),
        MissingSemicolon
    }

    pub fn run_file(path : PathBuf, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        let contents = fs::read_to_string(path).expect("Unable to read file");

        run(&contents, reserved)
    }

    pub fn run(contents : &str, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        scan(contents, reserved)
    }

    pub fn error(token : &Token, message : &'static str) {
        match token.t {
            TokenType::EOF => report(token.line, " at end ", message),
            _ =>  report(token.line, &format!(" at '{}'", token.lexeme), message),
        }
    }

    fn report(line : usize, location : &str,  message : &'static str) {
        eprintln!("[line {},  ] Error {}: {}", line, location, message);
    }
}
