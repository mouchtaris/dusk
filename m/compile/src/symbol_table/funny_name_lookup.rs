use super::*;

pub fn accept_name_as_local_alias(name: &str, var: &str) -> bool {
    if var.len() >= name.len() + 1 {
        // +1 because '.../<name>'
        let (_, suffix) = var.split_at(var.len() - name.len());

        if suffix.starts_with('/') && suffix.ends_with(name) {
            return true;
        }
    }

    false
}

pub fn accept_symbol<'a>(
    name: &'a str,
) -> impl 'a + for<'r> Fn((&str, &'r SymID)) -> Option<&'r SymID> {
    move |(var, sym_id)| {
        if accept_name_as_local_alias(name, var) {
            Some(sym_id)
        } else {
            None
        }
    }
}

pub fn accept<'a>(name: &'a str) -> impl 'a + Fn(&Scope) -> Result<&SymID> {
    |scope| {
        if let Some(x) = scope.symbols().find_map(accept_symbol(name)) {
            return Ok(x);
        }
        temg!("not found")
    }
}

pub fn lookup<'a, S: 'a + ?Sized + ScopesRef>(this: &'a S, name: &str) -> Result<&'a SymInfo> {
    todo!()
}
