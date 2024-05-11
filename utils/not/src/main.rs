fn main() {
    let mut args = std::env::args().into_iter().skip(1);

    let exec = args.next();
    let cli_args: Vec<_> = args.collect();

    let mut cmd = std::process::Command::new(exec.unwrap());
    cmd.args(cli_args);

    match cmd.status() {
        Ok(status) => match status.code() {
            Some(0) => std::process::exit(1),
            Some(_) => std::process::exit(0),
            _ => std::process::exit(1),
        },
        _ => std::process::exit(1),
    }
}
