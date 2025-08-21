use crate::parser::{Program, Function, Expression, Statement, Type};
use crate::targets::Target;
use crate::error::{AleccError, Result};
use std::collections::HashMap;

pub struct CodeGenerator {
    target: Target,
    output: String,
    label_counter: usize,
    string_literals: HashMap<String, String>,
}

impl CodeGenerator {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            output: String::new(),
            label_counter: 0,
            string_literals: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String> {
        self.emit_header();
        
        // Generate string literals section
        if !self.string_literals.is_empty() {
            self.emit_line(".section .rodata");
            for (content, label) in &self.string_literals {
                self.emit_line(&format!("{}:", label));
                self.emit_line(&format!("    .string \"{}\"", self.escape_string(content)));
            }
            self.emit_line("");
        }

        // Generate global variables
        if !program.global_variables.is_empty() {
            self.emit_line(".section .data");
            for (name, var_type, _initializer) in &program.global_variables {
                self.emit_global_variable(name, var_type)?;
            }
            self.emit_line("");
        }

        // Generate functions
        self.emit_line(".section .text");
        for function in &program.functions {
            self.generate_function(function)?;
        }

        Ok(self.output.clone())
    }

    fn emit_header(&mut self) {
        match self.target {
            Target::I386 => {
                self.emit_line(".arch i386");
                self.emit_line(".intel_syntax noprefix");
            }
            Target::Amd64 => {
                self.emit_line(".arch x86_64");
                self.emit_line(".intel_syntax noprefix");
            }
            Target::Arm64 => {
                self.emit_line(".arch armv8-a");
            }
        }
        self.emit_line("");
    }

    fn generate_function(&mut self, function: &Function) -> Result<()> {
        self.emit_line(&format!(".globl {}", function.name));
        self.emit_line(&format!("{}:", function.name));
        
        // Function prologue
        self.emit_function_prologue(&function.parameters)?;
        
        // Function body
        self.generate_statement(&function.body)?;
        
        // Function epilogue (if no explicit return)
        self.emit_function_epilogue()?;
        
        self.emit_line("");
        Ok(())
    }

    fn emit_function_prologue(&mut self, parameters: &[(String, Type)]) -> Result<()> {
        match self.target {
            Target::I386 => {
                self.emit_line("    push ebp");
                self.emit_line("    mov ebp, esp");
                
                // Reserve space for local variables (simplified)
                let stack_space = parameters.len() * 4; // Simplified calculation
                if stack_space > 0 {
                    self.emit_line(&format!("    sub esp, {}", stack_space));
                }
            }
            Target::Amd64 => {
                self.emit_line("    push rbp");
                self.emit_line("    mov rbp, rsp");
                
                let stack_space = parameters.len() * 8;
                if stack_space > 0 {
                    self.emit_line(&format!("    sub rsp, {}", stack_space));
                }
            }
            Target::Arm64 => {
                self.emit_line("    stp x29, x30, [sp, #-16]!");
                self.emit_line("    mov x29, sp");
                
                let stack_space = (parameters.len() * 8 + 15) & !15; // 16-byte aligned
                if stack_space > 0 {
                    self.emit_line(&format!("    sub sp, sp, #{}", stack_space));
                }
            }
        }
        Ok(())
    }

    fn emit_function_epilogue(&mut self) -> Result<()> {
        match self.target {
            Target::I386 => {
                self.emit_line("    mov esp, ebp");
                self.emit_line("    pop ebp");
                self.emit_line("    ret");
            }
            Target::Amd64 => {
                self.emit_line("    mov rsp, rbp");
                self.emit_line("    pop rbp");
                self.emit_line("    ret");
            }
            Target::Arm64 => {
                self.emit_line("    mov sp, x29");
                self.emit_line("    ldp x29, x30, [sp], #16");
                self.emit_line("    ret");
            }
        }
        Ok(())
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expression(expr)?;
                    // Move result to return register
                    match self.target {
                        Target::I386 => {
                            // Result should already be in eax
                        }
                        Target::Amd64 => {
                            // Result should already be in rax
                        }
                        Target::Arm64 => {
                            // Result should already be in x0
                        }
                    }
                }
                self.emit_function_epilogue()?;
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.generate_statement(stmt)?;
                }
            }
            Statement::If { condition, then_stmt, else_stmt } => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");
                
                self.generate_expression(condition)?;
                self.emit_conditional_jump(false, &else_label)?;
                
                self.generate_statement(then_stmt)?;
                self.emit_jump(&end_label)?;
                
                self.emit_line(&format!("{}:", else_label));
                if let Some(else_stmt) = else_stmt {
                    self.generate_statement(else_stmt)?;
                }
                
                self.emit_line(&format!("{}:", end_label));
            }
            Statement::While { condition, body } => {
                let loop_label = self.new_label("loop");
                let end_label = self.new_label("endloop");
                
                self.emit_line(&format!("{}:", loop_label));
                self.generate_expression(condition)?;
                self.emit_conditional_jump(false, &end_label)?;
                
                self.generate_statement(body)?;
                self.emit_jump(&loop_label)?;
                
                self.emit_line(&format!("{}:", end_label));
            }
            _ => {
                // Other statements not implemented yet
                return Err(AleccError::CodegenError {
                    message: "Statement type not implemented".to_string(),
                });
            }
        }
        Ok(())
    }

    fn generate_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            Expression::IntegerLiteral(value) => {
                match self.target {
                    Target::I386 => {
                        self.emit_line(&format!("    mov eax, {}", value));
                    }
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov rax, {}", value));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    mov x0, #{}", value));
                    }
                }
            }
            Expression::StringLiteral(value) => {
                let label = self.get_string_literal_label(value);
                match self.target {
                    Target::I386 => {
                        self.emit_line(&format!("    mov eax, OFFSET {}", label));
                    }
                    Target::Amd64 => {
                        self.emit_line(&format!("    lea rax, [{}]", label));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    adrp x0, {}", label));
                        self.emit_line(&format!("    add x0, x0, :lo12:{}", label));
                    }
                }
            }
            Expression::Identifier(name) => {
                // Load variable (simplified - assumes it's a parameter or global)
                match self.target {
                    Target::I386 => {
                        self.emit_line(&format!("    mov eax, DWORD PTR [{}]", name));
                    }
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov rax, QWORD PTR [{}]", name));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    adrp x1, {}", name));
                        self.emit_line(&format!("    add x1, x1, :lo12:{}", name));
                        self.emit_line("    ldr x0, [x1]");
                    }
                }
            }
            Expression::Call { function, arguments } => {
                // Generate arguments in reverse order
                for (i, arg) in arguments.iter().enumerate().rev() {
                    self.generate_expression(arg)?;
                    self.push_argument(i)?;
                }
                
                if let Expression::Identifier(func_name) = function.as_ref() {
                    self.emit_line(&format!("    call {}", func_name));
                } else {
                    return Err(AleccError::CodegenError {
                        message: "Indirect function calls not implemented".to_string(),
                    });
                }
                
                // Clean up stack
                let stack_cleanup = arguments.len() * self.target.pointer_size();
                if stack_cleanup > 0 {
                    match self.target {
                        Target::I386 => {
                            self.emit_line(&format!("    add esp, {}", stack_cleanup));
                        }
                        Target::Amd64 => {
                            // Arguments passed in registers, no cleanup needed
                        }
                        Target::Arm64 => {
                            // Arguments passed in registers, no cleanup needed
                        }
                    }
                }
            }
            _ => {
                return Err(AleccError::CodegenError {
                    message: "Expression type not implemented".to_string(),
                });
            }
        }
        Ok(())
    }

    fn push_argument(&mut self, _index: usize) -> Result<()> {
        match self.target {
            Target::I386 => {
                self.emit_line("    push eax");
            }
            Target::Amd64 => {
                // Use calling convention registers
                self.emit_line("    push rax"); // Simplified
            }
            Target::Arm64 => {
                // Use calling convention registers
                self.emit_line("    str x0, [sp, #-16]!"); // Simplified
            }
        }
        Ok(())
    }

    fn emit_conditional_jump(&mut self, condition: bool, label: &str) -> Result<()> {
        let instruction = if condition { "jnz" } else { "jz" };
        
        match self.target {
            Target::I386 | Target::Amd64 => {
                self.emit_line(&format!("    test eax, eax"));
                self.emit_line(&format!("    {} {}", instruction, label));
            }
            Target::Arm64 => {
                let branch_inst = if condition { "cbnz" } else { "cbz" };
                self.emit_line(&format!("    {} x0, {}", branch_inst, label));
            }
        }
        Ok(())
    }

    fn emit_jump(&mut self, label: &str) -> Result<()> {
        match self.target {
            Target::I386 | Target::Amd64 => {
                self.emit_line(&format!("    jmp {}", label));
            }
            Target::Arm64 => {
                self.emit_line(&format!("    b {}", label));
            }
        }
        Ok(())
    }

    fn emit_global_variable(&mut self, name: &str, var_type: &Type) -> Result<()> {
        let size = self.get_type_size(var_type);
        self.emit_line(&format!("{}:", name));
        match size {
            1 => self.emit_line("    .byte 0"),
            2 => self.emit_line("    .word 0"),
            4 => self.emit_line("    .long 0"),
            8 => self.emit_line("    .quad 0"),
            _ => self.emit_line(&format!("    .zero {}", size)),
        }
        Ok(())
    }

    fn get_type_size(&self, var_type: &Type) -> usize {
        match var_type {
            Type::Char => 1,
            Type::Short => 2,
            Type::Int => 4,
            Type::Long => self.target.pointer_size(),
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) => self.target.pointer_size(),
            _ => self.target.pointer_size(), // Default
        }
    }

    fn get_string_literal_label(&mut self, content: &str) -> String {
        if let Some(label) = self.string_literals.get(content) {
            label.clone()
        } else {
            let label = format!(".LC{}", self.string_literals.len());
            self.string_literals.insert(content.to_string(), label.clone());
            label
        }
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!(".L{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('\r', "\\r")
    }
}
