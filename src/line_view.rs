use std::{fmt::Write, process::Command};

#[derive(Default, Clone, Debug)]
pub struct Action {
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
}

impl Action {
    pub fn spawn(&self, line: String) -> String {
        let Action { prefix, suffix } = self;

        let iter = prefix
            .iter()
            .skip(1)
            .chain(Some(&line))
            .chain(suffix.iter());

        _ = iter
            .clone()
            .fold(
                &mut Command::new(prefix.first().map(String::as_str).unwrap_or("echo")),
                |cmd, arg| cmd.arg(arg),
            )
            .spawn();

        prefix
            .first()
            .into_iter()
            .chain(iter)
            .fold(String::new(), |mut cmd, component| {
                _ = write!(&mut cmd, " {component}");
                cmd
            })
            .trim()
            .into()
    }
}

#[derive(Clone, Default, Debug)]
pub struct LineView {
    pub lines: Vec<String>,
    pub action: Action,
}

impl LineView {
    pub fn parse(body: &str) -> Self {
        let mut action = Action::default();
        let mut lines = Vec::new();

        for line in body.lines() {
            if let Some(line) = line.strip_prefix('#') {
                if let Some(prefix) = line.strip_prefix("-pre ") {
                    action.prefix.push(prefix.into());
                } else if let Some(suffix) = line.strip_prefix("-suf ") {
                    action.suffix.push(suffix.into())
                }
            } else if !line.trim().is_empty() {
                lines.push(line.into());
            } else {
                lines.push(String::new());
            }
        }

        Self { lines, action }
    }
}
