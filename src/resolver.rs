use std::collections::HashMap;
use crate::interpreter_utils::NewError;
use crate::statement::{Interpreter, Stmt};
use crate::grammar::Expr;
use crate::lex::Token;

struct Resolver {
    scopes : Vec<HashMap<String, bool>>,
    interpreter : Interpreter
}

impl Resolver {
    pub fn read(&mut self, stmts : &[Stmt]) {
        for statements in stmts {
            self.resolve_statement(statements).unwrap();
        }
    }

    fn resolve_statement(&mut self, stmt : &Stmt) -> Result<(), NewError> {
        match stmt {
            Stmt::Block(block) => {
                self.begin_scope();
                self.read(block);
                self.end_scope();
            },
            Stmt::Var { name : _, initializer } => {
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer)?;
                }
            },
            Stmt::Function { name, params : _, body : _ } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(stmt);
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
            Stmt::Return { keyword : _, value } => {
                if let Some(val) = value {
                    self.resolve_expr(val)?;
                }
            },
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(body)?;
            },
            Stmt::Print(val) => {
                self.resolve_expr(val)?;
            }
            _ => ()
        }

        Ok(())
    }

    fn resolve_function(&mut self, stmt : &Stmt) {
        self.begin_scope();

        match stmt {
            Stmt::Function { name, params, body } => {
                for param in params {
                    self.declare(param);
                    self.define(name);
                }

                self.read(body);
            },
            _ => panic!()
        }

        self.end_scope();
    }

    fn resolve_expr(&mut self, expr : &Expr)-> Result<(), NewError> {
        match expr {
            Expr::Variable(name) => {
                if !self.scopes.is_empty() && let Some(boo) = self.scopes.last().unwrap().get(&name.lexeme)  {
                    if !boo {
                        return Err(NewError::Parse("Can't read local variable in its own initializer", name.clone()))
                    }

                    self.resolve_local(expr, &name.lexeme);
                }
            },
            Expr::Assign { name, val } => {
                self.resolve_expr(val)?;
                self.resolve_local(expr, &name.lexeme);

            },
            Expr::Binary { left, operator : _, right } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;


            },
            Expr::Call { callee, paren : _, args } => {
                self.resolve_expr(callee)?;

                for expr in args {
                    self.resolve_expr(expr)?;
                }

            },
            Expr::Grouping(group) => {
                self.resolve_expr(group)?;
            },
            Expr::Literal(_lit) => {

            },
            Expr::Unary { operator : _, right } => {
                self.resolve_expr(right)?;
            }
            _ => todo!()
        }

        Ok(())
    }

    fn resolve_local(&mut self, expr : &Expr, name : &str) {
        let length = self.scopes.len();

        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(name, length - 1 - i);
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

    pub fn declare(&mut self, name : &Token) {
        if self.scopes.is_empty() {return}

        let scope = self.scopes.last_mut().unwrap();

        scope.insert(name.lexeme.clone(), false);
    }

    pub fn define(&mut self, name : &Token) {
        if self.scopes.is_empty() {return}

        self.scopes.last_mut().unwrap().insert(name.lexeme.clone(), true);
    }
}
