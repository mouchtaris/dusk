pub const VERSION: &str = "0.0.1";

pub fn fnr<F>(mut f: F) -> impl FnMut(Range) -> Option<usize>
where
    F: FnMut(&char) -> bool,
{
    fn_(move |c| f(&c))
}

pub fn fn_<F>(mut f: F) -> impl FnMut(Range) -> Option<usize>
where
    F: FnMut(char) -> bool,
{
    move |r: Range| match r {
        &[c, ..] if f(c) => Some(1),
        _ => None,
    }
}

//  -> impl FnMut(Range) -> Option<usize> + '_ {

pub fn either<A: Prop, B: Prop>(mut a: A, mut b: B) -> impl FnMut(Range) -> Option<usize> {
    let mut pos_a = Some(0);
    let mut pos_b = Some(0);
    let mut pos = 0;

    move |r| {
        use std::cmp::max;

        let mut c = pos;

        if let Some(pa) = pos_a.as_mut() {
            match a.prop(r) {
                Some(n) => {
                    *pa += n;
                    c = max(pos, *pa);
                }
                None => pos_a = None,
            }
        }
        if let Some(pb) = pos_b.as_mut() {
            match b.prop(r) {
                Some(n) => {
                    *pb += n;
                    c = max(pos, *pb);
                }
                None => pos_b = None,
            }
        }

        let n = c - pos;
        pos = c;

        if pos_a.is_some() || pos_b.is_some() {
            Some(n)
        } else {
            None
        }
    }
}

pub fn one_of<I>(set: I) -> impl FnMut(Range) -> Option<usize>
where
    I: IntoIterator,
    I::Item: Into<char>,
{
    let mut set = set.into_iter().map(<_>::into);
    let mut done = false;
    move |r| {
        if done {
            None
        } else {
            done = true;
            let c = &r[0];
            set.find(|x| x == c).map(|_| 1)
        }
    }
}

pub fn one_and_any<A: Prop, B: Prop>(mut a: A, mut b: B) -> impl FnMut(Range) -> Option<usize> {
    let mut s = 0;
    move |r| match s {
        0 => {
            s = 1;
            a.prop(r)
        }
        _ => b.prop(r),
    }
}

pub fn exact(s: &str) -> impl FnMut(Range) -> Option<usize> + '_ {
    let mut chars = s.chars();
    let mut n = 0;
    move |r: Range| match (chars.next(), r) {
        (Some(x), &[c, ..]) if c == x => {
            n += 1;
            let r = match n {
                n if n == s.len() => n,
                _ => 0,
            };
            Some(r)
        }
        _ => None,
    }
}

pub trait Prop {
    fn prop(&mut self, range: &[char]) -> Option<usize>;

    fn match_str<const N: usize, S>(&mut self, inp: S) -> Span
    where
        S: AsRef<str>,
    {
        self.match_range::<N, _>(inp.as_ref().chars())
    }

    fn match_range<const N: usize, I>(&mut self, inp: I) -> Span
    where
        I: IntoIterator,
        I::Item: Into<char>,
    {
        let mut buf = Buffer::<N, char>::new();
        let mut inp = inp.into_iter().map(<_>::into);

        let mut span = 0;
        loop {
            if !buf.read_one(&mut inp) {
                break;
            }
            if let Some(n) = self.prop(&buf) {
                span += n;
            } else {
                break;
            }
        }

        span
    }
}
impl<F> Prop for F
where
    F: FnMut(Range) -> Option<usize>,
{
    fn prop(&mut self, range: Range) -> Option<usize> {
        self(range)
    }
}

impl<const N: usize, T> Buffer<N, T>
where
    T: Default + Copy,
{
    pub fn read_one<I>(&mut self, inp: &mut I) -> bool
    where
        I: Iterator,
        I::Item: Into<T>,
        T: Default,
    {
        let Self { range } = self;

        match inp.map(<_>::into).next() {
            Some(c) => {
                for n in 0..N - 1 {
                    let n = (N - 1) - n;
                    range[n] = range[n - 1];
                }
                range[0] = c;
                true
            }
            None => false,
        }
    }

    pub fn new() -> Self {
        Self {
            range: [<_>::default(); N],
        }
    }
}
impl<const N: usize, T> std::ops::Deref for Buffer<N, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.range
    }
}

pub type Range<'a> = &'a [char];
pub type Span = usize;

pub trait CharDeco: Into<char> {
    fn d10(self) -> bool {
        self.into().is_digit(10)
    }
}
impl CharDeco for char {}

#[derive(Debug)]
pub struct Buffer<const N: usize, T> {
    pub range: [T; N],
}

#[macro_export]
macro_rules! lexpop {
    (
    $(
    $name:ident, $body:expr
    ),*
    ) => {
    $(
        pub fn $name() -> impl $crate::Prop {
            fn f<F: FnMut($crate::Range) -> Option<usize>>(f: F) -> F { f }
            f($body)
        }
    )*
    };
}

#[cfg(test)]
mod __ {
    use super::{CharDeco, Prop};
    lexpop![fat, {
        let mut h = 0;
        let mut state = 0;
        let mut j = 0;
        move |r| {
            let ret = match (state, r) {
                // 0: Init
                (0, &['r', ..]) => {
                    h = 0;
                    state = 1;
                    Some(0)
                }
                // 1: Reading hashes
                (1, &['#', ..]) => {
                    h += 1;
                    Some(0)
                }
                (1, &['"', ..]) => {
                    state = 2;
                    j = h;
                    Some(0)
                }
                // 2: Reading content
                (2, &['"', ..]) if h == 0 => {
                    // Completely done, no hashes to read

                    // The extra commit is only r" and "
                    Some(2 + 1)
                }
                (2, &['"', ..]) => {
                    state = 3;
                    Some(0)
                }
                (2, _) => Some(1),
                // 3: Reading closing hashes
                (3, &['#', ..]) if j > 1 => {
                    // One less closing hash
                    j -= 1;
                    Some(0)
                }
                (3, &['#', ..]) if j == 1 => {
                    // Totally closed
                    // Commit:
                    //  1 r
                    //  h opening #
                    //  1 opening "
                    //  h closing #
                    //  1 closing "
                    Some(1 + h + 1 + 1 + h)
                }
                (3, &['"', ..]) => {
                    // Not closing yet

                    // Commit skipped stuff
                    // 1        false closing "
                    // (h - j)  false closing #
                    let n = 1 + (h - j);

                    // Commit this as content and continue trying to close
                    j = h;
                    Some(n)
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
                other => panic!("{}:{} {:?}", h, j, other),
            };
            ret
        }
    }];
    #[test]
    fn complex_prop0() {
        let wat = fat().match_range::<6, _>("r".chars());
        assert_eq!(wat, 0);
    }
    #[test]
    fn complex_prop1() {
        let wat = fat().match_range::<6, _>("r#".chars());
        assert_eq!(wat, 0);
    }
    #[test]
    fn complex_prop2() {
        let wat = fat().match_range::<6, _>("r#\"".chars());
        assert_eq!(wat, 0);
    }
    #[test]
    fn complex_prop3() {
        let wat = fat().match_range::<6, _>("r#\"\"".chars());
        assert_eq!(wat, 0);
    }
    #[test]
    fn complex_prop4() {
        let wat = fat().match_range::<6, _>("r##\"\"#".chars());
        assert_eq!(wat, 0);
    }
    #[test]
    fn complex_prop5() {
        let wat = fat().match_range::<6, _>("r##\"\"##".chars());
        assert_eq!(wat, 7);
    }
    #[test]
    fn complex_prop6() {
        let s = || "r#\"What is happening\"#".chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }
    #[test]
    fn complex_prop7() {
        let s = || r###"r##"a"b"#c"##"###.chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }
    #[test]
    fn complex_prop8() {
        let s = || r####"r"""####.chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }
    #[test]
    fn complex_prop9() {
        let s = || r####"r#"""#"####.chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }
    #[test]
    fn complex_prop10() {
        let s = || r####"r##""#"##"####.chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }
    #[test]
    fn complex_prop11() {
        let s = || r####"r###"""#"##"###"####.chars();
        let wat = fat().match_range::<6, _>(s());
        assert_eq!(wat, s().count());
    }

    lexpop![nat, |r| match r {
        &[p, ..] if p.d10() => Some(1),
        _ => None,
    }];
    #[test]
    fn match_range() {
        let wat = nat().match_range::<4, _>("Hello".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("1".chars());
        assert_eq!(wat, 1);

        let wat = nat().match_range::<4, _>("12".chars());
        assert_eq!(wat, 2);

        let wat = nat().match_range::<4, _>("123".chars());
        assert_eq!(wat, 3);

        let wat = nat().match_range::<4, _>("1234".chars());
        assert_eq!(wat, 4);

        let wat = nat().match_range::<4, _>("12345".chars());
        assert_eq!(wat, 5);

        let wat = nat().match_range::<4, _>("a12345".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("abcd+12345".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("+12345".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("-12345".chars());
        assert_eq!(wat, 0);

        let wat = nat().match_range::<4, _>("abcd-12345".chars());
        assert_eq!(wat, 0);
    }
}

error::Error! {
    Msg = String
}
