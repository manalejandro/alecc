use crate::parser::{Program, Function, Expression, Statement, Type, BinaryOperator};
use crate::targets::Target;
use crate::error::{AleccError, Result};
use std::collections::HashMap;

pub struct CodeGenerator {
    target: Target,
    output: String,
    label_counter: usize,
    string_literals: HashMap<String, String>,
    current_function_params: Vec<(String, i32)>, // (name, stack_offset)
}

impl CodeGenerator {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            output: String::new(),
            label_counter: 0,
            string_literals: HashMap::new(),
            current_function_params: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String> {
        self.emit_header();
        
        // Generate string literals section
        if !self.string_literals.is_empty() {
            self.emit_line(".section .rodata");
            let string_literals = self.string_literals.clone(); // Clone to avoid borrow issues
            for (content, label) in &string_literals {
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

        // Generate _start entry point
        self.generate_start_function()?;

        Ok(self.output.clone())
    }

    fn generate_start_function(&mut self) -> Result<()> {
        self.emit_line("");
        self.emit_line(".globl _start");
        self.emit_line("_start:");
        
        // Set up stack and call main
        self.emit_line("    push rbp");
        self.emit_line("    mov rbp, rsp");
        
        // Reserve space for temporary operations (prevents stack corruption)
        self.emit_line("    sub rsp, 128");
        
        // Call main function
        self.emit_line("    call main");
        
        // Exit syscall with main's return value
        self.emit_line("    mov rdi, rax");  // exit status = main's return value
        self.emit_line("    mov rax, 60");   // sys_exit syscall number
        self.emit_line("    syscall");       // invoke syscall
        
        Ok(())
    }

    fn emit_header(&mut self) {
        match self.target {
            Target::I386 => {
                self.emit_line(".arch i386");
                self.emit_line(".intel_syntax noprefix");
            }
            Target::Amd64 => {
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
        
        // Set up parameter tracking
        self.current_function_params.clear();
        
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
                
                // Reserve space for parameters + 128 bytes for temporaries
                let stack_space = parameters.len() * 4 + 128;
                self.emit_line(&format!("    sub esp, {}", stack_space));
                
                // Store parameters from stack (i386 calling convention)
                for (i, (name, _)) in parameters.iter().enumerate() {
                    let param_offset = -(i as i32 + 1) * 4;
                    let stack_offset = 8 + i as i32 * 4; // ebp + 8 + offset
                    self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", stack_offset));
                    self.emit_line(&format!("    mov DWORD PTR [ebp + {}], eax", param_offset));
                    self.current_function_params.push((name.clone(), param_offset));
                }
            }
            Target::Amd64 => {
                self.emit_line("    push rbp");
                self.emit_line("    mov rbp, rsp");
                
                // Reserve space for parameters + 128 bytes for temporaries
                let stack_space = parameters.len() * 8 + 128;
                self.emit_line(&format!("    sub rsp, {}", stack_space));
                
                // Store parameters from registers (x86_64 calling convention)
                let param_registers = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for (i, (name, _)) in parameters.iter().enumerate() {
                    let param_offset = -(i as i32 + 1) * 8;
                    if i < param_registers.len() {
                        // Parameter passed in register
                        self.emit_line(&format!("    mov QWORD PTR [rbp + {}], {}", param_offset, param_registers[i]));
                    } else {
                        // Parameter passed on stack
                        let stack_offset = 16 + (i - param_registers.len()) as i32 * 8;
                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", stack_offset));
                        self.emit_line(&format!("    mov QWORD PTR [rbp + {}], rax", param_offset));
                    }
                    self.current_function_params.push((name.clone(), param_offset));
                }
            }
            Target::Arm64 => {
                self.emit_line("    stp x29, x30, [sp, #-16]!");
                self.emit_line("    mov x29, sp");
                
                let stack_space = (parameters.len() * 8 + 128 + 15) & !15; // 16-byte aligned
                self.emit_line(&format!("    sub sp, sp, #{}", stack_space));
                
                // Store parameters from registers (ARM64 calling convention)
                for (i, (name, _)) in parameters.iter().enumerate() {
                    let param_offset = -(i as i32 + 1) * 8;
                    if i < 8 {
                        // Parameter passed in register x0-x7
                        self.emit_line(&format!("    str x{}, [x29, #{}]", i, param_offset));
                    } else {
                        // Parameter passed on stack
                        let stack_offset = 16 + (i - 8) as i32 * 8;
                        self.emit_line(&format!("    ldr x9, [x29, #{}]", stack_offset));
                        self.emit_line(&format!("    str x9, [x29, #{}]", param_offset));
                    }
                    self.current_function_params.push((name.clone(), param_offset));
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
                // Check if it's a function parameter first
                if let Some((_, offset)) = self.current_function_params.iter().find(|(param_name, _)| param_name == name) {
                    // Load parameter from stack
                    match self.target {
                        Target::I386 => {
                            self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                        }
                        Target::Amd64 => {
                            self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                        }
                        Target::Arm64 => {
                            self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                        }
                    }
                } else {
                    // Load global variable
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
            }
            Expression::Call { function, arguments } => {
                // Generate arguments and place in calling convention registers/stack
                match self.target {
                    Target::I386 => {
                        // i386: push arguments in reverse order
                        for arg in arguments.iter().rev() {
                            self.generate_expression(arg)?;
                            self.emit_line("    push eax");
                        }
                    }
                    Target::Amd64 => {
                        // x86_64: first 6 args in registers, rest on stack
                        let param_registers = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                        
                        // Generate arguments and store in registers/stack
                        for (i, arg) in arguments.iter().enumerate() {
                            self.generate_expression(arg)?;
                            if i < param_registers.len() {
                                // Move to parameter register
                                self.emit_line(&format!("    mov {}, rax", param_registers[i]));
                            } else {
                                // Push to stack (reverse order for stack args)
                                self.emit_line("    push rax");
                            }
                        }
                    }
                    Target::Arm64 => {
                        // ARM64: first 8 args in x0-x7, rest on stack
                        for (i, arg) in arguments.iter().enumerate() {
                            self.generate_expression(arg)?;
                            if i < 8 {
                                if i > 0 {
                                    self.emit_line(&format!("    mov x{}, x0", i));
                                }
                                // x0 already has the result for first argument
                            } else {
                                self.emit_line("    str x0, [sp, #-16]!");
                            }
                        }
                    }
                }
                
                if let Expression::Identifier(func_name) = function.as_ref() {
                    self.emit_line(&format!("    call {}", func_name));
                } else {
                    return Err(AleccError::CodegenError {
                        message: "Indirect function calls not implemented".to_string(),
                    });
                }
                
                // Clean up stack for arguments that were pushed
                match self.target {
                    Target::I386 => {
                        let stack_cleanup = arguments.len() * 4;
                        if stack_cleanup > 0 {
                            self.emit_line(&format!("    add esp, {}", stack_cleanup));
                        }
                    }
                    Target::Amd64 => {
                        // Clean up stack arguments (if any)
                        let stack_args = if arguments.len() > 6 { arguments.len() - 6 } else { 0 };
                        if stack_args > 0 {
                            self.emit_line(&format!("    add rsp, {}", stack_args * 8));
                        }
                    }
                    Target::Arm64 => {
                        // Clean up stack arguments (if any)
                        let stack_args = if arguments.len() > 8 { arguments.len() - 8 } else { 0 };
                        if stack_args > 0 {
                            self.emit_line(&format!("    add sp, sp, #{}", stack_args * 16));
                        }
                    }
                }
            }
            Expression::Binary { left, operator, right } => {
                // Generate binary operations
                // First generate right operand and save it
                self.generate_expression(right)?;
                match self.target {
                    Target::I386 => {
                        self.emit_line("    push eax");  // Save right operand
                    }
                    Target::Amd64 => {
                        self.emit_line("    push rax");  // Save right operand
                    }
                    Target::Arm64 => {
                        self.emit_line("    str x0, [sp, #-16]!");  // Save right operand
                    }
                }
                
                // Generate left operand
                self.generate_expression(left)?;
                
                // Pop right operand and perform operation
                match self.target {
                    Target::I386 => {
                        self.emit_line("    pop ebx");   // Right operand in ebx
                        match operator {
                            BinaryOperator::Add => self.emit_line("    add eax, ebx"),
                            BinaryOperator::Subtract => self.emit_line("    sub eax, ebx"),
                            BinaryOperator::Multiply => self.emit_line("    imul eax, ebx"),
                            BinaryOperator::Divide => {
                                self.emit_line("    cdq");  // Sign extend eax to edx:eax
                                self.emit_line("    idiv ebx");
                            }
                            _ => {
                                return Err(AleccError::CodegenError {
                                    message: format!("Binary operator {:?} not implemented for i386", operator),
                                });
                            }
                        }
                    }
                    Target::Amd64 => {
                        self.emit_line("    pop rbx");   // Right operand in rbx
                        match operator {
                            BinaryOperator::Add => self.emit_line("    add rax, rbx"),
                            BinaryOperator::Subtract => self.emit_line("    sub rax, rbx"),
                            BinaryOperator::Multiply => self.emit_line("    imul rax, rbx"),
                            BinaryOperator::Divide => {
                                self.emit_line("    cqo");  // Sign extend rax to rdx:rax
                                self.emit_line("    idiv rbx");
                            }
                            // Comparison operators
                            BinaryOperator::Equal => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    sete al");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::NotEqual => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    setne al");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::Less => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    setl al");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::Greater => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    setg al");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::LessEqual => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    setle al");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::GreaterEqual => {
                                self.emit_line("    cmp rax, rbx");
                                self.emit_line("    setge al");
                                self.emit_line("    movzx rax, al");
                            }
                            _ => {
                                return Err(AleccError::CodegenError {
                                    message: format!("Binary operator {:?} not implemented for amd64", operator),
                                });
                            }
                        }
                    }
                    Target::Arm64 => {
                        self.emit_line("    ldr x1, [sp], #16");  // Right operand in x1
                        match operator {
                            BinaryOperator::Add => self.emit_line("    add x0, x0, x1"),
                            BinaryOperator::Subtract => self.emit_line("    sub x0, x0, x1"),
                            BinaryOperator::Multiply => self.emit_line("    mul x0, x0, x1"),
                            BinaryOperator::Divide => self.emit_line("    sdiv x0, x0, x1"),
                            _ => {
                                return Err(AleccError::CodegenError {
                                    message: format!("Binary operator {:?} not implemented for arm64", operator),
                                });
                            }
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
