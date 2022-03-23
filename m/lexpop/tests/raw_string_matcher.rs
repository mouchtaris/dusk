use lexpop::{lex::fat, Prop};

macro_rules! t {
    ($mat:expr, $inp:expr, $exp:expr) => {
        let mut m = $mat;
        let s = $inp;
        let e = $exp;
        assert_eq!(m.match_str::<1, _>(s), e);
    };
    ($inp:expr, $exp:expr) => {
        t!(fat(), $inp, $exp);
    };
    ($inp:expr) => {
        let s = $inp;
        t!(s, s.len());
    };
}

#[test]
fn complex_prop0() {
    t!("r", 0);
}
#[test]
fn complex_prop1() {
    t!("r#", 0);
}
#[test]
fn complex_prop2() {
    t!("r#\"", 0);
}
#[test]
fn complex_prop3() {
    t!("r#\"\"", 0);
}
#[test]
fn complex_prop4() {
    t!("r##\"\"#", 0);
}
#[test]
fn complex_prop5() {
    t!("r##\"\"##", 7);
}
#[test]
fn complex_prop6() {
    t!("r#\"What is happening\"#");
}
#[test]
fn complex_prop7() {
    t!(r###"r##"a"b"#c"##"###);
}
#[test]
fn complex_prop8() {
    t!(r####"r"""####);
}
#[test]
fn complex_prop9() {
    t!(r####"r#"""#"####);
}
#[test]
fn complex_prop10() {
    t!(r####"r##""#"##"####);
}
#[test]
fn complex_prop11() {
    t!(r####"r###"""#"##"###"####);
}
#[test]
fn complex_prop12() {
    t!(r####"r#"Kitty"#"####);
}
#[test]
fn complex_prop13() {
    t!(r####"r#"K"#  "####, r##"r#"K"#"##.len());
}
#[test]
fn complex_prop14() {
    t!(r####"r"K""####);
}
#[test]
fn complex_prop15() {
    t!(r####"""####, 0);
}
#[test]
fn complex_prop16() {
    t!(r####""""####);
}
#[test]
fn complex_prop17() {
    t!(r####""#""####, "\"#\"".len());
}
