use std::collections::HashMap;
use crate::grammar::Value;
use crate::lex::Token;

#[derive(Debug, Clone)]
pub struct NsClass {
    name : String,
    methods : HashMap<String, Value>,
    super_class : Option<Box<NsClass>>
}

impl NsClass {
    pub fn new(name : &str, methods : HashMap<String, Value>, super_class : Option<NsClass>) -> Self {
        Self {
            name : name.to_string(),
            methods,
            super_class : if let Some(class) = super_class {Some(Box::new(class))} else {None}
        }
    }

    pub fn to_string(&self) -> &str {
        &self.name
    }

    pub fn find_method(&self, name : &str) -> Option<&Value> {
        let sub = self.methods.get(name);

        if let Some(ref class) = self.super_class {
            return class.find_method(name)
        }

        sub
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
