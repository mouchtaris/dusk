use super::StringBuf;

pub struct StringBufIterator<'s> {
    pub(super) source: &'s StringBuf,
    pub(super) counter: usize,
}

impl<'s> Iterator for StringBufIterator<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<&'s str> {
        let Self { source, counter } = self;
        let c = *counter;
        *counter += 1;
        source.seg(c)
    }
}

impl<'s> IntoIterator for &'s StringBuf {
    type Item = &'s str;
    type IntoIter = StringBufIterator<'s>;
    fn into_iter(self) -> Self::IntoIter {
        StringBufIterator {
            source: self,
            counter: 0,
        }
    }
}
