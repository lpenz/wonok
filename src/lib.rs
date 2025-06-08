// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use atomic_write_file::AtomicWriteFile;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::process;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use tracing::info;

#[tracing::instrument(ret)]
pub fn buildcmd(command: &[&str]) -> Command {
    let mut cmd = Command::new(command[0]);
    cmd.args(command.iter().skip(1))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

#[tracing::instrument(skip_all, ret, err)]
fn run(output: &str, command: &[&str]) -> Result<ExitStatus> {
    info!(output = output, command = ?command);
    let mut child = buildcmd(command).spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or(eyre!("Error taking stdout of process"))?;
    // Use a buffer with 8MiB
    let mut buffer: Vec<u8> = vec![0u8; 1024 * 1024 * 8];
    let mut file = AtomicWriteFile::options().open(output)?;
    loop {
        let n = stdout.read(&mut buffer)?;
        if n == 0 {
            if let Some(result) = child.try_wait()? {
                info!(msg="process exited", result=?result);
                if result.success() {
                    file.commit()?;
                }
                return Ok(result);
            }
        }
        file.write_all(&buffer[0..n])?;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output file
    pub output: String,

    /// The command to run; use stdin if empty (pipe mode)
    pub command: Vec<String>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    let command: Vec<&str> = cli.command.iter().map(String::as_ref).collect();
    let exitstatus = run(&cli.output, &command)?;
    process::exit(exitstatus.code().unwrap_or(0));
}
