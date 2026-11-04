pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

}

pub mod lex;
pub mod grammar;
pub mod parser;
pub mod statement;
pub mod resolver;

pub mod interpreter_utils {
    use std::path::PathBuf;
    use std::fs;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::statement::NewSpeakCallable;
    use crate::lex::*;
    use crate::grammar::Value;
    use crate::statement::Interpreter;
    use crate::statement::Scope;

    #[derive(Debug, Clone)]
    pub struct Time;

    impl NewSpeakCallable for Time {
        fn call(&self, _env : &mut Scope, _i : &mut Interpreter, _arguments : &[crate::grammar::Value],  _globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<crate::grammar::Value, NewError> {
            let sys_time = SystemTime::now().duration_since(UNIX_EPOCH);
            match sys_time {
                Ok(n) => Ok(Value::Number(n.as_secs_f64())),
                Err(_) => Err(NewError::NativeFunctionFail)
            }
        }

        fn arity(&self) -> usize {
            0
        }
    }

    #[derive(Debug)]
    pub enum NewError {
        Syntax(&'static str, usize),
        Parse(&'static str, Token),
        Runtime(&'static str, Token),
        FillInSemicolon,
        FillInStatement,
        FillInBlock,
        Assignment,
        NativeFunctionFail
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

    pub fn report(line : usize, location : &str,  message : &'static str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
    }

    pub fn get_globals() -> HashMap<String, Box<dyn NewSpeakCallable>> {
        let mut globals = HashMap::new();

        globals.insert("time".into(), Box::new(Time) as Box<dyn NewSpeakCallable>);

        globals
    }

    pub fn define_reserved() -> HashMap<&'static str, TokenType> {
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

        reserved
    }
}
