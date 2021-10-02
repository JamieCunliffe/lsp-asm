use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

use clap::{App, AppSettings, Arg, ArgMatches};

type DynError = Box<dyn std::error::Error + Sync + Send>;

fn main() {
    pretty_env_logger::init();
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
            ))
        .subcommand(
            App::new("build-docs")
                .about("Download and build instruction set documentation")
                .arg(
                    Arg::with_name("aarch64")
                        .long("aarch64")
                        .help("Downloads and builds the documentation for the AArch64 instruction set from arm.com")
                        .takes_value(false),
                ),
        )
        .get_matches();

    std::env::set_current_dir(project_root())?;

    match matches.subcommand() {
        ("test", Some(args)) => test(args.is_present("coverage"))?,
        ("build-docs", Some(args)) => build_docs(args)?,
        _ => panic!("Invalid subcommand specified - Have you added a subcommand without adding it to the match?"),
    };

    Ok(())
}

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
        env.insert("CARGO_TARGET_DIR", "coverage");
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
            r#"grcov . -s . --binary-path ./coverage/debug/ -t html --branch --ignore-not-existing -o ./coverage/debug/coverage/ --ignore "xtask/*""#,
            &env,
        )?;
    }

    Ok(())
}

fn build_docs(args: &ArgMatches) -> Result<(), DynError> {
    let mut tasks = Vec::new();
    let rt = tokio::runtime::Runtime::new()?;

    if args.is_present("aarch64") {
        tasks.push(rt.spawn(documentation_builder::build_aarch64_instructions()));
    }
    for task in tasks {
        rt.block_on(task)??;
    }

    Ok(())
}
