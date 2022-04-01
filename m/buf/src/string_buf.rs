use std::{
    fmt::{Display, Write},
    ops::Range,
};

#[derive(Default, Debug)]
pub struct StringBuf {
    buf: String,
    seg: Vec<Range<usize>>,
}

impl StringBuf {
    pub fn new() -> Self {
        <_>::default()
    }
    pub fn add<S>(&mut self, s: S) -> &mut Self
    where
        S: Display,
    {
        let Self { buf, seg } = self;

        let a = buf.len();
        write!(buf, "{}", s).unwrap();
        let b = buf.len();

        seg.push(a..b);

        self
    }

    pub fn len(&self) -> usize {
        self.seg.len()
    }

    pub fn seg(&self, i: usize) -> Option<&str> {
        let Self { buf, seg } = self;
        seg.get(i).map(|r| &buf[r.clone()])
    }
}
