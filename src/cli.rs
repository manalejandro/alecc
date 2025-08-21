use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "alecc")]
#[command(about = "A high-performance C/C++ compiler with GCC compatibility")]
#[command(version)]
pub struct Args {
    /// Input source files
    #[arg(value_name = "FILE")]
    pub input_files: Vec<PathBuf>,

    /// Output file name
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Target architecture
    #[arg(short = 't', long = "target", default_value = "native")]
    pub target: String,

    /// Compilation mode
    #[arg(short = 'c', long = "compile")]
    pub compile_only: bool,

    /// Generate assembly only
    #[arg(short = 'S', long = "assemble")]
    pub assembly_only: bool,

    /// Preprocessing only
    #[arg(short = 'E', long = "preprocess")]
    pub preprocess_only: bool,

    /// Optimization level
    #[arg(short = 'O', long = "optimize", default_value = "0")]
    pub optimization: String,

    /// Debug information
    #[arg(short = 'g', long = "debug")]
    pub debug: bool,

    /// Warning level
    #[arg(short = 'W', long = "warn")]
    pub warnings: Vec<String>,

    /// Include directories
    #[arg(short = 'I', long = "include")]
    pub include_dirs: Vec<PathBuf>,

    /// Library directories
    #[arg(short = 'L', long = "library-path")]
    pub library_dirs: Vec<PathBuf>,

    /// Libraries to link
    #[arg(short = 'l', long = "library")]
    pub libraries: Vec<String>,

    /// Define preprocessor macros
    #[arg(short = 'D', long = "define")]
    pub defines: Vec<String>,

    /// Undefine preprocessor macros
    #[arg(short = 'U', long = "undefine")]
    pub undefines: Vec<String>,

    /// C standard version
    #[arg(long = "std")]
    pub standard: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Position independent code
    #[arg(long = "pic")]
    pub pic: bool,

    /// Position independent executable
    #[arg(long = "pie")]
    pub pie: bool,

    /// Static linking
    #[arg(long = "static")]
    pub static_link: bool,

    /// Shared library creation
    #[arg(long = "shared")]
    pub shared: bool,

    /// Thread model
    #[arg(long = "thread-model", default_value = "posix")]
    pub thread_model: String,

    /// Enable LTO
    #[arg(long = "lto")]
    pub lto: bool,

    /// Cross compilation sysroot
    #[arg(long = "sysroot")]
    pub sysroot: Option<PathBuf>,

    /// Additional compiler flags
    #[arg(long = "extra-flags")]
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
    Os,
    Oz,
}
