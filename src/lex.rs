use crate::{interpreter::NewError};
use std::collections::HashMap;
use std::fmt::{self};

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
    pub t : TokenType,
    pub lexeme : String,
    pub line : usize
}

impl Token {
    pub fn new(t : TokenType, lexeme : String, line : usize) -> Self {
        Self {
            t,
            lexeme,
            line
        }
    }

    pub fn get_lexem(&self) -> &str {
        &self.lexeme
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type : {:?}, Lexeme: {}", self.t,  self.lexeme)
    }
}



// scan file
pub fn scan(source : &str, reserved: &mut HashMap<&str, TokenType>) -> (Vec<Token>, Vec<NewError>) {
    let mut tokens : Vec<Token> = Vec::new();
    let mut errors : Vec<NewError> = Vec::new();

    // Iterate source line by line, char by char
    for (line_number, line) in source.lines().enumerate() {

        let mut chars = line.chars().peekable();

        loop {
            let Some(c) = chars.next() else {
                break;
            };

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

                    while let Some(next) = chars.peek() && next.is_ascii_alphanumeric() {
                        let ch = chars.next().unwrap();
                        str.push(ch);
                    }
                    let lexeme = str.clone();

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
