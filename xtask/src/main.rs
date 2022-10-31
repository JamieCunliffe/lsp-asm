use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Command;

use commands::BuildDocArgs;

mod commands;

type DynError = Box<dyn std::error::Error + Sync + Send>;

fn main() {
    pretty_env_logger::init();
    if let Err(err) = try_main() {
        eprintln!("{err}");
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let args = commands::get_args();

    std::env::set_current_dir(project_root())?;

    match args {
        commands::Command::Test(args) => test(args.coverage),
        commands::Command::BuildDocs(args) => build_docs(&args),
    }
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
            .args([cmd])
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
        panic!("Command failed ({cmd})- todo handle better");
    }
}

fn build(env_vars: &HashMap<&str, &str>) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run_command(
        format!("{cargo} build --all --exclude xtask --color always").as_str(),
        env_vars,
    )?;

    Ok(())
}

fn clean(env_vars: &HashMap<&str, &str>) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run_command(format!("{cargo} clean").as_str(), env_vars)
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
    run_command(format!("{cargo} test --color always").as_str(), &env)?;
    if generate_coverage {
        run_command(
            r#"grcov . -s . --binary-path ./coverage/debug/ -t html --branch --ignore-not-existing -o ./coverage/debug/coverage/ --ignore "xtask/*""#,
            &env,
        )?;
    }

    Ok(())
}

fn build_docs(args: &BuildDocArgs) -> Result<(), DynError> {
    let mut tasks = Vec::new();
    let rt = tokio::runtime::Runtime::new()?;

    if args.aarch64 {
        tasks.push(rt.spawn(documentation_builder::build_aarch64_instructions()));
    }
    for task in tasks {
        rt.block_on(task)??;
    }

    Ok(())
}
