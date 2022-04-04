use {
    super::{facade, te, Compiler, FilePathExt, Result},
    std::fs,
};

pub trait IncludeExt: AsMut<Compiler> {
    fn include(&mut self, path: &str) -> Result<()> {
        let cmp = self.as_mut();

        let path = cmp.push_file_path(path);

        error::ldebug!("include :: {}", path);

        let input = te!(fs::read_to_string(path), "Include: {}", path);
        let block = te!(facade::parse_block(&input), "In include: {}", path);
        te!(cmp.compile(block));

        cmp.pop_file_path();

        Ok(())
    }
}

impl<C: AsMut<Compiler>> IncludeExt for C {}
