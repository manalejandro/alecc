use crate::targets::Target;
use crate::error::{AleccError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Linker {
    target: Target,
    output_path: PathBuf,
    object_files: Vec<PathBuf>,
    library_paths: Vec<PathBuf>,
    libraries: Vec<String>,
    static_link: bool,
    shared: bool,
    pic: bool,
    pie: bool,
    sysroot: Option<PathBuf>,
    debug: bool,
    lto: bool,
}

impl Linker {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            output_path: PathBuf::from("a.out"),
            object_files: Vec::new(),
            library_paths: Vec::new(),
            libraries: Vec::new(),
            static_link: false,
            shared: false,
            pic: false,
            pie: false,
            sysroot: None,
            debug: false,
            lto: false,
        }
    }

    pub fn set_output_path(&mut self, path: PathBuf) {
        self.output_path = path;
    }

    pub fn add_object_file(&mut self, path: PathBuf) {
        self.object_files.push(path);
    }

    pub fn add_library_path(&mut self, path: PathBuf) {
        self.library_paths.push(path);
    }

    pub fn add_library(&mut self, name: String) {
        self.libraries.push(name);
    }

    pub fn set_static_link(&mut self, static_link: bool) {
        self.static_link = static_link;
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.shared = shared;
    }

    pub fn set_pic(&mut self, pic: bool) {
        self.pic = pic;
    }

    pub fn set_pie(&mut self, pie: bool) {
        self.pie = pie;
    }

    pub fn set_sysroot(&mut self, sysroot: Option<PathBuf>) {
        self.sysroot = sysroot;
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn set_lto(&mut self, lto: bool) {
        self.lto = lto;
    }

    pub async fn link(&self) -> Result<()> {
        if self.object_files.is_empty() {
            return Err(AleccError::LinkerError {
                message: "No object files to link".to_string(),
            });
        }

        let linker_command = self.build_linker_command()?;
        
        let output = Command::new(&linker_command[0])
            .args(&linker_command[1..])
            .output()
            .map_err(|e| AleccError::LinkerError {
                message: format!("Failed to execute linker: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AleccError::LinkerError {
                message: format!("Linker failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn build_linker_command(&self) -> Result<Vec<String>> {
        let mut command = Vec::new();
        
        // Choose linker based on target
        let linker = match self.target {
            Target::I386 => "ld",
            Target::Amd64 => "ld",
            Target::Arm64 => "aarch64-linux-gnu-ld",
        };
        
        command.push(linker.to_string());

        // Target-specific flags
        match self.target {
            Target::I386 => {
                command.push("-m".to_string());
                command.push("elf_i386".to_string());
            }
            Target::Amd64 => {
                command.push("-m".to_string());
                command.push("elf_x86_64".to_string());
            }
            Target::Arm64 => {
                command.push("-m".to_string());
                command.push("aarch64linux".to_string());
            }
        }

        // Output file
        command.push("-o".to_string());
        command.push(self.output_path.to_string_lossy().to_string());

        // Sysroot
        if let Some(ref sysroot) = self.sysroot {
            command.push("--sysroot".to_string());
            command.push(sysroot.to_string_lossy().to_string());
        }

        // Position independent code
        if self.pic {
            command.push("-shared".to_string());
        }

        // Position independent executable
        if self.pie {
            command.push("-pie".to_string());
        }

        // Static linking
        if self.static_link {
            command.push("-static".to_string());
        }

        // Shared library
        if self.shared {
            command.push("-shared".to_string());
        }

        // Debug information
        if self.debug {
            command.push("-g".to_string());
        }

        // LTO
        if self.lto {
            command.push("--lto-O3".to_string());
        }

        // Dynamic linker
        if !self.static_link && !self.shared {
            let dynamic_linker = match self.target {
                Target::I386 => "/lib/ld-linux.so.2",
                Target::Amd64 => "/lib64/ld-linux-x86-64.so.2",
                Target::Arm64 => "/lib/ld-linux-aarch64.so.1",
            };
            command.push("-dynamic-linker".to_string());
            command.push(dynamic_linker.to_string());
        }

        // Standard library paths and startup files
        if !self.static_link && !self.shared {
            self.add_standard_startup_files(&mut command)?;
        }

        // Library search paths
        for path in &self.library_paths {
            command.push("-L".to_string());
            command.push(path.to_string_lossy().to_string());
        }

        // Add standard library paths
        self.add_standard_library_paths(&mut command)?;

        // Object files
        for obj in &self.object_files {
            command.push(obj.to_string_lossy().to_string());
        }

        // Libraries
        for lib in &self.libraries {
            command.push("-l".to_string());
            command.push(lib.clone());
        }

        // Standard libraries
        if !self.static_link {
            command.push("-lc".to_string());
        }

        Ok(command)
    }

    fn add_standard_startup_files(&self, _command: &mut Vec<String>) -> Result<()> {
        // Skip startup files when we have our own _start
        // This prevents conflicts with our custom _start implementation
        Ok(())
    }

    fn add_standard_library_paths(&self, command: &mut Vec<String>) -> Result<()> {
        let lib_paths = match self.target {
            Target::I386 => vec![
                "/usr/lib/i386-linux-gnu",
                "/lib/i386-linux-gnu",
                "/usr/lib32",
                "/lib32",
            ],
            Target::Amd64 => vec![
                "/usr/lib/x86_64-linux-gnu",
                "/lib/x86_64-linux-gnu",
                "/usr/lib64",
                "/lib64",
            ],
            Target::Arm64 => vec![
                "/usr/lib/aarch64-linux-gnu",
                "/lib/aarch64-linux-gnu",
            ],
        };

        for path in lib_paths {
            command.push("-L".to_string());
            command.push(path.to_string());
        }

        // Add GCC library path
        let gcc_lib = self.get_gcc_lib_path()?;
        command.push("-L".to_string());
        command.push(gcc_lib);

        Ok(())
    }

    fn get_gcc_lib_path(&self) -> Result<String> {
        // Try to find GCC library path
        let output = Command::new("gcc")
            .args(&["-print-libgcc-file-name"])
            .output()
            .map_err(|e| AleccError::LinkerError {
                message: format!("Failed to find GCC library path: {}", e),
            })?;

        if !output.status.success() {
            return Err(AleccError::LinkerError {
                message: "Failed to determine GCC library path".to_string(),
            });
        }

        let libgcc_path = String::from_utf8_lossy(&output.stdout);
        let libgcc_path = libgcc_path.trim();
        
        if let Some(parent) = Path::new(libgcc_path).parent() {
            Ok(parent.to_string_lossy().to_string())
        } else {
            Err(AleccError::LinkerError {
                message: "Invalid GCC library path".to_string(),
            })
        }
    }

    pub async fn link_shared_library(&self, soname: Option<&str>) -> Result<()> {
        let mut command = self.build_linker_command()?;
        
        // Remove executable-specific flags
        command.retain(|arg| arg != "-pie" && !arg.starts_with("-dynamic-linker"));
        
        // Add shared library flags
        if !command.contains(&"-shared".to_string()) {
            command.push("-shared".to_string());
        }
        
        if let Some(soname) = soname {
            command.push("-soname".to_string());
            command.push(soname.to_string());
        }

        let output = Command::new(&command[0])
            .args(&command[1..])
            .output()
            .map_err(|e| AleccError::LinkerError {
                message: format!("Failed to execute linker: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AleccError::LinkerError {
                message: format!("Shared library linking failed: {}", stderr),
            });
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn link_static_library(&self) -> Result<()> {
        // Use ar to create static library
        let mut command = vec!["ar".to_string(), "rcs".to_string()];
        command.push(self.output_path.to_string_lossy().to_string());
        
        for obj in &self.object_files {
            command.push(obj.to_string_lossy().to_string());
        }

        let output = Command::new(&command[0])
            .args(&command[1..])
            .output()
            .map_err(|e| AleccError::LinkerError {
                message: format!("Failed to execute ar: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AleccError::LinkerError {
                message: format!("Static library creation failed: {}", stderr),
            });
        }

        Ok(())
    }
}
