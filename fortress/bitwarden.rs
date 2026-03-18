use anyhow::{Result, bail};
use log::{debug, trace};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub mod folder;
pub mod item;

#[allow(async_fn_in_trait)]
pub trait CoreCommands: Sized {
    type ListItem: Serialize + DeserializeOwned;

    async fn create(&self) -> Result<()>;
    fn edit(&self) -> Result<()>;
    async fn list(&self) -> Result<Vec<Self::ListItem>>;
    async fn delete(&self) -> Result<()>;
    async fn restore(&self) -> Result<()>;
    fn get(id: String) -> Result<Self>;
}

/// Runs a `bw` subcommand with optional JSON input and optional encoding.
/// When `input` is provided, it is piped into stdin.
/// When `encode` is true, the input is first piped through `bw encode`.
fn bw(args: Vec<&str>, input: Option<&str>, encode: bool) -> Result<String> {
    debug!("bw command: bw {}", args.join(" "));
    trace!("bw input: {:?}, encode: {}", input, encode);

    let piped_input = match (input, encode) {
        (Some(json), true) => Some(pipe_to_command("bw", vec!["encode"], json)?),
        (Some(json), false) => Some(json.to_string()),
        (None, _) => None,
    };

    let result = match piped_input {
        Some(ref data) => pipe_to_command("bw", args, data),
        None => {
            let output = Command::new("bw").args(args).output()?;
            check_output(output)
        }
    };

    match &result {
        Ok(output) => trace!("bw output: {}", output),
        Err(e) => debug!("bw error: {}", e),
    }

    result
}

fn pipe_to_command(cmd: &str, args: Vec<&str>, input: &str) -> Result<String> {
    trace!(
        "pipe_to_command: {} {} | stdin length: {}",
        cmd,
        args.join(" "),
        input.len()
    );

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    child.stdin.take().unwrap().write_all(input.as_bytes())?;

    let result = check_output(child.wait_with_output()?);

    match &result {
        Ok(output) => trace!("pipe_to_command output: {}", output),
        Err(e) => debug!("pipe_to_command error: {}", e),
    }

    result
}

fn check_output(output: std::process::Output) -> Result<String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        bail!(
            "bw command failed (exit code {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
