pub type MatchName = fn(&str, &str) -> bool;

fn last_of_path() -> MatchName {
    |name, var| {
        if var.len() >= name.len() + 1 {
            // +1 because '.../<name>'
            let (_, suffix) = var.split_at(var.len() - name.len());

            if suffix.starts_with('/') && suffix.ends_with(name) {
                return true;
            }
        }

        false
    }
}
