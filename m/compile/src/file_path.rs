use super::Compiler;

pub trait FilePathExt: AsMut<Compiler> {
    fn push_file_path(&mut self, path: &str) -> &str {
        let paths = &mut self.cmp().current_file_path;
        let mut base = paths.last().cloned().unwrap_or_default();
        compute_include_path(&mut base, path);
        paths.push(base);
        paths.last().unwrap().as_str()
    }

    fn pop_file_path(&mut self) {
        self.cmp().current_file_path.pop();
    }

    fn cmp(&mut self) -> &mut Compiler {
        self.as_mut()
    }
}

impl<S: AsMut<Compiler>> FilePathExt for S {}

fn compute_include_path(base: &mut String, path: &str) {
    let (last_slash, _) = base
        .char_indices()
        .rev()
        .find(|&(_, x)| x == '/')
        .unwrap_or((0, '/'));
    base.truncate(last_slash + 1);
    base.push_str(path);
}
