use base::Architecture;
use std::error::Error;
use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};

use crate::config::AnalysisConfig;

pub(super) fn run_mca(
    data: &str,
    arch: &Architecture,
    config: &AnalysisConfig,
) -> Result<String, Box<dyn Error>> {
    info!("Running MCA with config: {:#?}", config);
    let mut command = Command::new("llvm-mca");
    let command = &mut command;

    command
        .arg(format!("-march={}", arch.to_llvm()))
        .arg("-all-stats")
        .arg("-all-views")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(cpu) = config.default_cpus.get(arch) {
        command.arg(format!("-mcpu={}", cpu));
    } else if !arch.is_host() {
        // MCA defaults the cpu to be the host cpu, if we are running for a
        // different architecture, then we need to pass generic through.
        command.arg("-mcpu=generic");
    }

    let mut command = command.spawn()?;
    let input = command
        .stdin
        .as_mut()
        .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "Failed to get stdin for llvm-mca"))?
        .write_all(data.as_bytes())?;
    // Drop input to close stdin
    drop(input);

    let result = command.wait_with_output()?;
    if result.status.success() {
        Ok(String::from_utf8(result.stdout)?)
    } else {
        let error = String::from_utf8(result.stderr)?;
        let mut lines = error.trim().split('\n').collect::<Vec<_>>();
        lines.dedup();

        Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            format!("LLVM-MCA returned error ({})", lines.join("\n")),
        )))
    }
}
