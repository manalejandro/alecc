use crate::lexer::{Token, TokenType};
use crate::error::{AleccError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Type {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Bool,
    Pointer(Box<Type>),
    Array(Box<Type>, Option<usize>),
    Function {
        return_type: Box<Type>,
        parameters: Vec<Type>,
        variadic: bool,
    },
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },
    Union {
        name: String,
        fields: Vec<(String, Type)>,
    },
    Enum {
        name: String,
        variants: Vec<(String, i64)>,
    },
    Typedef(String, Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BooleanLiteral(bool),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        member: String,
        is_arrow: bool,
    },
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    Cast {
        target_type: Type,
        expression: Box<Expression>,
    },
    Sizeof(Type),
    Assignment {
        target: Box<Expression>,
        operator: AssignmentOperator,
        value: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Modulo,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    LogicalAnd, LogicalOr,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LeftShift, RightShift,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Plus, Minus, LogicalNot, BitwiseNot,
    PreIncrement, PostIncrement,
    PreDecrement, PostDecrement,
    AddressOf, Dereference,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign, PlusAssign, MinusAssign, MultiplyAssign, DivideAssign, ModuloAssign,
    BitwiseAndAssign, BitwiseOrAssign, BitwiseXorAssign,
    LeftShiftAssign, RightShiftAssign,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Declaration {
        name: String,
        var_type: Type,
        initializer: Option<Expression>,
    },
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    Switch {
        expression: Expression,
        cases: Vec<(Option<Expression>, Vec<Statement>)>,
    },
    Return(Option<Expression>),
    Break,
    Continue,
    Goto(String),
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<(String, Type)>,
    pub body: Statement,
    pub is_inline: bool,
    pub is_static: bool,
    pub is_extern: bool,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub global_variables: Vec<(String, Type, Option<Expression>)>,
    pub type_definitions: HashMap<String, Type>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut functions = Vec::new();
        let mut global_variables = Vec::new();
        let mut type_definitions = HashMap::new();

        while !self.is_at_end() {
            match self.parse_declaration()? {
                Declaration::Function(func) => functions.push(func),
                Declaration::Variable(name, var_type, init) => {
                    global_variables.push((name, var_type, init));
                }
                Declaration::TypeDef(name, type_def) => {
                    type_definitions.insert(name, type_def);
                }
            }
        }

        Ok(Program {
            functions,
            global_variables,
            type_definitions,
        })
    }

    fn parse_declaration(&mut self) -> Result<Declaration> {
        if self.match_token(&TokenType::Typedef) {
            self.parse_typedef()
        } else {
            let storage_class = self.parse_storage_class();
            let base_type = self.parse_type()?;
            
            if self.check(&TokenType::LeftParen) || 
               (self.check(&TokenType::Identifier("".to_string())) && self.peek_ahead(1)?.token_type == TokenType::LeftParen) {
                self.parse_function_declaration(storage_class, base_type)
            } else {
                self.parse_variable_declaration(storage_class, base_type)
            }
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        let mut base_type = match &self.advance()?.token_type {
            TokenType::Void => Type::Void,
            TokenType::Char => Type::Char,
            TokenType::Short => Type::Short,
            TokenType::Int => Type::Int,
            TokenType::Long => Type::Long,
            TokenType::Float => Type::Float,
            TokenType::Double => Type::Double,
            TokenType::Bool => Type::Bool,
            TokenType::Struct => self.parse_struct_type()?,
            TokenType::Union => self.parse_union_type()?,
            TokenType::Enum => self.parse_enum_type()?,
            TokenType::Identifier(name) => {
                // Could be a typedef name
                Type::Typedef(name.clone(), Box::new(Type::Void)) // Placeholder
            }
            _ => {
                return Err(AleccError::ParseError {
                    line: self.current_token()?.line,
                    column: self.current_token()?.column,
                    message: "Expected type specifier".to_string(),
                });
            }
        };

        // Handle pointer declarators
        while self.match_token(&TokenType::Multiply) {
            base_type = Type::Pointer(Box::new(base_type));
        }

        Ok(base_type)
    }

    fn parse_struct_type(&mut self) -> Result<Type> {
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected struct name".to_string(),
            });
        };

        let mut fields = Vec::new();
        
        if self.match_token(&TokenType::LeftBrace) {
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                let field_type = self.parse_type()?;
                let field_name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                    name.clone()
                } else {
                    return Err(AleccError::ParseError {
                        line: self.current_token()?.line,
                        column: self.current_token()?.column,
                        message: "Expected field name".to_string(),
                    });
                };
                
                self.consume(&TokenType::Semicolon, "Expected ';' after field declaration")?;
                fields.push((field_name, field_type));
            }
            
            self.consume(&TokenType::RightBrace, "Expected '}' after struct body")?;
        }

        Ok(Type::Struct { name, fields })
    }

    fn parse_union_type(&mut self) -> Result<Type> {
        // Similar to struct parsing
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected union name".to_string(),
            });
        };

        let mut fields = Vec::new();
        
        if self.match_token(&TokenType::LeftBrace) {
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                let field_type = self.parse_type()?;
                let field_name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                    name.clone()
                } else {
                    return Err(AleccError::ParseError {
                        line: self.current_token()?.line,
                        column: self.current_token()?.column,
                        message: "Expected field name".to_string(),
                    });
                };
                
                self.consume(&TokenType::Semicolon, "Expected ';' after field declaration")?;
                fields.push((field_name, field_type));
            }
            
            self.consume(&TokenType::RightBrace, "Expected '}' after union body")?;
        }

        Ok(Type::Union { name, fields })
    }

    fn parse_enum_type(&mut self) -> Result<Type> {
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected enum name".to_string(),
            });
        };

        let mut variants = Vec::new();
        let mut current_value = 0i64;
        
        if self.match_token(&TokenType::LeftBrace) {
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                let variant_name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                    name.clone()
                } else {
                    return Err(AleccError::ParseError {
                        line: self.current_token()?.line,
                        column: self.current_token()?.column,
                        message: "Expected enum variant name".to_string(),
                    });
                };
                
                if self.match_token(&TokenType::Assign) {
                    if let TokenType::IntegerLiteral(value) = &self.advance()?.token_type {
                        current_value = *value;
                    } else {
                        return Err(AleccError::ParseError {
                            line: self.current_token()?.line,
                            column: self.current_token()?.column,
                            message: "Expected integer literal for enum value".to_string(),
                        });
                    }
                }
                
                variants.push((variant_name, current_value));
                current_value += 1;
                
                if !self.check(&TokenType::RightBrace) {
                    self.consume(&TokenType::Comma, "Expected ',' between enum variants")?;
                }
            }
            
            self.consume(&TokenType::RightBrace, "Expected '}' after enum body")?;
        }

        Ok(Type::Enum { name, variants })
    }

    // Helper methods
    fn current_token(&self) -> Result<&Token> {
        self.tokens.get(self.current).ok_or_else(|| AleccError::ParseError {
            line: 0,
            column: 0,
            message: "Unexpected end of input".to_string(),
        })
    }

    fn advance(&mut self) -> Result<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Result<&Token> {
        self.tokens.get(self.current - 1).ok_or_else(|| AleccError::ParseError {
            line: 0,
            column: 0,
            message: "No previous token".to_string(),
        })
    }

    fn peek_ahead(&self, offset: usize) -> Result<&Token> {
        self.tokens.get(self.current + offset).ok_or_else(|| AleccError::ParseError {
            line: 0,
            column: 0,
            message: "Unexpected end of input".to_string(),
        })
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.tokens.get(self.current).map(|t| &t.token_type), Some(TokenType::Eof))
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.current_token().unwrap().token_type) == 
            std::mem::discriminant(token_type)
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token> {
        if self.check(token_type) {
            self.advance()
        } else {
            Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: message.to_string(),
            })
        }
    }

    // Placeholder implementations for missing methods
    fn parse_storage_class(&mut self) -> StorageClass {
        StorageClass::None // Simplified for now
    }

    fn parse_typedef(&mut self) -> Result<Declaration> {
        let base_type = self.parse_type()?;
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected typedef name".to_string(),
            });
        };
        
        self.consume(&TokenType::Semicolon, "Expected ';' after typedef")?;
        Ok(Declaration::TypeDef(name, base_type))
    }

    fn parse_function_declaration(&mut self, _storage: StorageClass, return_type: Type) -> Result<Declaration> {
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected function name".to_string(),
            });
        };

        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        
        let mut parameters = Vec::new();
        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            let param_type = self.parse_type()?;
            let param_name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                name.clone()
            } else {
                return Err(AleccError::ParseError {
                    line: self.current_token()?.line,
                    column: self.current_token()?.column,
                    message: "Expected parameter name".to_string(),
                });
            };
            
            parameters.push((param_name, param_type));
            
            if !self.check(&TokenType::RightParen) {
                self.consume(&TokenType::Comma, "Expected ',' between parameters")?;
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        
        let body = if self.check(&TokenType::LeftBrace) {
            self.parse_block_statement()?
        } else {
            self.consume(&TokenType::Semicolon, "Expected ';' after function declaration")?;
            Statement::Block(Vec::new()) // Forward declaration
        };

        Ok(Declaration::Function(Function {
            name,
            return_type,
            parameters,
            body,
            is_inline: false,
            is_static: false,
            is_extern: false,
        }))
    }

    fn parse_variable_declaration(&mut self, _storage: StorageClass, var_type: Type) -> Result<Declaration> {
        let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
            name.clone()
        } else {
            return Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected variable name".to_string(),
            });
        };

        let initializer = if self.match_token(&TokenType::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after variable declaration")?;
        
        Ok(Declaration::Variable(name, var_type, initializer))
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}'")?;
        Ok(Statement::Block(statements))
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        // Simplified statement parsing
        if self.match_token(&TokenType::Return) {
            let expr = if !self.check(&TokenType::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(&TokenType::Semicolon, "Expected ';' after return")?;
            Ok(Statement::Return(expr))
        } else {
            let expr = self.parse_expression()?;
            self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        // Simplified expression parsing - just literals and identifiers for now
        match &self.advance()?.token_type {
            TokenType::IntegerLiteral(value) => Ok(Expression::IntegerLiteral(*value)),
            TokenType::FloatLiteral(value) => Ok(Expression::FloatLiteral(*value)),
            TokenType::StringLiteral(value) => Ok(Expression::StringLiteral(value.clone())),
            TokenType::CharLiteral(value) => Ok(Expression::CharLiteral(*value)),
            TokenType::Identifier(name) => Ok(Expression::Identifier(name.clone())),
            _ => Err(AleccError::ParseError {
                line: self.current_token()?.line,
                column: self.current_token()?.column,
                message: "Expected expression".to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum Declaration {
    Function(Function),
    Variable(String, Type, Option<Expression>),
    TypeDef(String, Type),
}

#[derive(Debug, Clone)]
enum StorageClass {
    None,
    Static,
    Extern,
    Auto,
    Register,
}
