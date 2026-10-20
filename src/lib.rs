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

pub mod interpreter {
    use std::path::PathBuf;
    use std::fs;
    use std::collections::HashMap;
    use crate::lex::scan;
    use crate::token::*;


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

pub mod token {
    use std::fmt::{self, Formatter};

    #[derive(Debug, Clone)]
    pub enum TokenType {

        // Single character tokens
        LeftParen,
        RightParen,
        LeftBrace,
        RightBrace,
        Comma,
        Dot,
        Minus,
        Plus,
        Semicolon,
        Slash,
        Star,

        // Double character tokens
        Bang,
        BangEqual,
        Equal,
        EqualEqual,
        Greater,
        GreaterEqual,
        Less,
        LessEqual,

        // Literals
        Identifier(String),
        String(String),
        Number(f64),

        // keywords
        And,
        Class,
        Otherwise, // Else
        ThoughtCrime, // False
        Func,
        For,
        On, // If
        Nil,
        Or,
        Redact, // print
        Return,
        Super,
        This,
        Truth, // True
        Var,
        While,

        //
        UnexpectedChar,

        EOF
    }

    #[derive(Debug)]
    pub struct Token {
        t : TokenType,
        lexeme : String,
        line : usize
    }

    impl Token {
        pub fn new(t : TokenType, lexeme : String, line : usize) -> Self {
            Self {
                t,
                lexeme,
                line
            }
        }
    }

    impl fmt::Display for Token {
        fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Type : {:?}, Lexeme: {}", self.t,  self.lexeme)
        }
    }
}

mod lex {
    use crate::{interpreter::NewError, token::{Token, TokenType}};
    use std::collections::HashMap;

    // scan file
    pub fn scan(source : &str, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
        let mut tokens : Vec<Token> = Vec::new();
        let mut errors : Vec<NewError> = Vec::new();

        // Iterate source line by line, char by char
        for (line_number, line) in source.lines().enumerate() {
            println!("{}", &line);
            let mut chars = line.chars().peekable();

            loop {
                let Some(c) = chars.next() else {
                    break;
                };

                println!("After break: {}", &c);

                let (t, lexeme) = match c {
                    '(' => (TokenType::LeftParen, c.to_string()),
                    ')' =>( TokenType::RightParen, c.to_string()),
                    '{' => (TokenType::LeftBrace, c.to_string()),
                    '}' => (TokenType::RightBrace, c.to_string()),
                    ',' => (TokenType::Comma, c.to_string()),
                    '.' => (TokenType::Dot, c.to_string()),
                    '-' =>( TokenType::Minus, c.to_string()),
                    '+' => (TokenType::Plus, c.to_string()),
                    ';' => (TokenType::Semicolon, c.to_string()),
                    '*' => (TokenType::Star, c.to_string()),
                    '!' => {
                        if let Some(ch) = chars.peek() && *ch == '='{
                            chars.next(); //consume the equal sign
                            (TokenType::BangEqual, "!=".to_string())
                        } else {
                            (TokenType::Bang, "!".to_string())
                        }
                    },
                    '=' => {
                        if let Some(ch) = chars.peek() && *ch == '='{
                            chars.next();
                            (TokenType::EqualEqual, "==".to_string())
                        } else {
                            (TokenType::Equal, "=".to_string())
                        }
                    },
                    '<' => {
                        if let Some(ch) = chars.peek() && *ch == '='{
                            chars.next();
                            (TokenType::LessEqual, "<=".to_string())
                        } else {
                            (TokenType::Less, "<".to_string())
                        }
                    },
                    '>' => {
                        if let Some(ch) = chars.peek() && *ch == '='{
                         chars.next();
                         (TokenType::GreaterEqual, ">=".to_string())
                        } else {
                            (TokenType::Greater, ">".to_string())
                        }
                    },
                    '/' => {
                        // if a comment is encountered, just skip the whole line
                        if let Some(ch) = chars.peek() && *ch == '/' {
                            break;
                        }

                        (TokenType::Slash, "/".to_string())
                    },
                    ' ' | '\t' | '\r' => continue,
                    '\n' => break,
                    '"' => {
                        let mut str = String::new();
                        let mut closing = false;

                        while let Some(next) = chars.peek() {
                            let ch = chars.next().unwrap();
                            if ch == '"' {
                                closing = true;
                                break
                            }
                            str.push(ch);
                        }

                        // use closing flag to notify user that the string literal was never closed properly
                        // no multi-line strings in this interpreter, that would be a thoughtcrime
                        if !closing {
                            errors.push(NewError::Syntax(line_number, "Closing \" not found"));
                            continue
                        }

                        let lexeme = str.clone();
                        (TokenType::String(str), lexeme)
                    },
                    'a'..='z' | 'A'..='Z' => {
                        let mut str = String::from(c);

                        while let Some(next) = chars.peek() && next.is_alphabetic() {
                            let ch = chars.next().unwrap();
                            println!("Processing char: {}", &ch);
                            str.push(ch);
                        }
                        let lexeme = str.clone();

                        println!("Peek at lexeme: {}", &lexeme);

                        if reserved.contains_key(&lexeme as &str) {
                            (reserved[&lexeme as &str].to_owned(), lexeme)
                        } else {
                            (TokenType::Identifier(str), lexeme)
                         }



                    },
                    '0'..='9' => {
                        let mut num = String::from(c);

                         while let Some(next) = chars.peek() && (next.is_numeric() || *next == '.') {
                            let digit = chars.next().unwrap();

                            num.push(digit);
                        }

                        let lexeme = num.clone();

                        let float = num.parse::<f64>();

                        if let Ok(f) = float {
                            (TokenType::Number(f), lexeme)
                        } else {
                            errors.push(NewError::Syntax(line_number, "Not a number: "));
                            continue
                        }
                    },
                    _ => (TokenType::UnexpectedChar, c.to_string())
                };

               if let TokenType::UnexpectedChar = t {
                   errors.push(NewError::Syntax(line_number, "Unexpected character at line: "))
               }

               let token = Token::new(t, lexeme, line_number);
               tokens.push(token);
           }
        }

        (tokens, errors)
    }
}
