use std::process::ExitCode;

fn normalize(s: &str) -> String {
    s.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_allowed(allowed_env: &str, separator: &str) -> Vec<String> {
    allowed_env
        .split(separator)
        .map(|s| normalize(s))
        .filter(|s| !s.is_empty())
        .collect()
}

fn find_system_npx() -> Option<std::path::PathBuf> {
    let self_exe = std::env::current_exe().ok().and_then(|p| std::fs::canonicalize(p).ok());
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        let candidate = std::path::Path::new(dir).join("npx");
        if !candidate.is_file() {
            continue;
        }
        if let Some(ref self_path) = self_exe {
            if let Ok(resolved) = std::fs::canonicalize(&candidate) {
                if &resolved == self_path {
                    continue;
                }
            }
        }
        return Some(candidate);
    }
    None
}

pub fn run() -> ExitCode {
    if std::env::var_os("Y_NPX_WRAPPER").is_some() {
        eprintln!("y-npx: loop detected (re-entered via exec), aborting");
        return ExitCode::from(1);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = normalize(&args.join(" "));

    let separator = std::env::var("Y_NPX_ALLOWED_CMDS_SEPARATOR").unwrap_or_else(|_| ",".into());
    let allowed_env = std::env::var("Y_NPX_ALLOWED_CMDS").unwrap_or_default();
    let allowed = parse_allowed(&allowed_env, &separator);

    if !allowed.iter().any(|a| a == &invocation) {
        eprintln!(
            "y-npx blocked npx because `{}` not found in Y_NPX_ALLOWED_CMDS (use installed tools)",
            args.join(" ")
        );
        return ExitCode::from(1);
    }

    let npx_path = match find_system_npx() {
        Some(p) => p,
        None => {
            eprintln!("y-npx: no system npx found in PATH");
            return ExitCode::from(1);
        }
    };

    let err = exec(&npx_path, &args);
    eprintln!("y-npx: failed to exec {}: {err}", npx_path.display());
    ExitCode::from(1)
}

#[cfg(unix)]
fn exec(path: &std::path::Path, args: &[String]) -> std::io::Error {
    use std::os::unix::process::CommandExt;
    std::process::Command::new(path)
        .args(args)
        .env("Y_NPX_WRAPPER", "y1")
        .exec()
}

#[cfg(not(unix))]
fn exec(path: &std::path::Path, args: &[String]) -> std::io::Error {
    match std::process::Command::new(path)
        .args(args)
        .env("Y_NPX_WRAPPER", "y1")
        .status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_collapses() {
        assert_eq!(normalize("  tsc   --noEmit  "), "tsc --noEmit");
        assert_eq!(normalize("tsc"), "tsc");
        assert_eq!(normalize("  "), "");
    }

    #[test]
    fn parse_allowed_comma_separator() {
        let allowed = parse_allowed("tsc --noEmit , eslint . ,  ", ",");
        assert_eq!(allowed, vec!["tsc --noEmit", "eslint ."]);
    }

    #[test]
    fn parse_allowed_custom_separator() {
        let allowed = parse_allowed("tsc --noEmit|eslint .", "|");
        assert_eq!(allowed, vec!["tsc --noEmit", "eslint ."]);
    }

    #[test]
    fn parse_allowed_empty() {
        let allowed = parse_allowed("", ",");
        assert!(allowed.is_empty());
    }

    #[test]
    fn exact_match_required() {
        let allowed = parse_allowed("tsc --noEmit", ",");
        assert!(allowed.iter().any(|a| a == "tsc --noEmit"));
        assert!(!allowed.iter().any(|a| a == "tsc"));
        assert!(!allowed.iter().any(|a| a == "tsc --noEmit --watch"));
    }
}
