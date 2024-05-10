use clap::{CommandFactory, FromArgMatches, Parser};
use litcheck_core::Input;
use litcheck_filecheck::Config;
use litcheck_filecheck::Test;

#[derive(Debug, Parser)]
#[command(name = "filecheck", arg_required_else_help(true))]
struct Cli {
    #[arg(value_name = "CHECK")]
    pub match_file: Input,
    #[arg(value_name = "VERIFY", default_value = "-")]
    pub input_file: Input,
    #[command(flatten)]
    pub config: Config,
}

fn main() {
    let cmd = Cli::command().mut_arg("allow_empty", |arg| arg.long("allow_empty"));
    let matches = cmd.get_matches();
    let args = Cli::from_arg_matches(&matches).unwrap();
    let match_file = args.match_file.into_source(true).unwrap();

    let mut config = args.config;
    config.comment_prefixes.sort();
    config.comment_prefixes.dedup();
    config.check_prefixes.sort();
    config.check_prefixes.dedup();

    let input_file = args.input_file.into_source(true).unwrap();
    let mut test = Test::new(match_file, &config);
    match test.verify(input_file) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("filecheck failed:\n{:?}", err);
            std::process::exit(1);
        }
    }
}
