use super::memory::Memory;
use super::io::{Audio, Display, Input};

pub struct Bus<A, D, I>
        where
            A: Audio,
            D: Display,
            I: Input {
    pub memory:Memory,
    pub audio:A,
    pub display:D,
    pub input:I,
}

impl<A, D, I> Bus<A, D, I>
        where
            A: Audio,
            D: Display,
            I: Input {
    pub fn new(audio:A, display:D, input:I) -> Bus<A, D, I> {
        Bus{
            memory:Memory::default(),
            audio:audio,
            display:display,
            input:input,
        }
    }
}
