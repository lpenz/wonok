// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "wonok runs the provided command and captures its stdout. If the command exits with success, wonok atomically writes the captured output to the specified file.\n\nThis ensures that the output file is only updated if the command completes successfully, preventing partial or corrupted files."
)]
pub struct Cli {
    /// Output file
    pub output: String,

    /// The command to run; use stdin if empty (pipe mode)
    pub command: Vec<String>,
}
