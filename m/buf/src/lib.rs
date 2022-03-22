pub const VERSION: &str = "0.0.1";

use std::ops::Range;

#[derive(Default, Debug)]
pub struct StringBuf {
    buf: String,
    seg: Vec<Range<usize>>,
}

impl StringBuf {
    pub fn new() -> Self {
        <_>::default()
    }
    pub fn add_str<S>(&mut self, s: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        let Self { buf, seg } = self;
        let s = s.as_ref();

        let a = buf.len();
        buf.push_str(s);
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

    pub fn seg_vec_in<'s>(&'s self, mut v: Vec<&'s str>) -> Vec<&'s str> {
        let mut i = 0;
        loop {
            match self.seg(i) {
                Some(s) => v.push(s),
                None => break,
            }
            i += 1;
        }
        v
    }
}
