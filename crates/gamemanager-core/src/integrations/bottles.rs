use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::{CoreError, Result};

const FLATPAK_APP_ID: &str = "com.usebottles.bottles";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottlesCli {
    pub program: PathBuf,
    pub args_prefix: Vec<OsString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottlesCommandOutput {
    pub success: bool,
    pub stderr: String,
    pub stdout: String,
}

impl BottlesCommandOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            success: true,
            stderr: String::new(),
            stdout: stdout.into(),
        }
    }

    pub fn failure(stderr: impl Into<String>) -> Self {
        Self {
            success: false,
            stderr: stderr.into(),
            stdout: String::new(),
        }
    }
}

pub trait BottlesCommandRunner: Send + Sync {
    fn run(&self, cli: &BottlesCli, args: &[&str]) -> io::Result<BottlesCommandOutput>;
}

pub trait BottlesCliLocator: Send + Sync {
    fn locate(&self) -> Option<BottlesCli>;
}

#[derive(Default)]
pub struct SystemBottlesCliLocator;

impl BottlesCliLocator for SystemBottlesCliLocator {
    fn locate(&self) -> Option<BottlesCli> {
        if Command::new("flatpak")
            .args(["info", FLATPAK_APP_ID])
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return Some(BottlesCli::new("flatpak").with_prefix([
                "run",
                "--command=bottles-cli",
                FLATPAK_APP_ID,
            ]));
        }
        Command::new("bottles-cli")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
            .then(|| BottlesCli::new("bottles-cli"))
    }
}

struct ProcessBottlesCommandRunner;

impl BottlesCommandRunner for ProcessBottlesCommandRunner {
    fn run(&self, cli: &BottlesCli, args: &[&str]) -> io::Result<BottlesCommandOutput> {
        let output = Command::new(&cli.program)
            .args(&cli.args_prefix)
            .args(args)
            .output()?;
        Ok(BottlesCommandOutput {
            success: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }
}

impl BottlesCli {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args_prefix: Vec::new(),
        }
    }

    pub fn with_prefix<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args_prefix = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn list_bottles_with(&self, runner: &dyn BottlesCommandRunner) -> Result<Vec<String>> {
        let json = runner.run(self, &["--json", "list", "bottles"])?;
        if json.success {
            let bottles = parse_bottles_json(&json.stdout);
            if !bottles.is_empty() {
                return Ok(bottles);
            }
        }

        let text = runner.run(self, &["list", "bottles"])?;
        if !text.success {
            let detail = text.stderr.trim();
            return Err(CoreError::Configuration(if detail.is_empty() {
                "bottles-cli failed while listing bottles".to_owned()
            } else {
                format!("bottles-cli failed while listing bottles: {detail}")
            }));
        }
        Ok(parse_bottles_text(&text.stdout))
    }

    pub async fn list_bottles(&self) -> Result<Vec<String>> {
        let cli = self.clone();
        tokio::task::spawn_blocking(move || cli.list_bottles_with(&ProcessBottlesCommandRunner))
            .await
            .map_err(|error| {
                CoreError::Configuration(format!("bottles list task could not complete: {error}"))
            })?
    }

    pub fn plan_run(
        &self,
        bottle: &str,
        executable: &Path,
        args: &[String],
    ) -> (PathBuf, Vec<OsString>) {
        let mut command_args = self.args_prefix.clone();
        command_args.extend(["run", "-e"].into_iter().map(OsString::from));
        command_args.push(executable.as_os_str().to_owned());
        command_args.extend(["-b", bottle].into_iter().map(OsString::from));
        if !args.is_empty() {
            command_args.push(OsString::from("--"));
            command_args.extend(args.iter().map(OsString::from));
        }
        (self.program.clone(), command_args)
    }
}

fn parse_bottles_json(raw: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let values = value
        .as_array()
        .or_else(|| value.get("bottles").and_then(Value::as_array));
    values.map_or_else(Vec::new, |values| {
        deduplicate_bottle_names(values.iter().filter_map(extract_bottle_name))
    })
}

fn extract_bottle_name(value: &Value) -> Option<String> {
    if let Some(name) = value.as_str() {
        return normalized_bottle_name(name);
    }
    value.as_object().and_then(|object| {
        ["name", "Name", "bottle", "Bottle", "id", "Id"]
            .iter()
            .find_map(|key| object.get(*key).and_then(Value::as_str))
            .and_then(normalized_bottle_name)
    })
}

fn parse_bottles_text(raw: &str) -> Vec<String> {
    deduplicate_bottle_names(raw.lines().filter_map(|line| {
        let line = line.trim();
        line.strip_prefix('-')
            .or_else(|| line.strip_prefix('*'))
            .and_then(normalized_bottle_name)
    }))
}

fn normalized_bottle_name(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn deduplicate_bottle_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut bottles = Vec::new();
    for name in names {
        if !bottles.iter().any(|existing| existing == &name) {
            bottles.push(name);
        }
    }
    bottles
}
