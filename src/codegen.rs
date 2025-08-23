use crate::parser::{Program, Function, Expression, Statement, Type, BinaryOperator, UnaryOperator};
use crate::targets::Target;
use crate::error::{AleccError, Result};
use std::collections::HashMap;

pub struct CodeGenerator {
    target: Target,
    output: String,
    label_counter: usize,
    string_literals: HashMap<String, String>,
    current_function_params: Vec<(String, i32)>, // (name, stack_offset)
    epilogue_emitted: bool,
    local_variables: HashMap<String, i32>, // (name, stack_offset)
    stack_offset: i32, // Current stack offset for local variables
    last_call_stack_cleanup: usize, // Stack bytes to clean up after last call
}

impl CodeGenerator {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            output: String::new(),
            label_counter: 0,
            string_literals: HashMap::new(),
            current_function_params: Vec::new(),
            epilogue_emitted: false,
            local_variables: HashMap::new(),
            stack_offset: 0,
            last_call_stack_cleanup: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String> {
        // First pass: collect all string literals
        for function in &program.functions {
            self.collect_string_literals_from_statement(&function.body)?;
        }
        
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
        
        // Reserve space for temporary operations (ensures proper stack alignment)
        // 120 bytes = 15*8, so after rbp push (8 bytes), total is 128 bytes = multiple of 16
        self.emit_line("    sub rsp, 120");
        
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
        // Check if function has a body (implementation) or is just a declaration
        match &function.body {
            Statement::Block(statements) if statements.is_empty() => {
                // This is a forward declaration, generate an external reference
                self.emit_line(&format!(".extern {}", function.name));
                return Ok(());
            }
            _ => {
                // This is a function definition, generate the actual function
            }
        }
        
        self.emit_line(&format!(".globl {}", function.name));
        self.emit_line(&format!("{}:", function.name));
        
        // Set up parameter tracking
        self.current_function_params.clear();
        self.local_variables.clear();
        // Start local variables after parameters to avoid collision
        self.stack_offset = -(function.parameters.len() as i32 * 8);
        self.epilogue_emitted = false;
        
        // Function prologue
        self.emit_function_prologue(&function.parameters)?;
        
        // Function body
        self.generate_statement(&function.body)?;
        
        // Function epilogue (always ensure we have a proper function ending)
        // This handles cases where there might not be explicit returns in all paths
        self.emit_function_epilogue()?;
        
        self.emit_line("");
        Ok(())
    }

    fn emit_function_prologue(&mut self, parameters: &[(String, Type)]) -> Result<()> {
        match self.target {
            Target::I386 => {
                self.emit_line("    push ebp");
                self.emit_line("    mov ebp, esp");
                
                // Reserve space for parameters only (no extra temporaries for now)
                let stack_space = parameters.len() * 4;
                if stack_space > 0 {
                    self.emit_line(&format!("    sub esp, {}", stack_space));
                }
                
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
                
                // Reserve space for parameters + ensure 16-byte alignment
                let stack_space = parameters.len() * 8;
                // Always reserve at least 8 bytes to maintain 16-byte alignment after rbp push
                let min_space = if stack_space == 0 { 8 } else { stack_space };
                let aligned_space = ((min_space + 15) / 16) * 16; // Round up to 16-byte boundary
                self.emit_line(&format!("    sub rsp, {}", aligned_space));
                
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
                
                let stack_space = parameters.len() * 8;
                if stack_space > 0 {
                    let aligned_space = (stack_space + 15) & !15; // 16-byte aligned
                    self.emit_line(&format!("    sub sp, sp, #{}", aligned_space));
                }
                
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
        if self.epilogue_emitted {
            return Ok(()); // Don't emit duplicate epilogues
        }
        
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
        
        self.epilogue_emitted = true;
        Ok(())
    }

    fn emit_function_epilogue_force(&mut self) -> Result<()> {
        // Force emit epilogue regardless of epilogue_emitted flag
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
        
        self.epilogue_emitted = true;
        Ok(())
    }

    fn generate_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Expression(expr) => {
                self.generate_expression(expr)?;
            }
            Statement::Declaration { name, var_type, initializer } => {
                // Calculate space needed based on type
                let size = match var_type {
                    Type::Array(_, Some(length)) => length * 8, // Assuming 8-byte elements
                    Type::Array(_, None) => 80, // Default size for unsized arrays
                    _ => 8, // Default 8 bytes for simple types
                };
                
                // Allocate space for variable/array
                self.stack_offset -= size as i32;
                let var_offset = self.stack_offset;
                
                // Store variable name and offset for later reference
                self.local_variables.insert(name.clone(), var_offset);
                
                if let Some(init_expr) = initializer {
                    self.generate_expression(init_expr)?;
                    // Store the value in the local variable slot
                    match self.target {
                        Target::Amd64 => {
                            self.emit_line(&format!("    mov QWORD PTR [rbp + {}], rax", var_offset));
                        }
                        Target::I386 => {
                            self.emit_line(&format!("    mov DWORD PTR [ebp + {}], eax", var_offset));
                        }
                        Target::Arm64 => {
                            self.emit_line(&format!("    str x0, [x29, #{}]", var_offset));
                        }
                    }
                }
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
                // Force emit epilogue for each return statement
                self.emit_function_epilogue_force()?;
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
                    // Reset epilogue flag for else branch in case it contains a return
                    let saved_epilogue_state = self.epilogue_emitted;
                    self.epilogue_emitted = false;
                    self.generate_statement(else_stmt)?;
                    // If else branch didn't emit epilogue, restore the saved state
                    if !self.epilogue_emitted {
                        self.epilogue_emitted = saved_epilogue_state;
                    }
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
            Statement::For { init, condition, increment, body } => {
                // Generate initialization
                if let Some(init_stmt) = init {
                    self.generate_statement(init_stmt)?;
                }
                
                let loop_label = self.new_label("forloop");
                let end_label = self.new_label("endfor");
                
                self.emit_line(&format!("{}:", loop_label));
                
                // Generate condition check
                if let Some(cond_expr) = condition {
                    self.generate_expression(cond_expr)?;
                    self.emit_conditional_jump(false, &end_label)?;
                }
                
                // Generate body
                self.generate_statement(body)?;
                
                // Generate increment
                if let Some(inc_expr) = increment {
                    self.generate_expression(inc_expr)?;
                }
                
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
                } else if let Some(offset) = self.local_variables.get(name) {
                    // Load local variable from stack
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
                        
                        // Ensure stack alignment before function call
                        // Stack must be 16-byte aligned before 'call' instruction
                        // Since 'call' pushes 8 bytes (return address), we need stack to be 8 bytes off 16-byte boundary
                        let stack_args = if arguments.len() > param_registers.len() { arguments.len() - param_registers.len() } else { 0 };
                        let mut stack_cleanup_size = 0;
                        
                        // Handle stack arguments if any
                        if stack_args > 0 {
                            let total_stack_bytes = stack_args * 8;
                            // Ensure alignment: if total_stack_bytes is odd multiple of 8, add 8 bytes for alignment
                            if (total_stack_bytes / 8) % 2 != 0 {
                                self.emit_line("    sub rsp, 8  # Stack alignment");
                                stack_cleanup_size += 8;
                            }
                            stack_cleanup_size += stack_args * 8;
                        }
                        // Note: No additional alignment for register-only calls since function prologue handles it
                        
                        // First, save any arguments that go on the stack (in reverse order)
                        if arguments.len() > param_registers.len() {
                            for arg in arguments.iter().skip(param_registers.len()).rev() {
                                self.generate_expression(arg)?;
                                self.emit_line("    push rax");
                            }
                        }
                        
                        // Then handle register arguments in reverse order to avoid overwriting
                        let reg_args: Vec<_> = arguments.iter().take(param_registers.len()).collect();
                        for (i, arg) in reg_args.iter().enumerate().rev() {
                            self.generate_expression(arg)?;
                            self.emit_line(&format!("    mov {}, rax", param_registers[i]));
                        }
                        
                        // Store cleanup size for later use
                        self.last_call_stack_cleanup = stack_cleanup_size;
                    }
                    Target::Arm64 => {
                        // ARM64: first 8 args in x0-x7, rest on stack
                        // Save stack arguments first
                        if arguments.len() > 8 {
                            for arg in arguments.iter().skip(8).rev() {
                                self.generate_expression(arg)?;
                                self.emit_line("    str x0, [sp, #-16]!");
                            }
                        }
                        
                        // Then handle register arguments in reverse order
                        let reg_args: Vec<_> = arguments.iter().take(8).collect();
                        for (i, arg) in reg_args.iter().enumerate().rev() {
                            self.generate_expression(arg)?;
                            if i > 0 {
                                self.emit_line(&format!("    mov x{}, x0", i));
                            }
                            // x0 already has the result for first argument
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
                        // Clean up stack using stored cleanup size
                        if self.last_call_stack_cleanup > 0 {
                            self.emit_line(&format!("    add rsp, {}", self.last_call_stack_cleanup));
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
                            BinaryOperator::Modulo => {
                                self.emit_line("    cdq");  // Sign extend eax to edx:eax
                                self.emit_line("    idiv ebx");
                                self.emit_line("    mov eax, edx"); // Remainder is in edx
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
                            BinaryOperator::Modulo => {
                                self.emit_line("    cqo");  // Sign extend rax to rdx:rax
                                self.emit_line("    idiv rbx");
                                self.emit_line("    mov rax, rdx"); // Remainder is in rdx
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
                            // Logical operators
                            BinaryOperator::LogicalAnd => {
                                self.emit_line("    test rax, rax");
                                self.emit_line("    setne al");
                                self.emit_line("    test rbx, rbx");
                                self.emit_line("    setne bl");
                                self.emit_line("    and al, bl");
                                self.emit_line("    movzx rax, al");
                            }
                            BinaryOperator::LogicalOr => {
                                self.emit_line("    test rax, rax");
                                self.emit_line("    setne al");
                                self.emit_line("    test rbx, rbx");
                                self.emit_line("    setne bl");
                                self.emit_line("    or al, bl");
                                self.emit_line("    movzx rax, al");
                            }
                            // Bitwise operators
                            BinaryOperator::BitwiseAnd => self.emit_line("    and rax, rbx"),
                            BinaryOperator::BitwiseOr => self.emit_line("    or rax, rbx"),
                            BinaryOperator::BitwiseXor => self.emit_line("    xor rax, rbx"),
                            // Shift operators
                            BinaryOperator::LeftShift => {
                                self.emit_line("    mov rcx, rbx"); // Shift count in rcx
                                self.emit_line("    shl rax, cl");
                            }
                            BinaryOperator::RightShift => {
                                self.emit_line("    mov rcx, rbx"); // Shift count in rcx
                                self.emit_line("    sar rax, cl"); // Arithmetic right shift
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
                            BinaryOperator::Modulo => {
                                self.emit_line("    sdiv x2, x0, x1");  // x2 = x0 / x1
                                self.emit_line("    msub x0, x2, x1, x0"); // x0 = x0 - (x2 * x1)
                            }
                            _ => {
                                return Err(AleccError::CodegenError {
                                    message: format!("Binary operator {:?} not implemented for arm64", operator),
                                });
                            }
                        }
                    }
                }
            }
            Expression::Unary { operator, operand } => {
                match operator {
                    UnaryOperator::Minus => {
                        self.generate_expression(operand)?;
                        match self.target {
                            Target::I386 => {
                                self.emit_line("    neg eax");
                            }
                            Target::Amd64 => {
                                self.emit_line("    neg rax");
                            }
                            Target::Arm64 => {
                                self.emit_line("    neg x0, x0");
                            }
                        }
                    }
                    UnaryOperator::Plus => {
                        // Plus is a no-op, just generate the operand
                        self.generate_expression(operand)?;
                    }
                    UnaryOperator::LogicalNot => {
                        self.generate_expression(operand)?;
                        match self.target {
                            Target::I386 => {
                                self.emit_line("    test eax, eax");
                                self.emit_line("    setz al");
                                self.emit_line("    movzx eax, al");
                            }
                            Target::Amd64 => {
                                self.emit_line("    test rax, rax");
                                self.emit_line("    setz al");
                                self.emit_line("    movzx rax, al");
                            }
                            Target::Arm64 => {
                                self.emit_line("    cmp x0, #0");
                                self.emit_line("    cset x0, eq");
                            }
                        }
                    }
                    UnaryOperator::BitwiseNot => {
                        self.generate_expression(operand)?;
                        match self.target {
                            Target::I386 => {
                                self.emit_line("    not eax");
                            }
                            Target::Amd64 => {
                                self.emit_line("    not rax");
                            }
                            Target::Arm64 => {
                                self.emit_line("    mvn x0, x0");
                            }
                        }
                    }
                    UnaryOperator::PreIncrement => {
                        // Load variable, increment, store back, and leave incremented value in register
                        if let Expression::Identifier(name) = operand.as_ref() {
                            if let Some(&offset) = self.local_variables.get(name) {
                                match self.target {
                                    Target::I386 => {
                                        self.emit_line(&format!("    inc DWORD PTR [ebp + {}]", offset));
                                        self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                                    }
                                    Target::Amd64 => {
                                        self.emit_line(&format!("    inc QWORD PTR [rbp + {}]", offset));
                                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                                    }
                                    Target::Arm64 => {
                                        self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                                        self.emit_line("    add x0, x0, #1");
                                        self.emit_line(&format!("    str x0, [x29, #{}]", offset));
                                    }
                                }
                            } else {
                                return Err(AleccError::CodegenError {
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        } else {
                            return Err(AleccError::CodegenError {
                                message: "Pre-increment can only be applied to variables".to_string(),
                            });
                        }
                    }
                    UnaryOperator::PostIncrement => {
                        // Load variable, store incremented value, but leave original value in register
                        if let Expression::Identifier(name) = operand.as_ref() {
                            if let Some(&offset) = self.local_variables.get(name) {
                                match self.target {
                                    Target::I386 => {
                                        self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                                        self.emit_line(&format!("    inc DWORD PTR [ebp + {}]", offset));
                                    }
                                    Target::Amd64 => {
                                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                                        self.emit_line(&format!("    inc QWORD PTR [rbp + {}]", offset));
                                    }
                                    Target::Arm64 => {
                                        self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                                        self.emit_line(&format!("    ldr x1, [x29, #{}]", offset));
                                        self.emit_line("    add x1, x1, #1");
                                        self.emit_line(&format!("    str x1, [x29, #{}]", offset));
                                    }
                                }
                            } else {
                                return Err(AleccError::CodegenError {
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        } else {
                            return Err(AleccError::CodegenError {
                                message: "Post-increment can only be applied to variables".to_string(),
                            });
                        }
                    }
                    UnaryOperator::PreDecrement => {
                        // Similar to PreIncrement but with decrement
                        if let Expression::Identifier(name) = operand.as_ref() {
                            if let Some(&offset) = self.local_variables.get(name) {
                                match self.target {
                                    Target::I386 => {
                                        self.emit_line(&format!("    dec DWORD PTR [ebp + {}]", offset));
                                        self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                                    }
                                    Target::Amd64 => {
                                        self.emit_line(&format!("    dec QWORD PTR [rbp + {}]", offset));
                                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                                    }
                                    Target::Arm64 => {
                                        self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                                        self.emit_line("    sub x0, x0, #1");
                                        self.emit_line(&format!("    str x0, [x29, #{}]", offset));
                                    }
                                }
                            } else {
                                return Err(AleccError::CodegenError {
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        } else {
                            return Err(AleccError::CodegenError {
                                message: "Pre-decrement can only be applied to variables".to_string(),
                            });
                        }
                    }
                    UnaryOperator::PostDecrement => {
                        // Similar to PostIncrement but with decrement
                        if let Expression::Identifier(name) = operand.as_ref() {
                            if let Some(&offset) = self.local_variables.get(name) {
                                match self.target {
                                    Target::I386 => {
                                        self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                                        self.emit_line(&format!("    dec DWORD PTR [ebp + {}]", offset));
                                    }
                                    Target::Amd64 => {
                                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                                        self.emit_line(&format!("    dec QWORD PTR [rbp + {}]", offset));
                                    }
                                    Target::Arm64 => {
                                        self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                                        self.emit_line(&format!("    ldr x1, [x29, #{}]", offset));
                                        self.emit_line("    sub x1, x1, #1");
                                        self.emit_line(&format!("    str x1, [x29, #{}]", offset));
                                    }
                                }
                            } else {
                                return Err(AleccError::CodegenError {
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        } else {
                            return Err(AleccError::CodegenError {
                                message: "Post-decrement can only be applied to variables".to_string(),
                            });
                        }
                    }
                    UnaryOperator::AddressOf => {
                        // Get address of a variable
                        if let Expression::Identifier(name) = operand.as_ref() {
                            if let Some(&offset) = self.local_variables.get(name) {
                                match self.target {
                                    Target::I386 => {
                                        self.emit_line(&format!("    lea eax, [ebp + {}]", offset));
                                    }
                                    Target::Amd64 => {
                                        self.emit_line(&format!("    lea rax, [rbp + {}]", offset));
                                    }
                                    Target::Arm64 => {
                                        self.emit_line(&format!("    add x0, x29, #{}", offset));
                                    }
                                }
                            } else {
                                return Err(AleccError::CodegenError {
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        } else {
                            return Err(AleccError::CodegenError {
                                message: "Address-of can only be applied to variables".to_string(),
                            });
                        }
                    }
                    UnaryOperator::Dereference => {
                        // Dereference a pointer (load value from address)
                        self.generate_expression(operand)?; // Get the address
                        match self.target {
                            Target::I386 => {
                                self.emit_line("    mov eax, DWORD PTR [eax]");
                            }
                            Target::Amd64 => {
                                self.emit_line("    mov rax, QWORD PTR [rax]");
                            }
                            Target::Arm64 => {
                                self.emit_line("    ldr x0, [x0]");
                            }
                        }
                    }
                }
            }
            Expression::Index { array, index } => {
                // Generate the array base address
                if let Expression::Identifier(array_name) = array.as_ref() {
                    if let Some(&base_offset) = self.local_variables.get(array_name) {
                        // Generate the index expression
                        self.generate_expression(index)?;
                        
                        // Calculate the array element address: base + index * element_size
                        match self.target {
                            Target::Amd64 => {
                                // Multiply index by 8 (assuming int is 8 bytes for simplicity)
                                self.emit_line("    imul rax, 8"); // Use imul instead of mul
                                // Add base address
                                self.emit_line(&format!("    lea rbx, [rbp + {}]", base_offset));
                                self.emit_line("    add rax, rbx");
                                // Load the value at that address
                                self.emit_line("    mov rax, QWORD PTR [rax]");
                            }
                            Target::I386 => {
                                // Similar for 32-bit
                                self.emit_line("    imul eax, 4"); // Use imul instead of mul
                                self.emit_line(&format!("    lea ebx, [ebp + {}]", base_offset));
                                self.emit_line("    add eax, ebx");
                                self.emit_line("    mov eax, DWORD PTR [eax]");
                            }
                            Target::Arm64 => {
                                // ARM64 implementation
                                self.emit_line("    lsl x0, x0, #3"); // multiply by 8
                                self.emit_line(&format!("    add x1, x29, #{}", base_offset));
                                self.emit_line("    add x0, x0, x1");
                                self.emit_line("    ldr x0, [x0]");
                            }
                        }
                    } else {
                        return Err(AleccError::CodegenError {
                            message: format!("Array '{}' not found", array_name),
                        });
                    }
                } else {
                    return Err(AleccError::CodegenError {
                        message: "Complex array expressions not yet supported".to_string(),
                    });
                }
            }
            Expression::Assignment { target, operator, value } => {
                // Handle compound assignment operators
                match operator {
                    crate::parser::AssignmentOperator::Assign => {
                        // Simple assignment: target = value
                        self.generate_expression(value)?;
                        self.store_in_target(target)?;
                    }
                    crate::parser::AssignmentOperator::PlusAssign => {
                        // target += value  =>  target = target + value
                        self.load_from_target(target)?; // Load current value
                        self.emit_line("    push rax"); // Save current value
                        self.generate_expression(value)?; // Generate RHS
                        self.emit_line("    pop rbx"); // Restore current value
                        self.emit_line("    add rax, rbx"); // target + value
                        self.store_in_target(target)?; // Store result
                    }
                    crate::parser::AssignmentOperator::MinusAssign => {
                        // target -= value  =>  target = target - value
                        self.load_from_target(target)?;
                        self.emit_line("    push rax");
                        self.generate_expression(value)?;
                        self.emit_line("    mov rbx, rax"); // RHS in rbx
                        self.emit_line("    pop rax"); // Current value in rax
                        self.emit_line("    sub rax, rbx"); // target - value
                        self.store_in_target(target)?;
                    }
                    crate::parser::AssignmentOperator::MultiplyAssign => {
                        // target *= value  =>  target = target * value
                        self.load_from_target(target)?;
                        self.emit_line("    push rax");
                        self.generate_expression(value)?;
                        self.emit_line("    pop rbx");
                        self.emit_line("    imul rax, rbx"); // target * value
                        self.store_in_target(target)?;
                    }
                    crate::parser::AssignmentOperator::DivideAssign => {
                        // target /= value  =>  target = target / value
                        self.load_from_target(target)?;
                        self.emit_line("    push rax");
                        self.generate_expression(value)?;
                        self.emit_line("    mov rbx, rax"); // RHS in rbx
                        self.emit_line("    pop rax"); // Current value in rax
                        self.emit_line("    cqo"); // Sign extend for division
                        self.emit_line("    idiv rbx"); // target / value
                        self.store_in_target(target)?;
                    }
                    _ => {
                        return Err(AleccError::CodegenError {
                            message: "Assignment operator not implemented".to_string(),
                        });
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

    #[allow(dead_code)]
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

    fn load_from_target(&mut self, target: &Expression) -> Result<()> {
        // Load the current value of target into rax
        if let Expression::Identifier(name) = target {
            if let Some(&offset) = self.local_variables.get(name) {
                match self.target {
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov rax, QWORD PTR [rbp + {}]", offset));
                    }
                    Target::I386 => {
                        self.emit_line(&format!("    mov eax, DWORD PTR [ebp + {}]", offset));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    ldr x0, [x29, #{}]", offset));
                    }
                }
            } else {
                // Global variable
                match self.target {
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov rax, QWORD PTR [{}]", name));
                    }
                    Target::I386 => {
                        self.emit_line(&format!("    mov eax, DWORD PTR [{}]", name));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    adrp x1, {}", name));
                        self.emit_line(&format!("    add x1, x1, :lo12:{}", name));
                        self.emit_line("    ldr x0, [x1]");
                    }
                }
            }
        } else {
            return Err(AleccError::CodegenError {
                message: "Complex assignment targets not supported for compound operators yet".to_string(),
            });
        }
        Ok(())
    }

    fn store_in_target(&mut self, target: &Expression) -> Result<()> {
        // Store rax value into target
        if let Expression::Identifier(name) = target {
            if let Some(&offset) = self.local_variables.get(name) {
                match self.target {
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov QWORD PTR [rbp + {}], rax", offset));
                    }
                    Target::I386 => {
                        self.emit_line(&format!("    mov DWORD PTR [ebp + {}], eax", offset));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    str x0, [x29, #{}]", offset));
                    }
                }
            } else {
                // Global variable
                match self.target {
                    Target::Amd64 => {
                        self.emit_line(&format!("    mov QWORD PTR [{}], rax", name));
                    }
                    Target::I386 => {
                        self.emit_line(&format!("    mov DWORD PTR [{}], eax", name));
                    }
                    Target::Arm64 => {
                        self.emit_line(&format!("    adrp x1, {}", name));
                        self.emit_line(&format!("    add x1, x1, :lo12:{}", name));
                        self.emit_line("    str x0, [x1]");
                    }
                }
            }
        } else {
            return Err(AleccError::CodegenError {
                message: "Complex assignment targets not supported for compound operators yet".to_string(),
            });
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

    fn collect_string_literals_from_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Expression(expr) => self.collect_string_literals_from_expression(expr),
            Statement::Block(statements) => {
                for stmt in statements {
                    self.collect_string_literals_from_statement(stmt)?;
                }
                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.collect_string_literals_from_expression(expr)?;
                }
                Ok(())
            }
            Statement::If { condition, then_stmt, else_stmt } => {
                self.collect_string_literals_from_expression(condition)?;
                self.collect_string_literals_from_statement(then_stmt)?;
                if let Some(else_statement) = else_stmt {
                    self.collect_string_literals_from_statement(else_statement)?;
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.collect_string_literals_from_expression(condition)?;
                self.collect_string_literals_from_statement(body)?;
                Ok(())
            }
            Statement::For { init, condition, increment, body } => {
                if let Some(init_stmt) = init {
                    self.collect_string_literals_from_statement(init_stmt)?;
                }
                if let Some(cond_expr) = condition {
                    self.collect_string_literals_from_expression(cond_expr)?;
                }
                if let Some(inc_expr) = increment {
                    self.collect_string_literals_from_expression(inc_expr)?;
                }
                self.collect_string_literals_from_statement(body)?;
                Ok(())
            }
            Statement::Declaration { initializer, .. } => {
                if let Some(expr) = initializer {
                    self.collect_string_literals_from_expression(expr)?;
                }
                Ok(())
            }
            _ => Ok(()) // Other statement types don't have expressions we need to collect
        }
    }

    fn collect_string_literals_from_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::StringLiteral(value) => {
                self.get_string_literal_label(value);
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.collect_string_literals_from_expression(left)?;
                self.collect_string_literals_from_expression(right)?;
                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.collect_string_literals_from_expression(operand)?;
                Ok(())
            }
            Expression::Call { function, arguments } => {
                self.collect_string_literals_from_expression(function)?;
                for arg in arguments {
                    self.collect_string_literals_from_expression(arg)?;
                }
                Ok(())
            }
            Expression::Assignment { target, value, .. } => {
                self.collect_string_literals_from_expression(target)?;
                self.collect_string_literals_from_expression(value)?;
                Ok(())
            }
            _ => Ok(()) // Other expression types don't contain string literals
        }
    }
}
