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
    pub memory: Option<Vec<Memory>>,
    pub page_size: u64,
    pub map_faults_to_address: Option<u64>,
    pub prohibit_unaligned_access: Option<bool>,
    pub repeat: Option<u32>,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub experiment: String,
    pub input: String,
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub dump_memory_before: bool,
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub dump_memory_after: bool,
}
