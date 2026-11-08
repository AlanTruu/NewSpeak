use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::grammar::Value;
use crate::interpreter_utils::NewError;
use crate::lex::Token;

#[derive(Debug, Clone)]
pub struct NsClass {
    name : String,
    methods : HashMap<String, Value>
}

impl NsClass {
    pub fn new(name : &str, methods : HashMap<String, Value>) -> Self {
        Self {
            name : name.to_string(),
            methods
        }
    }

    pub fn to_string(&self) -> &str {
        &self.name
    }

    pub fn find_method(&self, name : &str) -> Option<&Value> {
        self.methods.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstance {
    pub class : NsClass,
    pub fields : HashMap<String, Value>
}

impl ClassInstance {
    pub fn new(class : &NsClass) -> Self {
        Self {
            class : class.clone(),
            fields : HashMap::new()
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} instance", self.class.to_string())
    }

    pub fn set(&mut self, name : &Token, value : &Value) {
        self.fields.insert(name.lexeme.to_string(), value.clone());
    }
}
