use anyhow::Result;
use clap::Parser;
use tracing::{error, info};

mod cli;
mod codegen;
mod compiler;
mod error;
mod lexer;
mod linker;
mod optimizer;
mod parser;
mod targets;

use cli::Args;
use compiler::Compiler;

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
