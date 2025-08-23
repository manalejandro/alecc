#[cfg(test)]
mod tests {
    use alecc::cli::Args;
    use alecc::codegen::CodeGenerator;
    use alecc::compiler::Compiler;
    use alecc::lexer::{Lexer, TokenType};
    use alecc::parser::Parser;
    use alecc::targets::Target;
    use std::path::PathBuf;

    #[test]
    fn test_lexer_basic() {
        let input = "int main() { return 0; }".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert!(!tokens.is_empty());
        assert!(matches!(tokens[0].token_type, TokenType::Int));
    }

    #[test]
    fn test_lexer_numbers() {
        let input = "42 3.14 'a' \"hello\"".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(
            tokens[0].token_type,
            TokenType::IntegerLiteral(42)
        ));
        assert!(matches!(tokens[1].token_type, TokenType::FloatLiteral(_)));
        assert!(matches!(tokens[2].token_type, TokenType::CharLiteral('a')));
        assert!(matches!(tokens[3].token_type, TokenType::StringLiteral(_)));
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+ - * / == != < > <= >=".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[1].token_type, TokenType::Minus));
        assert!(matches!(tokens[2].token_type, TokenType::Multiply));
        assert!(matches!(tokens[3].token_type, TokenType::Divide));
        assert!(matches!(tokens[4].token_type, TokenType::Equal));
        assert!(matches!(tokens[5].token_type, TokenType::NotEqual));
    }

    #[test]
    fn test_lexer_comments() {
        let input = "int x; // comment\n/* block comment */ int y;".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        // Comments should be filtered out
        let identifier_count = tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Identifier(_)))
            .count();
        assert_eq!(identifier_count, 2); // x and y
    }

    #[test]
    fn test_parser_simple_function() {
        let input = "int main() { return 0; }".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_target_from_string() {
        assert_eq!(Target::from_string("i386"), Some(Target::I386));
        assert_eq!(Target::from_string("amd64"), Some(Target::Amd64));
        assert_eq!(Target::from_string("arm64"), Some(Target::Arm64));
        assert_eq!(Target::from_string("x86_64"), Some(Target::Amd64));
        assert_eq!(Target::from_string("invalid"), None);
    }

    #[test]
    fn test_target_properties() {
        assert_eq!(Target::I386.pointer_size(), 4);
        assert_eq!(Target::Amd64.pointer_size(), 8);
        assert_eq!(Target::Arm64.pointer_size(), 8);
    }

    #[test]
    fn test_codegen_simple() {
        let input = "int main() { return 42; }".to_string();
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let mut codegen = CodeGenerator::new(Target::Amd64);
        let assembly = codegen.generate(&program).unwrap();

        assert!(assembly.contains("main:"));
        assert!(assembly.contains("ret"));
    }

    #[tokio::test]
    async fn test_compiler_invalid_target() {
        let args = Args {
            input_files: vec![PathBuf::from("test.c")],
            target: "invalid_target".to_string(),
            output: None,
            compile_only: false,
            assembly_only: false,
            preprocess_only: false,
            optimization: "0".to_string(),
            debug: false,
            warnings: vec![],
            include_dirs: vec![],
            library_dirs: vec![],
            libraries: vec![],
            defines: vec![],
            undefines: vec![],
            standard: None,
            verbose: false,
            pic: false,
            pie: false,
            static_link: false,
            shared: false,
            thread_model: "posix".to_string(),
            lto: false,
            sysroot: None,
            extra_flags: vec![],
        };

        let result = Compiler::new(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_types() {
        use alecc::error::AleccError;

        let lex_error = AleccError::LexError {
            line: 1,
            column: 5,
            message: "Unexpected character".to_string(),
        };

        assert!(format!("{}", lex_error).contains("line 1"));
        assert!(format!("{}", lex_error).contains("column 5"));
    }
}
