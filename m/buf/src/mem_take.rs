pub trait MemTake: Default {
    fn mem_take(&mut self) -> Self;
}

impl<S> MemTake for S
where
    S: Default,
{
    fn mem_take(&mut self) -> Self {
        std::mem::take(self)
    }
}
