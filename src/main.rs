mod commands;
mod help;
mod npx;
mod package_json;
mod run;

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv0 = std::env::args().next().unwrap_or_default();
    let bin_name = std::path::Path::new(&argv0)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    match bin_name {
        "npx" | "y-npx" => npx::run(),
        _ => yarn_main(),
    }
}

fn yarn_main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match commands::classify(&args) {
        commands::Command::NoArgs => {
            eprintln!("y1 yarn port requires a subcommand");
            ExitCode::from(99)
        }
        commands::Command::Rejected { args } => {
            eprintln!("y1 yarn port rejected: {}", args.join(" "));
            ExitCode::from(99)
        }
        commands::Command::Run {
            task: Some(task),
            extra_args,
        } => run::run_task(&task, &extra_args),
        commands::Command::Run { task: None, .. } => run::list_scripts(),
        commands::Command::Help => help::print_help(),
        commands::Command::HelpRun => help::print_help_run(),
    }
}
