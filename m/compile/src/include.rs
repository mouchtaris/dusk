use super::{facade, fs, te, Compiler, FilePathExt, Mut, Result, SymInfo, SymbolTableExt};

pub trait IncludeExt: Mut<Compiler> {
    fn include(&mut self, path: &str) -> Result<()> {
        let cmp = self.borrow_mut();

        let input = te!((*cmp).include_file(path));
        let block = te!(facade::parse_block(&input), "In include: {}", path);

        let cmp_result = cmp.compile(block);

        let path = cmp.pop_file_path().unwrap();
        te!(cmp_result, "In including: {}", path);

        Ok(())
    }

    fn include_str(&mut self, ident: &str, path: &str) -> Result<SymInfo> {
        let cmp = self.borrow_mut();

        let input = te!((*cmp).include_file(path));

        let cmp_result = cmp.compile_text(input);

        let path = cmp.pop_file_path().unwrap();
        let sinfo = te!(cmp_result, "In emiting lit-string from: {}", path);
        cmp.alias_name(ident, &sinfo);

        Ok(sinfo)
    }
}

impl<C: Mut<Compiler>> IncludeExt for C {}

trait IncludePrivate: Mut<Compiler> {
    fn include_file(&mut self, path: &str) -> Result<String> {
        let cmp = self.borrow_mut();

        error::ltrace!("resolving include: {}", path);
        let path = (*cmp).push_file_path(path);

        error::ldebug!("include :: {}", path);

        let input = te!(fs::read_to_string(path), "Include: {}", path);
        Ok(input)
    }
}
impl<C: Mut<Compiler>> IncludePrivate for C {}
