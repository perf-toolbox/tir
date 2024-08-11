use std::collections::HashMap;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Memory {
    pub address: u64,
    pub value: i64,
    pub value_size: u8,
    pub region_size: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub register_state: HashMap<String, u64>,
    pub memory: Vec<Memory>,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub experiment: String,
    pub input: String,
}
