use std::collections::HashMap;
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

    fn find_method(&self, name : &str) -> Option<&Value> {
        self.methods.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstance {
    class : NsClass,
    fields : HashMap<String, Value>
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

    pub fn get(&self, name : &Token) -> Result<Value, NewError> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone())
        } else if let Some(value) = self.class.find_method(&name.lexeme){
            return Ok(value.clone())
        } else {
            Err(NewError::Runtime("Undefined property" , name.clone()))
        }
    }

    pub fn set(&mut self, name : &Token, value : &Value) {
        self.fields.insert(name.lexeme.to_string(), value.clone());
    }
}
