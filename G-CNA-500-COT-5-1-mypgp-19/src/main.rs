#![forbid(unsafe_code)]

use my_pgp_lib::cli::args::{is_help_requested, parse, usage, validate_args};
use my_pgp_lib::cli::dispatch::run;
use std::env;
use std::process;

const EXIT_FAILURE: i32 = 84;

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    if is_help_requested(&argv) {
        print!("{}", usage());
        return;
    }
    let result = parse(&argv).and_then(|args| {
        validate_args(&args)?;
        run(&args)
    });
    if let Err(error) = result {
        eprintln!("my_pgp: {}", error);
        process::exit(EXIT_FAILURE);
    }
}
