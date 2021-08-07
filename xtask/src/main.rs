use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

use clap::{App, AppSettings, Arg};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let matches = App::new("lsp-asm build system")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new("test").about("Run unit tests").arg(
                Arg::with_name("coverage")
                    .long("coverage")
                    .help("Generate coverage report")
                    .takes_value(false)
                    .required(false),
            ),
        )
        .get_matches();

    match matches.subcommand() {

        ("test", Some(args)) => test(args.is_present("coverage"))?,
        _ => panic!("Invalid subcommand specified - Have you added a subcommand without adding it to the match?"),
    };

    Ok(())
}

#[allow(unused)]
fn project_root() -> String {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn run_command(cmd: &str, env_vars: &HashMap<&str, &str>) -> Result<(), DynError> {
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .envs(env_vars)
            .args(&[cmd])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .envs(env_vars)
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };

    println!("{}", std::str::from_utf8(output.stdout.as_slice())?);
    println!("{}", std::str::from_utf8(output.stderr.as_slice())?);
    if output.status.success() {
        Ok(())
    } else {
        panic!("Command failed ({})- todo handle better", cmd);
    }
}

fn build(env_vars: &HashMap<&str, &str>) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run_command(
        format!("{} build --all --exclude xtask --color always", cargo).as_str(),
        env_vars,
    )?;

    Ok(())
}

fn clean(env_vars: &HashMap<&str, &str>) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run_command(format!("{} clean", cargo).as_str(), env_vars)
}

fn test(generate_coverage: bool) -> Result<(), DynError> {
    let mut env = HashMap::new();

    if generate_coverage {
        clean(&env)?;

        env.insert("CARGO_INCREMENTAL", "0");
        env.insert(
            "RUSTFLAGS",
            "-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off",
        );
        env.insert("RUSTDOCFLAGS", "-Cpanic=abort");
        env.insert("RUSTC_BOOTSTRAP", "1");
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    build(&env)?;
    run_command(format!("{} test --color always", cargo).as_str(), &env)?;
    if generate_coverage {
        run_command(
            r#"grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --ignore "xtask/*""#,
            &env,
        )?;
    }
    Ok(())
}
