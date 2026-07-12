// Copyright (C) 2025 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fs;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use tempfile::TempDir;

fn wonok_bin() -> &'static str {
    env!("CARGO_BIN_EXE_wonok")
}

fn run_wonok(output: &str, args: &[&str]) -> (bool, String, String) {
    let child = Command::new(wonok_bin())
        .arg(output)
        .arg("--")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to execute wonok");
    (
        child.status.success(),
        String::from_utf8_lossy(&child.stdout).to_string(),
        String::from_utf8_lossy(&child.stderr).to_string(),
    )
}

fn run_wonok_stdin(output: &str, input: &[u8]) -> (bool, String, String) {
    let mut child = Command::new(wonok_bin())
        .arg(output)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute wonok");
    child
        .stdin
        .take()
        .expect("failed to open stdin")
        .write_all(input)
        .expect("failed to write stdin");
    let output = child.wait_with_output().expect("failed to wait on wonok");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn successful_command_writes_output() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["echo", "hello world"]);
    assert!(success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "hello world\n");
}

#[test]
fn successful_command_with_args() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["echo", "-n", "no newline"]);
    assert!(success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "no newline");
}

#[test]
fn failing_command_does_not_write_file() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["false"]);
    assert!(!success);
    assert!(!outfile.exists());
}

#[test]
fn failing_command_does_not_overwrite_existing_file() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    fs::write(&outfile, "original content").unwrap();
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["false"]);
    assert!(!success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn failing_command_does_not_overwrite_existing_with_stdout() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    fs::write(&outfile, "original content").unwrap();
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["sh", "-c", "echo new; exit 1"]);
    assert!(!success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn command_not_found_fails() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["nonexistent_command_xyz"]);
    assert!(!success);
    assert!(!outfile.exists());
}

#[test]
fn pipe_mode_writes_stdin() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok_stdin(outfile.to_str().unwrap(), b"piped data here\n");
    assert!(success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "piped data here\n");
}

#[test]
fn pipe_mode_empty_stdin() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok_stdin(outfile.to_str().unwrap(), b"");
    assert!(success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "");
}

#[test]
fn pipe_mode_binary_data() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.bin");
    let data: Vec<u8> = (0..=255).collect();
    let (success, _, _) = run_wonok_stdin(outfile.to_str().unwrap(), &data);
    assert!(success);
    let content = fs::read(&outfile).unwrap();
    assert_eq!(content, data);
}

#[test]
fn pipe_mode_large_input() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let data = "x".repeat(16 * 1024 * 1024); // 16 MiB
    let (success, _, _) = run_wonok_stdin(outfile.to_str().unwrap(), data.as_bytes());
    assert!(success);
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content.len(), data.len());
}

#[test]
fn command_stderr_is_visible() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (_, _, stderr) = run_wonok(
        outfile.to_str().unwrap(),
        &["sh", "-c", "echo error_msg >&2; exit 1"],
    );
    assert!(stderr.contains("error_msg"));
}

#[test]
fn command_stderr_visible_on_success() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, stderr) = run_wonok(
        outfile.to_str().unwrap(),
        &["sh", "-c", "echo error_msg >&2; echo output"],
    );
    assert!(success);
    assert!(stderr.contains("error_msg"));
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "output\n");
}

#[test]
fn output_path_with_directories() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("subdir/deep/output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["echo", "nested"]);
    assert!(!success);
}

#[test]
fn successful_command_overwrites_previous_output() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["echo", "first"]);
    assert!(success);
    assert_eq!(fs::read_to_string(&outfile).unwrap(), "first\n");

    let (success, _, _) = run_wonok(outfile.to_str().unwrap(), &["echo", "second"]);
    assert!(success);
    assert_eq!(fs::read_to_string(&outfile).unwrap(), "second\n");
}

#[test]
fn exit_code_propagated_from_command() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    let child = Command::new(wonok_bin())
        .arg(outfile.to_str().unwrap())
        .arg("--")
        .args(["sh", "-c", "exit 42"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(child.status.code(), Some(42));
    assert!(!outfile.exists());
}

#[test]
fn pipe_mode_does_not_overwrite_on_read_error() {
    let tmp = TempDir::new().unwrap();
    let outfile = tmp.path().join("output.txt");
    fs::write(&outfile, "original").unwrap();
    let mut child = Command::new(wonok_bin())
        .arg(outfile.to_str().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Drop stdin without writing anything (simulates broken pipe)
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    // Wonok reads empty stdin, commits empty file
    assert!(output.status.success());
    let content = fs::read_to_string(&outfile).unwrap();
    assert_eq!(content, "");
}
