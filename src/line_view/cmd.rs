use std::path::PathBuf;

use crate::{Error, LineSource, Result};

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    exe: Option<PathBuf>,
    arg: Vec<String>,
}

impl Cmd {
    pub fn exe(&mut self, exe: PathBuf) -> &mut Self {
        self.exe = Some(exe);
        self
    }

    pub fn arg(&mut self, arg: String) -> &mut Self {
        self.arg.push(arg);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.exe.is_none()
    }

    pub fn execute(
        &self,
        line_nr: usize,
        line_src: LineSource,
        params: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result {
        let Some(exe) = &self.exe else { return Ok(()) };

        let args = self
            .arg
            .iter()
            .map(String::from)
            .chain(params.into_iter().map(|param| param.into()))
            .collect::<Vec<String>>();

        std::process::Command::new(exe)
            .env("LINE_VIEW_LINE_NR", line_nr.to_string())
            .env("LINE_VIEW_LINE_SRC", line_src.to_string())
            .args(&args)
            .spawn()
            .map_err(|err| Error::Spawn {
                err,
                program: exe.display().to_string(),
                args,
            })?;
        Ok(())
    }
}
