use clap::{Parser, Subcommand};
use images::build::build_from_instructions;
use std::fs;
use tugfile::parser::parse_line;

#[derive(Parser)]
#[command(name = "tug")]
#[command(about = "Container builder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build => {
            let contents = fs::read_to_string("Tugfile").expect("Failed to read Tugfile");

            let instructions = contents.lines().filter_map(parse_line).collect();
            build_from_instructions(instructions);
        }
    }
}
