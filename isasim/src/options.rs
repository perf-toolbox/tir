use std::collections::HashMap;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub register_state: HashMap<String, u64>,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub experiment: String,
    pub input: String,
}
