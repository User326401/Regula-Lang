#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_pos: usize,
    pub end_pos: usize,
}

impl Span {
    #[inline(always)]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start_pos: self.start_pos.min(other.start_pos),
            end_pos: self.end_pos.max(other.end_pos),
        }
    }

    #[inline(always)]
    pub fn new(start_pos: usize, end_pos: usize) -> Self {
        Self { start_pos, end_pos }
    }
}
