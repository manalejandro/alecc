use crate::cli::Args;
use crate::codegen::CodeGenerator;
use crate::error::{AleccError, Result};
use crate::lexer::Lexer;
use crate::linker::Linker;
use crate::optimizer::{OptimizationLevel, Optimizer};
use crate::parser::Parser;
use crate::targets::Target;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{debug, info, warn};

pub struct Compiler {
    args: Args,
    target: Target,
    temp_files: Vec<PathBuf>,
}

impl Compiler {
    pub fn new(args: Args) -> Result<Self> {
        let target =
            Target::from_string(&args.target).ok_or_else(|| AleccError::UnsupportedTarget {
                target: args.target.clone(),
            })?;

        Ok(Self {
            args,
            target,
            temp_files: Vec::new(),
        })
    }

    pub async fn compile(&mut self) -> Result<()> {
        if self.args.input_files.is_empty() {
            return Err(AleccError::InvalidArgument {
                message: "No input files specified".to_string(),
            });
        }

        info!(
            "Compiling {} files for target {}",
            self.args.input_files.len(),
            self.target.as_str()
        );

        let mut object_files = Vec::new();
        let input_files = self.args.input_files.clone(); // Clone to avoid borrow issues

        // Process each input file
        for input_file in &input_files {
            debug!("Processing file: {}", input_file.display());

            let extension = input_file
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            match extension {
                "c" | "cpp" | "cxx" | "cc" | "C" => {
                    let obj_file = self.compile_source_file(input_file).await?;
                    if !self.args.compile_only
                        && !self.args.assembly_only
                        && !self.args.preprocess_only
                    {
                        object_files.push(obj_file);
                    }
                }
                "s" | "S" => {
                    let obj_file = self.assemble_file(input_file).await?;
                    if !self.args.compile_only && !self.args.assembly_only {
                        object_files.push(obj_file);
                    }
                }
                "o" => {
                    object_files.push(input_file.clone());
                }
                _ => {
                    warn!(
                        "Unknown file extension for {}, treating as C source",
                        input_file.display()
                    );
                    let obj_file = self.compile_source_file(input_file).await?;
                    if !self.args.compile_only
                        && !self.args.assembly_only
                        && !self.args.preprocess_only
                    {
                        object_files.push(obj_file);
                    }
                }
            }
        }

        // Link if not compile-only
        if !self.args.compile_only && !self.args.assembly_only && !self.args.preprocess_only {
            self.link_files(object_files).await?;
        }

        // Cleanup temporary files
        self.cleanup().await?;

        Ok(())
    }

    async fn compile_source_file(&mut self, input_file: &Path) -> Result<PathBuf> {
        info!("Compiling source file: {}", input_file.display());

        // Read source file
        let source =
            fs::read_to_string(input_file)
                .await
                .map_err(|_e| AleccError::FileNotFound {
                    path: input_file.to_string_lossy().to_string(),
                })?;

        // Preprocessing
        let preprocessed = if self.args.preprocess_only {
            let output_path = self.get_output_path(input_file, "i")?;
            let preprocessed = self.preprocess(&source, input_file).await?;
            fs::write(&output_path, preprocessed)
                .await
                .map_err(AleccError::IoError)?;
            return Ok(output_path);
        } else {
            self.preprocess(&source, input_file).await?
        };

        // Lexical analysis
        debug!("Lexical analysis for {}", input_file.display());
        let mut lexer = Lexer::new(preprocessed);
        let tokens = lexer.tokenize()?;

        // Parsing
        debug!("Parsing {}", input_file.display());
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse()?;

        // Optimization
        let opt_level = OptimizationLevel::from_string(&self.args.optimization);
        let mut optimizer = Optimizer::new(opt_level);
        optimizer.optimize(&mut program)?;

        // Code generation
        debug!("Code generation for {}", input_file.display());
        let mut codegen = CodeGenerator::new(self.target);
        let assembly = codegen.generate(&program)?;

        if self.args.assembly_only {
            let output_path = self.get_output_path(input_file, "s")?;
            fs::write(&output_path, assembly)
                .await
                .map_err(AleccError::IoError)?;
            return Ok(output_path);
        }

        // Write assembly to temporary file
        let asm_path = self.create_temp_file("s")?;
        fs::write(&asm_path, assembly)
            .await
            .map_err(AleccError::IoError)?;

        // Assemble
        let obj_path = self.assemble_file(&asm_path).await?;

        Ok(obj_path)
    }

    async fn preprocess(&self, source: &str, input_file: &Path) -> Result<String> {
        debug!("Preprocessing {}", input_file.display());

        // Simple preprocessing - just handle basic #include and #define
        let mut preprocessed = String::new();
        let mut defines = std::collections::HashMap::new();

        // Add command-line defines
        for define in &self.args.defines {
            if let Some(eq_pos) = define.find('=') {
                let key = define[..eq_pos].to_string();
                let value = define[eq_pos + 1..].to_string();
                defines.insert(key, value);
            } else {
                defines.insert(define.clone(), "1".to_string());
            }
        }

        // Process source line by line
        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("#include") {
                // Handle #include (simplified)
                match self.extract_include_file(trimmed) {
                    Ok(include_file) => {
                        match self.resolve_include_path(&include_file) {
                            Ok(include_path) => {
                                if include_path.exists() {
                                    match fs::read_to_string(&include_path).await {
                                        Ok(include_content) => {
                                            // Simple include without recursive preprocessing to avoid recursion issues
                                            preprocessed.push_str(&include_content);
                                            preprocessed.push('\n');
                                        }
                                        Err(_) => {
                                            // Skip file if can't read
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Skip include if can't resolve path
                            }
                        }
                    }
                    Err(_) => {
                        // Skip malformed include
                    }
                }
            } else if let Some(stripped) = trimmed.strip_prefix("#define") {
                // Handle #define (simplified)
                let parts: Vec<&str> = stripped.split_whitespace().collect();
                if !parts.is_empty() {
                    let key = parts[0].to_string();
                    let value = if parts.len() > 1 {
                        parts[1..].join(" ")
                    } else {
                        "1".to_string()
                    };
                    defines.insert(key, value);
                }
            } else if !trimmed.starts_with('#') {
                // Regular line - expand macros
                let mut expanded_line = line.to_string();
                for (key, value) in &defines {
                    expanded_line = expanded_line.replace(key, value);
                }
                preprocessed.push_str(&expanded_line);
                preprocessed.push('\n');
            }
        }

        Ok(preprocessed)
    }

    fn extract_include_file(&self, line: &str) -> Result<String> {
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if start != end {
                    return Ok(line[start + 1..end].to_string());
                }
            }
        }

        if let Some(start) = line.find('<') {
            if let Some(end) = line.rfind('>') {
                if start != end {
                    return Ok(line[start + 1..end].to_string());
                }
            }
        }

        Err(AleccError::ParseError {
            line: 0,
            column: 0,
            message: format!("Invalid #include directive: {}", line),
        })
    }

    fn resolve_include_path(&self, include_file: &str) -> Result<PathBuf> {
        // Check current directory first
        let current_path = PathBuf::from(include_file);
        if current_path.exists() {
            return Ok(current_path);
        }

        // Check include directories
        for include_dir in &self.args.include_dirs {
            let path = include_dir.join(include_file);
            if path.exists() {
                return Ok(path);
            }
        }

        // Check system include directories
        let system_includes = match self.target {
            Target::I386 => vec![
                "/usr/include",
                "/usr/local/include",
                "/usr/include/i386-linux-gnu",
            ],
            Target::Amd64 => vec![
                "/usr/include",
                "/usr/local/include",
                "/usr/include/x86_64-linux-gnu",
            ],
            Target::Arm64 => vec![
                "/usr/include",
                "/usr/local/include",
                "/usr/include/aarch64-linux-gnu",
            ],
        };

        for sys_dir in system_includes {
            let path = Path::new(sys_dir).join(include_file);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(AleccError::FileNotFound {
            path: include_file.to_string(),
        })
    }

    async fn assemble_file(&mut self, asm_file: &Path) -> Result<PathBuf> {
        debug!("Assembling {}", asm_file.display());

        let obj_path = if self.args.compile_only {
            self.get_output_path(asm_file, "o")?
        } else {
            self.create_temp_file("o")?
        };

        let assembler = match self.target {
            Target::I386 => "as",
            Target::Amd64 => "as",
            Target::Arm64 => "aarch64-linux-gnu-as",
        };

        let mut command = Command::new(assembler);

        match self.target {
            Target::I386 => {
                command.args(["--32"]);
            }
            Target::Amd64 => {
                command.args(["--64"]);
            }
            Target::Arm64 => {
                // Default options for aarch64
            }
        }

        command.args([
            "-o",
            &obj_path.to_string_lossy(),
            &asm_file.to_string_lossy(),
        ]);

        let output = command.output().map_err(|e| AleccError::CodegenError {
            message: format!("Failed to execute assembler: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AleccError::CodegenError {
                message: format!("Assembly failed: {}", stderr),
            });
        }

        Ok(obj_path)
    }

    async fn link_files(&mut self, object_files: Vec<PathBuf>) -> Result<()> {
        info!("Linking {} object files", object_files.len());

        let mut linker = Linker::new(self.target);

        // Set output path
        let output_path = self.args.output.clone().unwrap_or_else(|| {
            if self.args.shared {
                PathBuf::from("lib.so")
            } else {
                PathBuf::from("a.out")
            }
        });
        linker.set_output_path(output_path);

        // Add object files
        for obj in object_files {
            linker.add_object_file(obj);
        }

        // Add library paths
        for lib_path in &self.args.library_dirs {
            linker.add_library_path(lib_path.clone());
        }

        // Add libraries
        for lib in &self.args.libraries {
            linker.add_library(lib.clone());
        }

        // Set linker options
        linker.set_static_link(self.args.static_link);
        linker.set_shared(self.args.shared);
        linker.set_pic(self.args.pic);
        linker.set_pie(self.args.pie);
        linker.set_debug(self.args.debug);
        linker.set_lto(self.args.lto);
        linker.set_sysroot(self.args.sysroot.clone());

        // Link
        if self.args.shared {
            linker.link_shared_library(None).await?;
        } else {
            linker.link().await?;
        }

        Ok(())
    }

    fn get_output_path(&self, input_file: &Path, extension: &str) -> Result<PathBuf> {
        if let Some(ref output) = self.args.output {
            Ok(output.clone())
        } else {
            let stem = input_file
                .file_stem()
                .ok_or_else(|| AleccError::InvalidArgument {
                    message: "Invalid input file name".to_string(),
                })?;
            Ok(PathBuf::from(format!(
                "{}.{}",
                stem.to_string_lossy(),
                extension
            )))
        }
    }

    fn create_temp_file(&mut self, extension: &str) -> Result<PathBuf> {
        let temp_path = std::env::temp_dir().join(format!(
            "alecc_{}_{}.{}",
            std::process::id(),
            self.temp_files.len(),
            extension
        ));
        self.temp_files.push(temp_path.clone());
        Ok(temp_path)
    }

    async fn cleanup(&mut self) -> Result<()> {
        for temp_file in &self.temp_files {
            if temp_file.exists() {
                if let Err(e) = fs::remove_file(temp_file).await {
                    warn!(
                        "Failed to remove temporary file {}: {}",
                        temp_file.display(),
                        e
                    );
                }
            }
        }
        self.temp_files.clear();
        Ok(())
    }
}
