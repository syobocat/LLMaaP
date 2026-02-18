use std::{
    io::Read,
    process::{Command, Stdio},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use wait_timeout::ChildExt;

#[derive(Serialize)]
pub struct Output {
    timeout: bool,
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
}

#[derive(Deserialize)]
pub struct Exec {
    command: String,
    capture_output: bool,
}

impl Exec {
    pub fn run(&self) -> Output {
        let mut process = Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let timeout = Duration::from_secs(60);
        let mut is_timeout = false;
        let status = process.wait_timeout(timeout).unwrap().map_or_else(
            || {
                is_timeout = true;
                process.kill().unwrap();
                process.wait().unwrap().code()
            },
            |status| status.code(),
        );

        let (stdout, stderr) = if self.capture_output {
            let mut stdout = String::new();
            let mut stderr = String::new();
            let _ = process.stdout.unwrap().read_to_string(&mut stdout);
            let _ = process.stderr.unwrap().read_to_string(&mut stderr);
            (Some(stdout), Some(stderr))
        } else {
            (None, None)
        };

        Output {
            timeout: is_timeout,
            exit_code: status,
            stdout,
            stderr,
        }
    }
}
