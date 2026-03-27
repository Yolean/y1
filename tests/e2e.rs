use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn y1_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_y1"));
    // Resolve to absolute path
    if path.is_relative() {
        path = std::env::current_dir().unwrap().join(path);
    }
    path
}

fn normalize_output(s: &str) -> String {
    // Normalize only the duration number, preserving all surrounding bytes.
    // "Done in 0.02s." becomes "Done in {DURATION}."
    // The prefix "Done in " and suffix "s." must match byte-for-byte.
    let re = regex::Regex::new(r"(Done in )\d+\.\d+(s\.)").unwrap();
    let s = re.replace_all(s, "${1}{DURATION}.").to_string();
    // Normalize cwd references
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    s.replace(manifest_dir, "{CWD}")
}

fn read_fixture_file(path: &Path) -> String {
    if path.exists() {
        fs::read_to_string(path).unwrap()
    } else {
        String::new()
    }
}

fn discover_fixture_cases() -> Vec<PathBuf> {
    let dir = fixtures_dir();
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();
    cases.sort();
    cases
}

fn parse_fixture_env(case_dir: &Path) -> Vec<(String, String)> {
    let content = read_fixture_file(&case_dir.join("env"));
    content
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

fn setup_argv0_symlink(case_dir: &Path) -> Option<PathBuf> {
    let content = read_fixture_file(&case_dir.join("argv0"));
    let name = content.trim();
    if name.is_empty() {
        return None;
    }
    let link_path = case_dir.join(name);
    // Remove stale symlink if present
    let _ = fs::remove_file(&link_path);
    std::os::unix::fs::symlink(y1_binary(), &link_path).unwrap();
    Some(link_path)
}

fn run_fixture_case(case_dir: &Path) {
    let case_name = case_dir.file_name().unwrap().to_str().unwrap();

    let args_content = read_fixture_file(&case_dir.join("args"));
    let args: Vec<&str> = args_content.lines().filter(|l| !l.is_empty()).collect();

    let cwd_content = read_fixture_file(&case_dir.join("cwd"));
    let cwd = cwd_content.trim();
    let working_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(cwd);

    let expected_stdout = read_fixture_file(&case_dir.join("expected-stdout"));
    let expected_stderr = read_fixture_file(&case_dir.join("expected-stderr"));
    let expected_exitcode: i32 = read_fixture_file(&case_dir.join("expected-exitcode"))
        .trim()
        .parse()
        .unwrap_or(0);

    let env_vars = parse_fixture_env(case_dir);
    let symlink = setup_argv0_symlink(case_dir);
    let default_binary = y1_binary();
    let binary = symlink.as_deref().unwrap_or(&default_binary);

    let mut cmd = Command::new(binary);
    cmd.args(&args).current_dir(&working_dir);
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }
    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("[{case_name}] failed to execute y1: {e}"));

    let actual_stdout = normalize_output(&String::from_utf8_lossy(&output.stdout));
    let actual_stderr = normalize_output(&String::from_utf8_lossy(&output.stderr));
    let actual_exitcode = output.status.code().unwrap_or(-1);

    assert_eq!(
        actual_exitcode, expected_exitcode,
        "[{case_name}] exit code mismatch"
    );
    assert_eq!(
        actual_stdout, expected_stdout,
        "[{case_name}] stdout mismatch"
    );
    assert_eq!(
        actual_stderr, expected_stderr,
        "[{case_name}] stderr mismatch"
    );
}

/// Verify that expected-stdout matches yarn-stdout byte-for-byte after applying
/// the same deviations the generator applies (strip info Visit lines, normalize timing).
/// Only checks generated fixtures (those with yarn-* files).
/// This catches any unintentional drift between y1's output format and yarn's.
#[test]
fn yarn_byte_fidelity() {
    let cases = discover_fixture_cases();

    fn apply_deviations(s: &str) -> String {
        s.lines()
            .filter(|line| !line.starts_with("info Visit https://yarnpkg.com/en/docs/cli/"))
            .filter(|line| !line.starts_with("  Visit https://yarnpkg.com/en/docs/cli/"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    let mut failures = Vec::new();
    for case in &cases {
        let name = case.file_name().unwrap().to_str().unwrap();
        let yarn_stdout_path = case.join("yarn-stdout");
        if !yarn_stdout_path.exists() {
            continue; // manual fixture, no yarn reference
        }
        let yarn_stdout = apply_deviations(&read_fixture_file(&yarn_stdout_path));
        let expected_stdout = read_fixture_file(&case.join("expected-stdout"));

        if yarn_stdout.as_bytes() != expected_stdout.as_bytes() {
            failures.push(format!(
                "  {name}: yarn-stdout (after deviations) and expected-stdout differ at byte level"
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Byte fidelity check failed for {} fixture(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

#[test]
fn fixture_tests() {
    let cases = discover_fixture_cases();
    assert!(!cases.is_empty(), "no fixture cases found");

    let mut failures = Vec::new();
    for case in &cases {
        let name = case.file_name().unwrap().to_str().unwrap().to_string();
        let result = std::panic::catch_unwind(|| run_fixture_case(case));
        if let Err(e) = result {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            failures.push((name, msg));
        }
    }

    if !failures.is_empty() {
        let report: String = failures
            .iter()
            .map(|(name, msg)| format!("  FAIL {name}: {msg}"))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "{}/{} fixture tests failed:\n{report}",
            failures.len(),
            cases.len()
        );
    }
}
