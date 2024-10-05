use glob::glob;
use map_ok::MapOk;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{PathBuf, Path};
use std::process::{Command, Output};
use unescaper::unescape;

#[derive(Debug, Deserialize)]
struct GlobalConfig {
    suite: Suite,
}

#[derive(Debug, Deserialize)]
struct Suite {
    name: String,
    glob: Vec<String>,
}

fn resolve_path() -> String {
    let path = env::var("PATH").unwrap().to_string();

    let cur_exe = env::current_exe().unwrap();
    let target_dir = cur_exe.parent().unwrap().to_str().unwrap();

    format!("{}:{}", target_dir, path)
}

fn run_command(command: &str, test_path: &Path) -> Result<Output, std::io::Error> {
    let words = shlex::split(command)
        .unwrap()
        .iter()
        .map(|term| {
            let term = term.to_string();
            let term = term.replace("%s", test_path.to_str().unwrap());
            let term = term.replace(
                "%S",
                test_path.parent().unwrap().to_str().unwrap(),
            );
            term
        })
        .collect::<Vec<String>>();

    let script = words.join(" ");

    let mut filtered_env: HashMap<String, String> = env::vars()
        .filter(|(k, _)| k == "TERM" || k == "TZ" || k == "LANG" || k == "LD_LIBRARY_PATH")
        .collect();
    filtered_env.insert("PATH".to_string(), resolve_path());

    Command::new("bash")
        .args(["-c", &script])
        .env_clear()
        .envs(&filtered_env)
        .output()
}

fn run_test(test: &PathBuf) -> bool {
    let path_str = test.to_str().unwrap();

    let test_contents = std::fs::read_to_string(test).expect("Failed to read test file");
    let lines = test_contents.lines();

    let re = Regex::new(".*RUN:(.*)$").unwrap();

    let run_lines = lines
        .flat_map(|line| re.captures_iter(line))
        .filter_map(|c| c.get(1).map(|c| c.as_str().trim()))
        .collect::<Vec<&str>>();

    let mut has_failures = false;

    for command in run_lines {
        print!("test {} ... ", path_str);
        match run_command(command, test) {
            Err(_) => {
                has_failures = true;
                print!("fail");
            }
            Ok(output) => {
                if output.status.success() {
                    print!("ok");
                } else {
                    has_failures = true;
                    let stdout = unescape(&String::from_utf8(output.stdout).unwrap()).unwrap();
                    eprintln!("STDOUT: \n{}", stdout);
                    let stderr = unescape(&String::from_utf8(output.stderr).unwrap()).unwrap();
                    eprintln!("STDERR: \n{}", stderr);
                    print!("fail")
                }
            }
        }

        println!(" : test")
    }

    has_failures
}

pub fn main() -> Result<(), String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR env var not defined".to_string())?;

    let configs = glob(&format!("{}/../../**/test_suite.toml", manifest_dir))
        .map_err(|_| "Failed to glob test directories".to_string())?
        .map_ok(|entry| {
            let config = std::fs::read_to_string(&entry).expect("Failed to read file");
            let config: GlobalConfig = toml::from_str(&config).expect("Failed to decode a file");

            (entry.canonicalize().unwrap(), config.suite)
        })
        .filter_map(|suite| suite.ok())
        .collect::<Vec<(PathBuf, Suite)>>();

    let mut has_failures = false;
    for (path, suite) in configs {
        eprintln!("Running {}", &suite.name);

        let path = path.parent().unwrap().to_str().unwrap();

        let tests = suite
            .glob
            .iter()
            .flat_map(|pattern| {
                glob(&format!("{}/{}", path, &pattern)).expect("Failed to glob tests")
            })
            .filter_map(|test| test.ok())
            .collect::<Vec<_>>();

        for test in tests {
            has_failures |= run_test(&test);
        }
    }

    if has_failures {
        std::process::exit(1);
    }

    Ok(())
}
