use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Instruction {
    From(String),
    Run(String),
    Copy { src: String, dest: String },
    Cmd(String),
    Workdir(String),
    Expose(u16),
    Env { key: String, value: String },
    EntryPoint(String),
    Add { src: String, dest: String },
}
