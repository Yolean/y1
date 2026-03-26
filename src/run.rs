use std::io::Write;
use std::process::{Command, ExitCode};
use std::time::Instant;

use crate::package_json;

const VERSION: &str = "1.22.22";

// Task result messages used by log processing pipelines.
// These formats are byte-for-byte identical to yarn v1.22.22 output.
// The success message goes to stdout; the failure message goes to stderr.
const DONE_PREFIX: &str = "Done in ";
const DONE_SUFFIX: &str = "s.";
const ERROR_PREFIX: &str = "error Command failed with exit code ";
const ERROR_SUFFIX: &str = ".";

fn format_done(elapsed: f64) -> String {
    format!("{DONE_PREFIX}{elapsed:.2}{DONE_SUFFIX}")
}

fn format_error(code: i32) -> String {
    format!("{ERROR_PREFIX}{code}{ERROR_SUFFIX}")
}

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
                println!("{}", format_done(elapsed));
                ExitCode::from(0)
            } else {
                std::io::stdout().flush().ok();
                eprintln!("{}", format_error(code));
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
    println!("{}", format_done(elapsed));
    std::io::stdout().flush().ok();

    eprintln!("error There are no binary scripts available.");
    eprintln!("error No command specified.");

    ExitCode::from(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests assert byte-level fidelity of task result messages against
    // yarn v1.22.22's actual output. These lines are parsed by log processing
    // pipelines and must never change.

    #[test]
    fn done_message_bytes() {
        let msg = format_done(0.02);
        assert_eq!(msg, "Done in 0.02s.");
        // Verify exact bytes: "Done in " + digits + "s."
        let bytes = msg.as_bytes();
        assert_eq!(&bytes[..8], b"Done in ");
        assert_eq!(&bytes[bytes.len() - 2..], b"s.");
    }

    #[test]
    fn done_message_to_stdout() {
        // format_done is printed via println! (stdout), not eprintln! (stderr).
        // This is verified structurally: format_done returns a String that
        // run_task passes to println!, and format_error to eprintln!.
        // The fixture e2e tests verify the actual stream assignment.
        let msg = format_done(1.50);
        assert_eq!(msg, "Done in 1.50s.");
    }

    #[test]
    fn error_message_bytes() {
        let msg = format_error(123);
        assert_eq!(msg, "error Command failed with exit code 123.");
        // Verify prefix and suffix bytes exactly match yarn v1.22.22
        assert!(msg.starts_with(ERROR_PREFIX));
        assert!(msg.ends_with(ERROR_SUFFIX));
        // The variable part (exit code) is between prefix and suffix
        let code_str = &msg[ERROR_PREFIX.len()..msg.len() - ERROR_SUFFIX.len()];
        assert_eq!(code_str, "123");
    }

    #[test]
    fn error_message_to_stderr() {
        // format_error is printed via eprintln! (stderr), not println! (stdout).
        // Verified structurally and by fixture e2e tests.
        let msg = format_error(1);
        assert_eq!(msg, "error Command failed with exit code 1.");
    }

    #[test]
    fn done_message_matches_yarn_format() {
        // Yarn v1.22.22 outputs exactly: "Done in " + duration + "s.\n" to stdout
        // The \n comes from println!, not from format_done.
        for (secs, expected) in [
            (0.02, "Done in 0.02s."),
            (0.00, "Done in 0.00s."),
            (1.50, "Done in 1.50s."),
            (99.99, "Done in 99.99s."),
        ] {
            assert_eq!(format_done(secs), expected, "mismatch for {secs}");
        }
    }

    #[test]
    fn error_message_matches_yarn_format() {
        // Yarn v1.22.22 outputs exactly: "error Command failed with exit code " + N + ".\n" to stderr
        for (code, expected) in [
            (1, "error Command failed with exit code 1."),
            (2, "error Command failed with exit code 2."),
            (123, "error Command failed with exit code 123."),
            (255, "error Command failed with exit code 255."),
        ] {
            assert_eq!(format_error(code), expected, "mismatch for code {code}");
        }
    }
}
