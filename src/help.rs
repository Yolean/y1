use std::process::ExitCode;

const HELP_TEXT: &str = "
  y1 (yarn v1.22.22 task runner)

  Usage: yarn [command] [flags]

  Displays help information.

  Options:

    -s, --silent                        skip Yarn console logs, other types of logs (script output) will be printed
    -h, --help                          output usage information
  Commands:
    - help
    - run

  Run `yarn help COMMAND` for more information on specific commands.

";

const HELP_RUN_TEXT: &str = "
  y1 (yarn v1.22.22 task runner)

  Usage: yarn run [script] [-- <args>]

  Runs a defined package script.

  Options:

    -s, --silent                        skip Yarn console logs, other types of logs (script output) will be printed
    -h, --help                          output usage information

";

pub fn print_help() -> ExitCode {
    print!("{HELP_TEXT}");
    ExitCode::from(0)
}

pub fn print_help_run() -> ExitCode {
    print!("{HELP_RUN_TEXT}");
    ExitCode::from(0)
}
