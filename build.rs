// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::Zsh;
use color_eyre::{Result, eyre::eyre};
use man::prelude::*;
use std::env;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path;

include!("src/cli.rs");

fn generate_man_page<P: AsRef<path::Path>>(outdir: P) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join("wonok.1");
    let cmd = Cli::command();
    let manpage: Manual = clap2man::Manual::try_from(&cmd)
        .map_err(|e| eyre!(e))?
        .into();
    let manpage = manpage
        .example(
            Example::new()
                .text("Atomically write the output of 'ls' to 'files.txt' if it succeeds")
                .command("wonok files.txt ls"),
        )
        .render();
    File::create(man_path)?.write_all(manpage.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
    let mut outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    fs::create_dir_all(&outdir)?;
    generate_man_page(&outdir)?;
    // build/wonok-*/out
    outdir.pop();
    // build/wonok-*
    outdir.pop();
    // build
    outdir.pop();
    // .
    // (either target/release or target/build)
    generate_man_page(&outdir)?;
    // Generate shell completions:
    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, "wonok", &outdir)?;
    let path = generate_to(Fish, &mut cmd, "wonok", &outdir)?;
    let mut fd = OpenOptions::new().append(true).open(path)?;
    writeln!(fd, "complete -c wonok --wraps command")?;
    writeln!(fd, "complete -c wonok --no-files")?;
    generate_to(Zsh, &mut cmd, "wonok", &outdir)?;
    Ok(())
}
