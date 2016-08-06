use super::memory::Memory;
use super::io::Audio;
use super::ncursesio::{Display, Input};

pub struct Bus<'a, A>
        where A: Audio {
    pub memory:Memory,
    pub audio:A,
    pub display:Display<'a>,
    pub input:Input<'a>,
}

impl<'a, A> Bus<'a, A>
        where A:Audio {
    pub fn new(audio:A, display:Display<'a>, input:Input<'a>) -> Bus<'a, A> {
        Bus{
            memory:Memory::default(),
            audio:audio,
            display:display,
            input:input,
        }
    }
}
