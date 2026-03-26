use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("y1 yarn port requires a subcommand");
        return ExitCode::from(99);
    }

    eprintln!("y1 yarn port rejected: {}", args.join(" "));
    ExitCode::from(99)
}
