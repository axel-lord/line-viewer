use crate::{Error, Result, LineSource};

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    pre: Vec<String>,
    suf: Vec<String>,
}

impl Cmd {
    pub fn pre(&mut self, arg: impl Into<String>) -> &mut Self {
        self.pre.push(arg.into());
        self
    }

    pub fn suf(&mut self, arg: impl Into<String>) -> &mut Self {
        self.suf.push(arg.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.suf.is_empty() && self.pre.is_empty()
    }

    pub fn execute(
        &self,
        line_nr: usize,
        line_src: LineSource,
        params: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result {
        let mut args = self
            .pre
            .iter()
            .cloned()
            .chain(params.into_iter().map(Into::into))
            .chain(self.suf.iter().cloned());

        let Some(program) = args.next() else {
            return Ok(());
        };

        let args = args.collect::<Vec<_>>();

        std::process::Command::new(&program)
            .env("LINE_VIEW_LINE_NR", line_nr.to_string())
            .env("LINE_VIEW_LINE_SRC", line_src.to_string())
            .args(&args)
            .spawn()
            .map_err(|err| Error::Spawn { err, program, args })?;
        Ok(())
    }
}
