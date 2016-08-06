use super::bus::Bus;
use super::io::Audio;
use super::ncursesio::{Display, Input};
use super::processor::Processor;

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8<'a, A>
        where A:Audio {
    processor:Processor,
    bus:Bus<'a, A>
}

impl<'a, A> Chip8<'a, A>
        where A:Audio {
    pub fn new(audio:A, display:Display<'a>, input:Input<'a>) -> Chip8<'a, A> {
        Chip8{
            processor:Processor::default(),
            bus:Bus::new(audio, display, input),
        }
    }
}

impl<'a, A> Chip8<'a, A>
        where A:Audio {
    pub fn load_rom(&mut self, buff:&[u8]){
        self.bus.memory.set_range(0x200, buff)
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.processor.cycle(&mut self.bus);
        }
    }
}
