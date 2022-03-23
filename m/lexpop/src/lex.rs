super::lexpop![fat, {
    let mut h = 0;
    let mut state = 0;
    let mut j = 0;
    let mut nr = 0;
    move |r| {
        let a = r[0];
        let ret = match (state, a) {
            // 0: Init
            (0, 'r') => {
                h = 0;
                nr = 1;
                state = 1;
                Some(0)
            }
            (0, '"') => {
                h = 0;
                nr = 0;
                state = 2;
                Some(0)
            }
            // 1: Reading hashes
            (1, '#') => {
                h += 1;
                Some(0)
            }
            (1, '"') => {
                state = 2;
                j = h;
                Some(0)
            }
            // 2: Reading content
            (2, '"') if h == 0 => {
                // Completely done, no hashes to read
                state = 4;

                // The extra commit is only r" and "
                Some(nr + 1 + 1)
            }
            (2, '"') => {
                state = 3;
                Some(0)
            }
            (2, _) => Some(1),
            // 3: Reading closing hashes
            (3, '"') => {
                // Count the previous " as content
                // and keep trying to close
                let n = h - j;
                j = h;
                Some(1 + n)
            }
            (3, '#') if j > 1 => {
                // One less closing hash
                j -= 1;
                Some(0)
            }
            (3, '#') if j == 1 => {
                // Totally closed
                state = 4;
                // Commit:
                //  1 r
                //  h opening #
                //  1 opening "
                //  h closing #
                //  1 closing "
                Some(1 + h + 1 + 1 + h)
            }
            (3, _) => {
                // Not closing yet

                // Commit skipped stuff
                // 1        false closing "
                // (h - j)  false closing #
                // 1        non-special character consumed just now
                let n = 1 + (h - j) + 1;

                // Back to reading content
                state = 2;
                j = h;

                Some(n)
            }
            (4, _) => None,
            _ => None,
        };
        ret
    }
}];
