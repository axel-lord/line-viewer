use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct Cmd {
    pre: Vec<String>,
    suf: Vec<String>,
}

impl Cmd {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn execute(&self, params: impl IntoIterator<Item = impl Into<String>>) -> Result {
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
            .args(&args)
            .spawn()
            .map_err(|err| Error::Spawn { err, program, args })?;
        Ok(())
    }
}
