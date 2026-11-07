use std::collections::HashMap;
use crate::interpreter_utils::NewError;
use crate::statement::{Interpreter, Stmt};
use crate::grammar::{Expr};
use crate::lex::Token;

#[derive(Debug, Clone, Copy)]
pub enum FunctionType {
    Func,
    NaF,
    Method
}

pub struct Resolver<'a> {
    scopes : Vec<HashMap<String, bool>>,
    interpreter : &'a mut Interpreter,
    current_func : FunctionType
}

impl <'a>Resolver<'a> {
    pub fn new(interpreter : &'a mut Interpreter) -> Self {
        Self {
            scopes : vec![HashMap::new()],
            current_func : FunctionType::NaF,
            interpreter
        }
    }

    pub fn read(&mut self, stmts : &[Stmt]) -> Result<(), NewError> {
        for statements in stmts {
            self.resolve_statement(statements)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt : &Stmt) -> Result<(), NewError> {
        match stmt {
            Stmt::Block(block) => {
                self.begin_scope();
                self.read(block)?;
                self.end_scope();
            },
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(name);
            },
            Stmt::Function { name, params : _, body : _ } => {
                self.declare(name)?;
                self.define(name);

                self.resolve_function(stmt, FunctionType::Func)?;
            },
            Stmt::Expression(expr) => {
                self.resolve_expr(expr)?;
            },
            Stmt::If { condition, on, otherwise } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(on)?;

                if let Some(otherwise_stmt) = otherwise {
                    self.resolve_statement(otherwise_stmt)?;
                }
            },
            Stmt::Return { keyword , value } => {
                if let FunctionType::NaF = self.current_func {
                    return Err(NewError::Runtime("Cannot return from top-level code", keyword.clone()))
                }

                if let Some(val) = value {
                    self.resolve_expr(val)?;
                }
            },
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(body)?;
            },
            Stmt::Print(val) => {
                // println!("resolving print with expression: {:?} \n", val);
                self.resolve_expr(val)?;
            },
            Stmt::Class { name, methods } => {
                self.declare(name)?;
                self.define(name);

                for method in methods {
                    let declaration = FunctionType::Method;

                    self.resolve_function(method, declaration)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_function(&mut self, stmt : &Stmt, func_type : FunctionType) -> Result<(), NewError> {
        // self.begin_scope();

        match stmt {
            Stmt::Function { name : _, params, body } => {
                let func = self.current_func;
                self.current_func = func_type;

                for param in params {
                    self.declare(param)?;
                    self.define(param);
                }

                self.read(body)?;
                self.current_func = func;
            },
            _ => panic!("Not a function")
        }

        // self.end_scope();

        Ok(())
    }

    fn resolve_expr(&mut self, expr : &Expr) -> Result<(), NewError> {
        match expr {
            Expr::Variable{token, index } => {
                if !self.scopes.is_empty() && let Some(boo) = self.scopes.last().unwrap().get(&token.lexeme)  {
                    if !boo {
                        return Err(NewError::Parse("Can't read local variable in its own initializer", token.clone()))
                    }
                }

                // println!("In resolve_expr with expr {:?}", expr);

                self.resolve_local(&token.lexeme, *index);
            },
            Expr::Assign { name, val, index } => {
                self.resolve_expr(val)?;
                self.resolve_local(&name.lexeme, *index);
            },
            Expr::Binary { left, operator : _, right, index : _ } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            },
            Expr::Call { callee, paren : _, args, index : _ } => {
                self.resolve_expr(callee)?;

                for expr in args {
                    self.resolve_expr(expr)?;
                }
            },
            Expr::Grouping{group, index : _} => {
                self.resolve_expr(group)?;
            },
            Expr::Literal{value : _, index : _} => {

            },
            Expr::Unary { operator : _, right, index : _ } => {
                self.resolve_expr(right)?;
            },
            Expr::Logical { left, operator : _, right, index : _ } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            },
            Expr::Get { object, name : _ , index : _} => {
                self.resolve_expr(object)?;
            },
            Expr::Set { object, name : _, value, index : _ } => {
                self.resolve_expr(value)?;
                self.resolve_expr(object)?;
            }
        }

        Ok(())
    }

    fn resolve_local(&mut self, name : &str, index : usize) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                self.interpreter.resolve(index, i);
                return
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name : &Token) -> Result<(), NewError> {
        if self.scopes.is_empty() {return Ok(())}

        let scope = self.scopes.last_mut().unwrap();

        if let Some(_var) = scope.get(&name.lexeme) {
            return Err(NewError::Runtime("Already a variable with that name", name.clone()))
        }

        scope.insert(name.lexeme.clone(), false);

        Ok(())
    }

    pub fn define(&mut self, name : &Token) {
        if self.scopes.is_empty() {return}

        self.scopes.last_mut().unwrap().insert(name.lexeme.clone(), true);
    }
}
