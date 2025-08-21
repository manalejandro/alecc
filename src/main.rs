use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, error};

mod compiler;
mod lexer;
mod parser;
mod codegen;
mod optimizer;
mod linker;
mod targets;
mod cli;
mod error;

use compiler::Compiler;
use cli::Args;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    
    info!("Starting ALECC compiler v{}", env!("CARGO_PKG_VERSION"));
    
    let mut compiler = Compiler::new(args.clone())?;
    
    match compiler.compile().await {
        Ok(()) => {
            info!("Compilation completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}
