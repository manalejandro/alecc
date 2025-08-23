use crate::error::{AleccError, Result};
use crate::lexer::{Token, TokenType};
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
    #[allow(dead_code)]
    Pointer(Box<Type>),
    #[allow(dead_code)]
    Array(Box<Type>, Option<usize>),
    #[allow(dead_code)]
    Function {
        return_type: Box<Type>,
        parameters: Vec<Type>,
        variadic: bool,
    },
    #[allow(dead_code)]
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },
    #[allow(dead_code)]
    Union {
        name: String,
        fields: Vec<(String, Type)>,
    },
    #[allow(dead_code)]
    Enum {
        name: String,
        variants: Vec<(String, i64)>,
    },
    #[allow(dead_code)]
    Typedef(String, Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    #[allow(dead_code)]
    FloatLiteral(f64),
    StringLiteral(String),
    #[allow(dead_code)]
    CharLiteral(char),
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    Member {
        object: Box<Expression>,
        member: String,
        is_arrow: bool,
    },
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    #[allow(dead_code)]
    Cast {
        target_type: Type,
        expression: Box<Expression>,
    },
    #[allow(dead_code)]
    Sizeof(Type),
    Assignment {
        target: Box<Expression>,
        operator: AssignmentOperator,
        value: Box<Expression>,
    },
    #[allow(dead_code)]
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
    PreIncrement,
    PostIncrement,
    PreDecrement,
    PostDecrement,
    AddressOf,
    Dereference,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign,
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    #[allow(dead_code)]
    ModuloAssign,
    #[allow(dead_code)]
    BitwiseAndAssign,
    #[allow(dead_code)]
    BitwiseOrAssign,
    #[allow(dead_code)]
    BitwiseXorAssign,
    #[allow(dead_code)]
    LeftShiftAssign,
    #[allow(dead_code)]
    RightShiftAssign,
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
    #[allow(dead_code)]
    DoWhile {
        body: Box<Statement>,
        condition: Expression,
    },
    #[allow(dead_code)]
    Switch {
        expression: Expression,
        cases: Vec<(Option<Expression>, Vec<Statement>)>,
    },
    Return(Option<Expression>),
    #[allow(dead_code)]
    Break,
    #[allow(dead_code)]
    Continue,
    #[allow(dead_code)]
    Goto(String),
    #[allow(dead_code)]
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    #[allow(dead_code)]
    pub return_type: Type,
    pub parameters: Vec<(String, Type)>,
    pub body: Statement,
    #[allow(dead_code)]
    pub is_inline: bool,
    #[allow(dead_code)]
    pub is_static: bool,
    #[allow(dead_code)]
    pub is_extern: bool,
    #[allow(dead_code)]
    pub is_variadic: bool,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub global_variables: Vec<(String, Type, Option<Expression>)>,
    #[allow(dead_code)]
    pub type_definitions: HashMap<String, Type>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self { tokens, current: 0 };
        parser.skip_newlines(); // Skip initial newlines
        parser
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

            if self.check(&TokenType::LeftParen)
                || (self.check(&TokenType::Identifier("".to_string()))
                    && self.peek_ahead(1)?.token_type == TokenType::LeftParen)
            {
                self.parse_function_declaration(storage_class, base_type)
            } else {
                self.parse_variable_declaration(storage_class, base_type)
            }
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        // Skip type qualifiers like const, volatile
        while self.match_token(&TokenType::Const) || self.match_token(&TokenType::Volatile) {
            // Just consume the qualifier for now
        }

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
            // Skip const after *
            while self.match_token(&TokenType::Const) || self.match_token(&TokenType::Volatile) {
                // Just consume the qualifier for now
            }
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

                self.consume(
                    &TokenType::Semicolon,
                    "Expected ';' after field declaration",
                )?;
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

                self.consume(
                    &TokenType::Semicolon,
                    "Expected ';' after field declaration",
                )?;
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
                let variant_name = if let TokenType::Identifier(name) = &self.advance()?.token_type
                {
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
        self.tokens
            .get(self.current)
            .ok_or_else(|| AleccError::ParseError {
                line: 0,
                column: 0,
                message: "Unexpected end of input".to_string(),
            })
    }

    fn advance(&mut self) -> Result<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.skip_newlines();
        self.previous()
    }

    fn skip_newlines(&mut self) {
        while !self.is_at_end() {
            if let Ok(token) = self.current_token() {
                if token.token_type == TokenType::Newline {
                    self.current += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn previous(&self) -> Result<&Token> {
        self.tokens
            .get(self.current - 1)
            .ok_or_else(|| AleccError::ParseError {
                line: 0,
                column: 0,
                message: "No previous token".to_string(),
            })
    }

    fn peek_ahead(&self, offset: usize) -> Result<&Token> {
        self.tokens
            .get(self.current + offset)
            .ok_or_else(|| AleccError::ParseError {
                line: 0,
                column: 0,
                message: "Unexpected end of input".to_string(),
            })
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
            || matches!(
                self.tokens.get(self.current).map(|t| &t.token_type),
                Some(TokenType::Eof)
            )
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.current_token().unwrap().token_type)
                == std::mem::discriminant(token_type)
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

    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance().unwrap();
                return true;
            }
        }
        false
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

    fn parse_function_declaration(
        &mut self,
        _storage: StorageClass,
        return_type: Type,
    ) -> Result<Declaration> {
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
        let mut is_variadic = false;

        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            if self.match_token(&TokenType::Ellipsis) {
                is_variadic = true;
                break;
            }

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
            self.advance()?; // Consume the LeftBrace
            self.parse_block_statement()?
        } else {
            self.consume(
                &TokenType::Semicolon,
                "Expected ';' after function declaration",
            )?;
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
            is_variadic,
        }))
    }

    fn parse_variable_declaration(
        &mut self,
        _storage: StorageClass,
        var_type: Type,
    ) -> Result<Declaration> {
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

        self.consume(
            &TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Declaration::Variable(name, var_type, initializer))
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        // Note: LeftBrace was already consumed by match_token in parse_statement
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;
        Ok(Statement::Block(statements))
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        // Try to parse different types of statements
        if self.match_token(&TokenType::Return) {
            let expr = if !self.check(&TokenType::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(&TokenType::Semicolon, "Expected ';' after return")?;
            Ok(Statement::Return(expr))
        } else if self.match_token(&TokenType::If) {
            self.parse_if_statement()
        } else if self.match_token(&TokenType::While) {
            self.parse_while_statement()
        } else if self.match_token(&TokenType::For) {
            self.parse_for_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            self.parse_block_statement()
        } else if self.is_type(&self.current_token()?.token_type) {
            // Variable declaration - convert to Statement format
            let mut var_type = self.parse_type()?;
            let name = if let TokenType::Identifier(name) = &self.advance()?.token_type {
                name.clone()
            } else {
                return Err(AleccError::ParseError {
                    line: self.current_token()?.line,
                    column: self.current_token()?.column,
                    message: "Expected variable name".to_string(),
                });
            };

            // Check for array declaration
            if self.match_token(&TokenType::LeftBracket) {
                let size = if self.check(&TokenType::RightBracket) {
                    None
                } else {
                    // Parse array size (should be a constant expression)
                    let size_expr = self.parse_expression()?;
                    if let Expression::IntegerLiteral(size) = size_expr {
                        Some(size as usize)
                    } else {
                        // For now, just use a default size if not a simple integer
                        Some(10)
                    }
                };
                self.consume(&TokenType::RightBracket, "Expected ']' after array size")?;
                var_type = Type::Array(Box::new(var_type), size);
            }

            let initializer = if self.match_token(&TokenType::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(
                &TokenType::Semicolon,
                "Expected ';' after variable declaration",
            )?;

            Ok(Statement::Declaration {
                name,
                var_type,
                initializer,
            })
        } else {
            // Expression statement
            let expr = self.parse_expression()?;
            self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_stmt = Box::new(self.parse_statement()?);
        let else_stmt = if self.match_token(&TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after while condition")?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'for'")?;

        let init = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };

        if init.is_none() {
            self.advance()?; // consume semicolon
        }

        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenType::Semicolon, "Expected ';' after for condition")?;

        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenType::RightParen, "Expected ')' after for clauses")?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For {
            init,
            condition,
            increment,
            body,
        })
    }

    fn is_type(&self, token_type: &TokenType) -> bool {
        matches!(
            token_type,
            TokenType::Int
                | TokenType::Float
                | TokenType::Double
                | TokenType::Char
                | TokenType::Void
                | TokenType::Short
                | TokenType::Long
                | TokenType::Signed
                | TokenType::Unsigned
        )
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let expr = self.parse_logical_or()?;

        if self.match_token(&TokenType::Assign) {
            let value = self.parse_assignment()?; // Right associative
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                operator: AssignmentOperator::Assign,
                value: Box::new(value),
            });
        } else if self.match_token(&TokenType::PlusAssign) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                operator: AssignmentOperator::PlusAssign,
                value: Box::new(value),
            });
        } else if self.match_token(&TokenType::MinusAssign) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                operator: AssignmentOperator::MinusAssign,
                value: Box::new(value),
            });
        } else if self.match_token(&TokenType::MultiplyAssign) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                operator: AssignmentOperator::MultiplyAssign,
                value: Box::new(value),
            });
        } else if self.match_token(&TokenType::DivideAssign) {
            let value = self.parse_assignment()?;
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                operator: AssignmentOperator::DivideAssign,
                value: Box::new(value),
            });
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenType::LogicalOr) {
            let operator = BinaryOperator::LogicalOr;
            let right = self.parse_logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(&TokenType::LogicalAnd) {
            let operator = BinaryOperator::LogicalAnd;
            let right = self.parse_bitwise_or()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&TokenType::BitwiseOr) {
            let operator = BinaryOperator::BitwiseOr;
            let right = self.parse_bitwise_xor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&TokenType::BitwiseXor) {
            let operator = BinaryOperator::BitwiseXor;
            let right = self.parse_bitwise_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenType::BitwiseAnd) {
            let operator = BinaryOperator::BitwiseAnd;
            let right = self.parse_equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[TokenType::Equal, TokenType::NotEqual]) {
            let operator = match self.previous()?.token_type {
                TokenType::Equal => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_shift()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = match self.previous()?.token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            let right = self.parse_shift()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        while self.match_tokens(&[TokenType::LeftShift, TokenType::RightShift]) {
            let operator = match self.previous()?.token_type {
                TokenType::LeftShift => BinaryOperator::LeftShift,
                TokenType::RightShift => BinaryOperator::RightShift,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = match self.previous()?.token_type {
                TokenType::Minus => BinaryOperator::Subtract,
                TokenType::Plus => BinaryOperator::Add,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_tokens(&[TokenType::Divide, TokenType::Multiply, TokenType::Modulo]) {
            let operator = match self.previous()?.token_type {
                TokenType::Divide => BinaryOperator::Divide,
                TokenType::Multiply => BinaryOperator::Multiply,
                TokenType::Modulo => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        if self.match_tokens(&[
            TokenType::LogicalNot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Increment,
            TokenType::Decrement,
            TokenType::BitwiseAnd,
            TokenType::Multiply,
            TokenType::BitwiseNot,
        ]) {
            let operator = match self.previous()?.token_type {
                TokenType::LogicalNot => UnaryOperator::LogicalNot,
                TokenType::Minus => UnaryOperator::Minus,
                TokenType::Plus => UnaryOperator::Plus,
                TokenType::Increment => UnaryOperator::PreIncrement,
                TokenType::Decrement => UnaryOperator::PreDecrement,
                TokenType::BitwiseAnd => UnaryOperator::AddressOf,
                TokenType::Multiply => UnaryOperator::Dereference,
                TokenType::BitwiseNot => UnaryOperator::BitwiseNot,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(right),
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenType::LeftBracket) {
                // Array indexing
                let index = self.parse_expression()?;
                self.consume(&TokenType::RightBracket, "Expected ']' after array index")?;
                expr = Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&TokenType::Increment) {
                expr = Expression::Unary {
                    operator: UnaryOperator::PostIncrement,
                    operand: Box::new(expr),
                };
            } else if self.match_token(&TokenType::Decrement) {
                expr = Expression::Unary {
                    operator: UnaryOperator::PostDecrement,
                    operand: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Expression::Call {
            function: Box::new(callee),
            arguments,
        })
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        if self.match_token(&TokenType::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        let token = self.advance()?;
        match &token.token_type {
            TokenType::IntegerLiteral(value) => Ok(Expression::IntegerLiteral(*value)),
            TokenType::FloatLiteral(value) => Ok(Expression::FloatLiteral(*value)),
            TokenType::StringLiteral(value) => Ok(Expression::StringLiteral(value.clone())),
            TokenType::CharLiteral(value) => Ok(Expression::CharLiteral(*value)),
            TokenType::Identifier(name) => Ok(Expression::Identifier(name.clone())),
            _ => Err(AleccError::ParseError {
                line: token.line,
                column: token.column,
                message: format!("Expected expression, found {:?}", token.token_type),
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
    #[allow(dead_code)]
    Static,
    #[allow(dead_code)]
    Extern,
    #[allow(dead_code)]
    Auto,
    #[allow(dead_code)]
    Register,
}
