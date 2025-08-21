use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),

    // Identifiers
    Identifier(String),

    // Keywords
    Auto, Break, Case, Char, Const, Continue, Default, Do,
    Double, Else, Enum, Extern, Float, For, Goto, If,
    Int, Long, Register, Return, Short, Signed, Sizeof, Static,
    Struct, Switch, Typedef, Union, Unsigned, Void, Volatile, While,

    // C++ Keywords
    Bool, Class, Explicit, Export, False, Friend, Inline, Mutable,
    Namespace, New, Operator, Private, Protected, Public, Template,
    This, Throw, True, Try, Typename, Using, Virtual,

    // Operators
    Plus, Minus, Multiply, Divide, Modulo,
    Assign, PlusAssign, MinusAssign, MultiplyAssign, DivideAssign, ModuloAssign,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    LogicalAnd, LogicalOr, LogicalNot,
    BitwiseAnd, BitwiseOr, BitwiseXor, BitwiseNot,
    LeftShift, RightShift, LeftShiftAssign, RightShiftAssign,
    BitwiseAndAssign, BitwiseOrAssign, BitwiseXorAssign,
    Increment, Decrement,
    Arrow, Dot, Question, Colon,

    // Delimiters
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Semicolon, Comma,

    // Preprocessor
    Hash, HashHash,

    // Special
    Eof,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, length: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            length,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::IntegerLiteral(n) => write!(f, "{}", n),
            TokenType::FloatLiteral(n) => write!(f, "{}", n),
            TokenType::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenType::CharLiteral(c) => write!(f, "'{}'", c),
            TokenType::Identifier(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> crate::error::Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }

            let start_line = self.line;
            let start_column = self.column;
            let start_position = self.position;

            match self.scan_token() {
                Ok(Some(token_type)) => {
                    let length = self.position - start_position;
                    tokens.push(Token::new(token_type, start_line, start_column, length));
                }
                Ok(None) => {} // Skip whitespace/comments
                Err(e) => return Err(e),
            }
        }

        tokens.push(Token::new(TokenType::Eof, self.line, self.column, 0));
        Ok(tokens)
    }

    fn scan_token(&mut self) -> crate::error::Result<Option<TokenType>> {
        let c = self.advance();
        
        match c {
            '+' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::PlusAssign))
                } else if self.match_char('+') {
                    Ok(Some(TokenType::Increment))
                } else {
                    Ok(Some(TokenType::Plus))
                }
            }
            '-' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::MinusAssign))
                } else if self.match_char('-') {
                    Ok(Some(TokenType::Decrement))
                } else if self.match_char('>') {
                    Ok(Some(TokenType::Arrow))
                } else {
                    Ok(Some(TokenType::Minus))
                }
            }
            '*' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::MultiplyAssign))
                } else {
                    Ok(Some(TokenType::Multiply))
                }
            }
            '/' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::DivideAssign))
                } else if self.match_char('/') {
                    self.skip_line_comment();
                    Ok(None)
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    Ok(None)
                } else {
                    Ok(Some(TokenType::Divide))
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::Equal))
                } else {
                    Ok(Some(TokenType::Assign))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::NotEqual))
                } else {
                    Ok(Some(TokenType::LogicalNot))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::LessEqual))
                } else if self.match_char('<') {
                    if self.match_char('=') {
                        Ok(Some(TokenType::LeftShiftAssign))
                    } else {
                        Ok(Some(TokenType::LeftShift))
                    }
                } else {
                    Ok(Some(TokenType::Less))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::GreaterEqual))
                } else if self.match_char('>') {
                    if self.match_char('=') {
                        Ok(Some(TokenType::RightShiftAssign))
                    } else {
                        Ok(Some(TokenType::RightShift))
                    }
                } else {
                    Ok(Some(TokenType::Greater))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Some(TokenType::LogicalAnd))
                } else if self.match_char('=') {
                    Ok(Some(TokenType::BitwiseAndAssign))
                } else {
                    Ok(Some(TokenType::BitwiseAnd))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Some(TokenType::LogicalOr))
                } else if self.match_char('=') {
                    Ok(Some(TokenType::BitwiseOrAssign))
                } else {
                    Ok(Some(TokenType::BitwiseOr))
                }
            }
            '^' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::BitwiseXorAssign))
                } else {
                    Ok(Some(TokenType::BitwiseXor))
                }
            }
            '~' => Ok(Some(TokenType::BitwiseNot)),
            '%' => {
                if self.match_char('=') {
                    Ok(Some(TokenType::ModuloAssign))
                } else {
                    Ok(Some(TokenType::Modulo))
                }
            }
            '(' => Ok(Some(TokenType::LeftParen)),
            ')' => Ok(Some(TokenType::RightParen)),
            '{' => Ok(Some(TokenType::LeftBrace)),
            '}' => Ok(Some(TokenType::RightBrace)),
            '[' => Ok(Some(TokenType::LeftBracket)),
            ']' => Ok(Some(TokenType::RightBracket)),
            ';' => Ok(Some(TokenType::Semicolon)),
            ',' => Ok(Some(TokenType::Comma)),
            '.' => Ok(Some(TokenType::Dot)),
            '?' => Ok(Some(TokenType::Question)),
            ':' => Ok(Some(TokenType::Colon)),
            '#' => {
                if self.match_char('#') {
                    Ok(Some(TokenType::HashHash))
                } else {
                    Ok(Some(TokenType::Hash))
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Some(TokenType::Newline))
            }
            '"' => self.scan_string(),
            '\'' => self.scan_char(),
            _ => {
                if c.is_ascii_digit() {
                    self.scan_number()
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.scan_identifier()
                } else {
                    Err(crate::error::AleccError::LexError {
                        line: self.line,
                        column: self.column - 1,
                        message: format!("Unexpected character: '{}'", c),
                    })
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.position += 1;
        self.column += 1;
        c
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input.chars().nth(self.position).unwrap_or('\0')
        }
    }

    fn peek(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input.chars().nth(self.position + 1).unwrap_or('\0')
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> crate::error::Result<()> {
        while !self.is_at_end() {
            if self.current_char() == '*' && self.peek() == '/' {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }
        
        Err(crate::error::AleccError::LexError {
            line: self.line,
            column: self.column,
            message: "Unterminated block comment".to_string(),
        })
    }

    fn scan_string(&mut self) -> crate::error::Result<Option<TokenType>> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = match self.current_char() {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '0' => '\0',
                        c => c,
                    };
                    value.push(escaped);
                    self.advance();
                }
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        
        if self.is_at_end() {
            return Err(crate::error::AleccError::LexError {
                line: self.line,
                column: self.column,
                message: "Unterminated string literal".to_string(),
            });
        }
        
        self.advance(); // consume closing '"'
        Ok(Some(TokenType::StringLiteral(value)))
    }

    fn scan_char(&mut self) -> crate::error::Result<Option<TokenType>> {
        if self.is_at_end() {
            return Err(crate::error::AleccError::LexError {
                line: self.line,
                column: self.column,
                message: "Unterminated character literal".to_string(),
            });
        }
        
        let c = if self.current_char() == '\\' {
            self.advance();
            if self.is_at_end() {
                return Err(crate::error::AleccError::LexError {
                    line: self.line,
                    column: self.column,
                    message: "Unterminated character literal".to_string(),
                });
            }
            match self.current_char() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '0' => '\0',
                c => c,
            }
        } else {
            self.current_char()
        };
        
        self.advance();
        
        if self.is_at_end() || self.current_char() != '\'' {
            return Err(crate::error::AleccError::LexError {
                line: self.line,
                column: self.column,
                message: "Unterminated character literal".to_string(),
            });
        }
        
        self.advance(); // consume closing '\''
        Ok(Some(TokenType::CharLiteral(c)))
    }

    fn scan_number(&mut self) -> crate::error::Result<Option<TokenType>> {
        let start = self.position - 1;
        
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            self.advance();
        }
        
        let mut is_float = false;
        if !self.is_at_end() && self.current_char() == '.' && self.peek().is_ascii_digit() {
            is_float = true;
            self.advance(); // consume '.'
            
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                self.advance();
            }
        }
        
        let text = &self.input[start..self.position];
        
        if is_float {
            match text.parse::<f64>() {
                Ok(value) => Ok(Some(TokenType::FloatLiteral(value))),
                Err(_) => Err(crate::error::AleccError::LexError {
                    line: self.line,
                    column: self.column,
                    message: format!("Invalid float literal: {}", text),
                }),
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => Ok(Some(TokenType::IntegerLiteral(value))),
                Err(_) => Err(crate::error::AleccError::LexError {
                    line: self.line,
                    column: self.column,
                    message: format!("Invalid integer literal: {}", text),
                }),
            }
        }
    }

    fn scan_identifier(&mut self) -> crate::error::Result<Option<TokenType>> {
        let start = self.position - 1;
        
        while !self.is_at_end() {
            let c = self.current_char();
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let text = &self.input[start..self.position];
        let token_type = match text {
            "auto" => TokenType::Auto,
            "break" => TokenType::Break,
            "case" => TokenType::Case,
            "char" => TokenType::Char,
            "const" => TokenType::Const,
            "continue" => TokenType::Continue,
            "default" => TokenType::Default,
            "do" => TokenType::Do,
            "double" => TokenType::Double,
            "else" => TokenType::Else,
            "enum" => TokenType::Enum,
            "extern" => TokenType::Extern,
            "float" => TokenType::Float,
            "for" => TokenType::For,
            "goto" => TokenType::Goto,
            "if" => TokenType::If,
            "int" => TokenType::Int,
            "long" => TokenType::Long,
            "register" => TokenType::Register,
            "return" => TokenType::Return,
            "short" => TokenType::Short,
            "signed" => TokenType::Signed,
            "sizeof" => TokenType::Sizeof,
            "static" => TokenType::Static,
            "struct" => TokenType::Struct,
            "switch" => TokenType::Switch,
            "typedef" => TokenType::Typedef,
            "union" => TokenType::Union,
            "unsigned" => TokenType::Unsigned,
            "void" => TokenType::Void,
            "volatile" => TokenType::Volatile,
            "while" => TokenType::While,
            // C++ keywords
            "bool" => TokenType::Bool,
            "class" => TokenType::Class,
            "explicit" => TokenType::Explicit,
            "export" => TokenType::Export,
            "false" => TokenType::False,
            "friend" => TokenType::Friend,
            "inline" => TokenType::Inline,
            "mutable" => TokenType::Mutable,
            "namespace" => TokenType::Namespace,
            "new" => TokenType::New,
            "operator" => TokenType::Operator,
            "private" => TokenType::Private,
            "protected" => TokenType::Protected,
            "public" => TokenType::Public,
            "template" => TokenType::Template,
            "this" => TokenType::This,
            "throw" => TokenType::Throw,
            "true" => TokenType::True,
            "try" => TokenType::Try,
            "typename" => TokenType::Typename,
            "using" => TokenType::Using,
            "virtual" => TokenType::Virtual,
            _ => TokenType::Identifier(text.to_string()),
        };
        
        Ok(Some(token_type))
    }
}
