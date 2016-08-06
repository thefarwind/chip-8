use super::memory::Memory;
use super::ncursesio::{Audio, Display, Input};

pub struct Bus<'a> {
    pub memory:Memory,
    pub audio:Audio,
    pub display:Display<'a>,
    pub input:Input<'a>,
}

impl<'a> Bus<'a> {
    pub fn new(audio:Audio, display:Display<'a>, input:Input<'a>) -> Bus<'a> {
        Bus{
            memory:Memory::default(),
            audio:audio,
            display:display,
            input:input,
        }
    }
}
