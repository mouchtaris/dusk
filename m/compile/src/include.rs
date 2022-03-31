use {
    super::{facade, te, Compiler, Result},
    std::{fs, path::Path},
};

pub trait IncludeExt: AsMut<Compiler> {
    fn include<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let cmp = self.as_mut();
        let input = te!(fs::read_to_string(path));
        let block = te!(facade::parse_block(&input));
        te!(cmp.compile(block));
        Ok(())
    }
}

impl<C: AsMut<Compiler>> IncludeExt for C {}
