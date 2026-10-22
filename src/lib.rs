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

pub mod interpreter {
    use std::path::PathBuf;
    use std::fs;
    use std::collections::HashMap;
    use crate::lex::*;


    #[derive(Debug)]
    pub enum NewError {
        Syntax(usize, &'static str)
    }

    pub fn run_file(path : PathBuf, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        let contents = fs::read_to_string(path).expect("Unable to read file");

        println!("In run_file");

        run(&contents, reserved)
    }

    pub fn run(contents : &str, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        println!("In run");

        scan(contents, reserved)
    }

    fn error(line : usize, message : &'static str) -> Result<(), NewError> {
        report(line, "", message)
    }

    fn report(line : usize, location : &str,  message : &'static str) -> Result<(), NewError> {
        println!("[line {},  ] Error where {}", line, message);

        Err(NewError::Syntax(line, message))
    }
}
