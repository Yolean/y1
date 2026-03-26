const YARN_BUILTINS: &[&str] = &[
    "access",
    "add",
    "audit",
    "autoclean",
    "bin",
    "cache",
    "check",
    "config",
    "create",
    "exec",
    "generate-lock-entry",
    "generateLockEntry",
    "global",
    "import",
    "info",
    "init",
    "install",
    "licenses",
    "link",
    "list",
    "login",
    "logout",
    "node",
    "outdated",
    "owner",
    "pack",
    "policies",
    "publish",
    "remove",
    "tag",
    "team",
    "unlink",
    "unplug",
    "upgrade",
    "upgrade-interactive",
    "upgradeInteractive",
    "version",
    "versions",
    "why",
    "workspace",
    "workspaces",
];

pub enum Command {
    Run {
        task: Option<String>,
        extra_args: Vec<String>,
    },
    Help,
    HelpRun,
    Rejected {
        args: Vec<String>,
    },
    NoArgs,
}

pub fn classify(args: &[String]) -> Command {
    if args.is_empty() {
        return Command::NoArgs;
    }

    match args[0].as_str() {
        "run" => parse_run(&args[1..]),
        "help" => {
            if args.len() >= 2 && args[1] == "run" {
                Command::HelpRun
            } else if args.len() == 1 {
                Command::Help
            } else {
                Command::Rejected {
                    args: args.to_vec(),
                }
            }
        }
        "--help" | "-h" => Command::Help,
        word if YARN_BUILTINS.contains(&word) => Command::Rejected {
            args: args.to_vec(),
        },
        _ => {
            // Shorthand: yarn <task> = yarn run <task>
            parse_run(args)
        }
    }
}

fn parse_run(args: &[String]) -> Command {
    if args.is_empty() {
        return Command::Run {
            task: None,
            extra_args: vec![],
        };
    }

    let task = args[0].clone();
    let rest = &args[1..];

    // Find -- separator for extra args
    let extra_args = if let Some(pos) = rest.iter().position(|a| a == "--") {
        let after_dash = &rest[pos + 1..];
        // Support both "-- --extra" (y1 deviation) and "-- -- --extra" (yarn convention)
        if !after_dash.is_empty() && after_dash[0] == "--" {
            after_dash[1..].to_vec()
        } else {
            after_dash.to_vec()
        }
    } else {
        vec![]
    };

    Command::Run {
        task: Some(task),
        extra_args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        if s.is_empty() {
            vec![]
        } else {
            s.split_whitespace().map(String::from).collect()
        }
    }

    #[test]
    fn no_args() {
        assert!(matches!(classify(&args("")), Command::NoArgs));
    }

    #[test]
    fn run_with_task() {
        let cmd = classify(&args("run test"));
        match cmd {
            Command::Run {
                task: Some(t),
                extra_args,
            } => {
                assert_eq!(t, "test");
                assert!(extra_args.is_empty());
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn run_no_task() {
        let cmd = classify(&args("run"));
        assert!(matches!(
            cmd,
            Command::Run {
                task: None,
                extra_args: _
            }
        ));
    }

    #[test]
    fn shorthand_task() {
        let cmd = classify(&args("test"));
        match cmd {
            Command::Run {
                task: Some(t),
                extra_args,
            } => {
                assert_eq!(t, "test");
                assert!(extra_args.is_empty());
            }
            _ => panic!("expected Run via shorthand"),
        }
    }

    #[test]
    fn rejected_install() {
        assert!(matches!(
            classify(&args("install")),
            Command::Rejected { .. }
        ));
    }

    #[test]
    fn rejected_add() {
        assert!(matches!(classify(&args("add foo")), Command::Rejected { .. }));
    }

    #[test]
    fn help() {
        assert!(matches!(classify(&args("help")), Command::Help));
        assert!(matches!(classify(&args("--help")), Command::Help));
        assert!(matches!(classify(&args("-h")), Command::Help));
    }

    #[test]
    fn help_run() {
        assert!(matches!(classify(&args("help run")), Command::HelpRun));
    }

    #[test]
    fn help_unknown_rejected() {
        assert!(matches!(
            classify(&args("help add")),
            Command::Rejected { .. }
        ));
    }

    #[test]
    fn double_dash_extra_args() {
        let cmd = classify(&args("run test -- -- --extra"));
        match cmd {
            Command::Run {
                task: Some(t),
                extra_args,
            } => {
                assert_eq!(t, "test");
                assert_eq!(extra_args, vec!["--extra"]);
            }
            _ => panic!("expected Run with extra args"),
        }
    }

    #[test]
    fn single_dash_extra_args() {
        let cmd = classify(&args("run test -- --extra"));
        match cmd {
            Command::Run {
                task: Some(t),
                extra_args,
            } => {
                assert_eq!(t, "test");
                assert_eq!(extra_args, vec!["--extra"]);
            }
            _ => panic!("expected Run with extra args"),
        }
    }

    #[test]
    fn all_builtins_rejected() {
        for builtin in YARN_BUILTINS {
            let cmd = classify(&[builtin.to_string()]);
            assert!(
                matches!(cmd, Command::Rejected { .. }),
                "{builtin} should be rejected"
            );
        }
    }
}
