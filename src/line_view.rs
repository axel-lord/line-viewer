use std::{fmt::Write, io, process::Command};

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
    pub title: Option<String>,
    pub lines: Vec<String>,
    pub action: Action,
}

impl LineView {
    pub fn parse(body: &str) -> Self {
        let mut this = Self::default();

        for line in body.lines() {
            if let Some(line) = line.strip_prefix('#') {
                if let Some(prefix) = line.strip_prefix("-pre ") {
                    this.action.prefix.push(prefix.into());
                } else if let Some(suffix) = line.strip_prefix("-suf ") {
                    this.action.suffix.push(suffix.into())
                } else if let Some(title) = line.strip_prefix("-title ") {
                    this.title = Some(String::from(title));
                }
            } else if !line.trim().is_empty() {
                this.lines.push(line.into());
            } else {
                this.lines.push(String::new());
            }
        }

        this
    }

    pub fn write(&self, mut writer: impl io::Write) -> io::Result<()> {
        let Self {
            lines,
            action: Action { prefix, suffix },
            title,
        } = self;

        if let Some(title) = title.as_ref() {
            writeln!(writer, "#-title {}", title.trim())?;
        }
        for pre in prefix {
            writeln!(writer, "#-pre {}", pre.trim())?;
        }
        for suf in suffix {
            writeln!(writer, "#-suf {}", suf.trim())?;
        }
        for line in lines {
            writeln!(writer, "{}", line.trim())?;
        }
        Ok(())
    }
}
