use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BottlesCli {
    pub program: PathBuf,
    pub args_prefix: Vec<OsString>,
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
