use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, os::unix::net::UnixStream};
use tugfile::{
    instruction::{self, Instruction},
    parser::parse_line,
};

#[derive(Parser)]
#[command(name = "tug")]
#[command(about = "Container builder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Serialize, Deserialize)]
pub struct TugRequest {
    pub command: String,
    pub instructions: Option<Vec<Instruction>>,
    pub image: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build,
}

pub fn run_cli() {
    let cli = Cli::parse();
    println!("cli command {:?}", cli.command);
    match cli.command {
        Commands::Build => {
            let contents = fs::read_to_string("Tugfile").expect("Failed to read Tugfile");
            // println!("contents in tugfile {}", contents);
            let instructions: Vec<Instruction> = contents.lines().filter_map(parse_line).collect();
            println!("insturctions {:?}", instructions);

            let req = TugRequest {
                command: "build".into(),
                instructions: Some(instructions),
                image: None,
            };

            let json = serde_json::to_string(&req).unwrap(); // converting instruction struct into json (serializing)

            let mut stream = UnixStream::connect("/run/tugd.sock").expect("connection fail");
            stream.write_all(json.as_bytes()).unwrap();
        }
    }
}
