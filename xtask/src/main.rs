use std::{env, path::PathBuf};
use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let task = env::args().nth(1);
    let sh = Shell::new()?;
    match task.as_deref() {
        Some("help") => print_help(),
        Some("build") => build(&sh)?,
        Some("check") => {
            build(&sh)?;
            check(&sh)?
        }
        Some("check-only") => check(&sh)?,
        Some("docs") => build_docs(&sh)?,
        _ => print_help(),
    }
    Ok(())
}

fn build(sh: &Shell) -> anyhow::Result<()> {
    let root = project_root();
    sh.change_dir(root);

    cmd!(sh, "cargo build").run()?;

    Ok(())
}

fn check(sh: &Shell) -> anyhow::Result<()> {
    let root = project_root();
    sh.change_dir(root);

    cmd!(sh, "cargo run --bin check-runner").run()?;

    Ok(())
}

fn build_docs(sh: &Shell) -> anyhow::Result<()> {
    let root = project_root();
    sh.change_dir(&root);

    cmd!(sh, "cargo doc --no-deps").run()?;

    let api_dest = root.join("docs/api");
    if std::fs::read_dir(&api_dest).is_ok() {
        std::fs::remove_dir_all(&api_dest)?;
    }

    let api_src = root.join("target/doc");
    std::fs::rename(api_src, api_dest)?;

    cmd!(sh, "mdbook build").run()?;

    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

build            builds TIR project
check            builds project and runs check tests
check-only       only runs check tests without building the project
docs             builds project documentation
help             shows this message
"
    )
}

fn project_root() -> PathBuf {
    let dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());
    PathBuf::from(dir).parent().unwrap().to_owned()
}
