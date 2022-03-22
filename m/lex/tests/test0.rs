use {
    lex::{self as x, longopt, shortopt},
    lexpop::{self as l, Prop},
};

macro_rules! test {
    ($prop:expr, $inp:expr, $exp:expr) => {{
        let mut m = $prop;
        let v = m.match_str::<4, _>($inp);
        assert_eq!(v, $exp);
    }};
}

#[test]
fn test_longopt() {
    test!(longopt(), "-", 0);
    test!(longopt(), "--", 2);
    test!(longopt(), "---", 2);
    test!(longopt(), "--a", 3);

    test!(l::exact("--"), "--", 2);
    test!(l::exact("--"), "-", 0);
    test!(x::ident(), "-", 0);
    let m = || l::one_and_any(l::exact("--"), x::ident);
    test!(m(), "-", 0);
    test!(m(), "--", 2);
    test!(m(), "-a", 0);
    test!(longopt(), "-a", 0);
}

#[test]
fn test_shortopt() {
    test!(shortopt(), "-", 1);
    test!(shortopt(), "--", 1);
    test!(shortopt(), "-a", 2);
}
