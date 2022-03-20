#[cfg(test)]
use lexpop::{either, exact, fn_, fnr, one_and_any, one_of, Prop};

#[test]
fn test_either() {
    const N: usize = 1;

    let a = || exact("the");
    let b = || exact("apple");
    let m = || either(b(), a());
    let m = |s| m().match_str::<N, _>(s);

    assert_eq!(m(""), 0);
    assert_eq!(m("t"), 0);
    assert_eq!(m("th"), 0);
    assert_eq!(m("the"), 3);
    assert_eq!(m("a"), 0);
    assert_eq!(m("ap"), 0);
    assert_eq!(m("app"), 0);
    assert_eq!(m("appl"), 0);
    assert_eq!(m("apple"), 5);
    assert_eq!(m("theextra"), 3);
    assert_eq!(m("theapple"), 3);
    assert_eq!(m("applethe"), 5);
}

#[test]
fn test_one_and_any() {
    const N: usize = 1;

    let a = || fn_(char::is_whitespace);
    let b = || fnr(char::is_ascii_punctuation);
    let m = || one_and_any(b(), a());
    let m = |s| m().match_str::<N, _>(s);

    assert_eq!(m(""), 0);
    assert_eq!(m("."), 1);
    assert_eq!(m("/"), 1);
    assert_eq!(m("//"), 1);
    assert_eq!(m("/ "), 2);
    assert_eq!(m("/  "), 3);
}

#[test]
fn test_one_of() {
    const N: usize = 1;

    let m = |e: &str, s| one_of(e.chars()).match_str::<N, _>(s);

    assert_eq!(m("", ""), 0);
    assert_eq!(m("%#$@", ""), 0);
    assert_eq!(m("%#$@", "%"), 1);
    assert_eq!(m("%#$@", "#"), 1);
    assert_eq!(m("%#$@", "$"), 1);
    assert_eq!(m("%#$@", "@"), 1);
    assert_eq!(m("%#$@", "@$"), 1);
    assert_eq!(m("%#$@", "#@$%"), 1);
    assert_eq!(m("%#$@", "#@$"), 1);
    assert_eq!(m("%#$@", " #@$"), 0);
}

#[test]
fn test_exact() {
    const N: usize = 1;

    let m = |e, s| exact(e).match_str::<N, _>(s);

    assert_eq!(m("", ""), 0);

    assert_eq!(m("a", ""), 0);
    assert_eq!(m("ab", ""), 0);

    assert_eq!(m("", "a"), 0);
    assert_eq!(m("", "ab"), 0);

    assert_eq!(m("a", "a"), 1);

    assert_eq!(m("a", "ab"), 1);
    assert_eq!(m("b", "ab"), 0);

    assert_eq!(m("ab", "aba"), 2);
    assert_eq!(m("ba", "aba"), 0);

    assert_eq!(m("baab", "baab"), 4);
    assert_eq!(m("baab", "baabi"), 4);
    assert_eq!(m("baab", "baa"), 0);
}

#[test]
fn test_from_fn() {
    const N: usize = 1;

    let mut a = fnr(char::is_ascii_digit);
    let mut m = |s| a.match_str::<N, _>(s);
    assert_eq!(m(""), 0);
    assert_eq!(m("."), 0);
    assert_eq!(m("1"), 1);
    assert_eq!(m("1."), 1);
    assert_eq!(m("11"), 2);
    assert_eq!(m("11."), 2);
    assert_eq!(m("11./"), 2);
    assert_eq!(m("11./22"), 2);

    let mut b = fn_(|c| c.is_ascii_punctuation());
    let mut m = |s| b.match_str::<N, _>(s);
    assert_eq!(m(""), 0);
    assert_eq!(m("."), 1);
    assert_eq!(m("/."), 2);
    assert_eq!(m("/.ab"), 2);
    assert_eq!(m("ab/."), 0);
}
