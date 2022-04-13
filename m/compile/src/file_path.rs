use super::Compiler;

pub trait FilePathExt: AsMut<Compiler> {
    fn push_file_path(&mut self, path: &str) -> &str {
        let paths = &mut self.cmp().current_file_path;
        let mut base = paths.last().cloned().unwrap_or_default();
        compute_include_path(&mut base, path);
        paths.push(base);
        paths.last().unwrap().as_str()
    }

    fn pop_file_path(&mut self) -> Option<String> {
        self.cmp().current_file_path.pop()
    }

    fn cmp(&mut self) -> &mut Compiler {
        self.as_mut()
    }
}

impl<S: AsMut<Compiler>> FilePathExt for S {}

pub fn compute_include_path(base: &mut String, path: &str) {
    error::ltrace!("resolve path {} + {}", base, path);

    if path.starts_with('/') {
        base.clear();
        base.push_str(path);
    } else {
        let last_slash = base
            .char_indices()
            .rev()
            .find(|&(_, x)| x == '/')
            .map(|(l, _)| l);

        match last_slash {
            Some(n) => {
                base.truncate(n + 1);
            }
            None => {
                base.clear();
                base.push_str("./");
            }
        }

        let path = if path.starts_with("./") {
            &path[2..]
        } else {
            path
        };

        base.push_str(path);
    }
}
