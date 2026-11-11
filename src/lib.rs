#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

use crate::interpreter_utils::{get_reserved, get_globals};
    use crate::parser::Parser;
    use crate::statement::{Interpreter, Environment, Scope};
    use crate::interpreter_utils::run;
    use crate::resolver::Resolver;
    use crate::grammar::Value;
    use std::assert_matches;
    use std::collections::HashMap;

    #[test]
    fn resolve_dbg() {
        let contento =
            "var i = 0;
            while (truth) {
                redact i;
                i = i + 1;
            }";

        let mut reserved = get_reserved();

        let (tokens, _errs) = run(contento, &mut reserved);

        let mut p = Parser {tokens : tokens.into_iter().peekable(), index : 0};

        let statements = p.parse().unwrap();

        let env = Environment {values : HashMap::new(), index : 0};

        let mut interpreter = Interpreter::new();

        let mut scope = Scope {
            environments : vec![env]
        };

        let mut resolver = Resolver::new(&mut interpreter);

        resolver.read(&statements).unwrap();

        println!("Interpreter locals: {:?}", interpreter.locals);

        let _val = interpreter.interpret(&statements, &mut scope, &get_globals());

        assert_matches!(Value::Number(OrderedFloat(100.0)), _val);
    }

}

pub mod lex;
pub mod grammar;
pub mod parser;
pub mod statement;
pub mod resolver;
pub mod class;

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
    use ordered_float::OrderedFloat;

    #[derive(Debug, Clone)]
    pub struct Time;

    impl NewSpeakCallable for Time {
        fn call(&self, _env : &mut Scope, _i : &mut Interpreter, _arguments : &[crate::grammar::Value],  _globals : &HashMap<String, Box<dyn NewSpeakCallable>>) -> Result<crate::grammar::Value, NewError> {
            let sys_time = SystemTime::now().duration_since(UNIX_EPOCH);
            match sys_time {
                Ok(n) => Ok(Value::Number(OrderedFloat(n.as_secs_f64()))),
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

    pub fn run_file(path : PathBuf, reserved: &HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        let contents = fs::read_to_string(path).expect("Unable to read file");

        run(&contents, reserved)
    }

    pub fn run(contents : &str, reserved: &HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
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

    /// Builds a map of global values such as FFI function time.
    pub fn get_globals() -> HashMap<String, Box<dyn NewSpeakCallable>> {
        let mut globals = HashMap::new();

        globals.insert("time".into(), Box::new(Time) as Box<dyn NewSpeakCallable>);

        globals
    }

    /// Buids a map of reserved words. String literals match to the corresponding TokenType.
    pub fn get_reserved() -> HashMap<&'static str, TokenType> {
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

    pub fn start() {}
}
