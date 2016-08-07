use super::memory::Memory;
use super::io::{Audio, Display};
use super::ncursesio::Input;

pub struct Bus<'a, A, D>
        where
            A: Audio,
            D: Display {
    pub memory:Memory,
    pub audio:A,
    pub display:D,
    pub input:Input<'a>,
}

impl<'a, A, D> Bus<'a, A, D>
        where
            A: Audio,
            D: Display {
    pub fn new(audio:A, display:D, input:Input<'a>) -> Bus<'a, A, D> {
        Bus{
            memory:Memory::default(),
            audio:audio,
            display:display,
            input:input,
        }
    }
}
