use std::io::Write;
use std::process::{Command, ExitCode};
use std::time::Instant;

use crate::package_json;

const VERSION: &str = "1.22.22";

fn print_header() {
    println!("yarn run v{VERSION}");
}

fn build_path() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let node_bin = cwd.join("node_modules/.bin");
    let current_path = std::env::var("PATH").unwrap_or_default();
    if node_bin.exists() {
        format!("{}:{current_path}", node_bin.display())
    } else {
        current_path
    }
}

pub fn run_task(task: &str, extra_args: &[String]) -> ExitCode {
    print_header();

    let scripts = match package_json::read_scripts() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error {e}");
            return ExitCode::from(1);
        }
    };

    let script = match scripts.get(task) {
        Some(s) => s.clone(),
        None => {
            eprintln!("error Command \"{task}\" not found.");
            return ExitCode::from(1);
        }
    };

    let full_script = if extra_args.is_empty() {
        script.clone()
    } else {
        format!("{script} {}", extra_args.join(" "))
    };

    println!("$ {full_script}");
    std::io::stdout().flush().ok();

    let start = Instant::now();
    let path = build_path();
    let init_cwd = std::env::current_dir()
        .unwrap_or_default()
        .display()
        .to_string();

    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(&full_script)
        .env("PATH", &path)
        .env("INIT_CWD", &init_cwd)
        .status();

    let elapsed = start.elapsed().as_secs_f64();

    match status {
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            if code == 0 {
                println!("Done in {elapsed:.2}s.");
                ExitCode::from(0)
            } else {
                std::io::stdout().flush().ok();
                eprintln!("error Command failed with exit code {code}.");
                ExitCode::from(code as u8)
            }
        }
        Err(e) => {
            eprintln!("error Failed to execute script: {e}");
            ExitCode::from(1)
        }
    }
}

pub fn list_scripts() -> ExitCode {
    print_header();

    let start = Instant::now();

    let scripts = match package_json::read_scripts() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error {e}");
            return ExitCode::from(1);
        }
    };

    println!("info Project commands");
    for (name, cmd) in &scripts {
        println!("   - {name}");
        println!("      {cmd}");
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("Done in {elapsed:.2}s.");
    std::io::stdout().flush().ok();

    eprintln!("error There are no binary scripts available.");
    eprintln!("error No command specified.");

    ExitCode::from(0)
}
