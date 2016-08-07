use super::bus::Bus;
use super::io::{Audio, Display, Input};
use super::processor::Processor;

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8<A, D, I>
        where
            A: Audio,
            D: Display,
            I: Input {
    processor:Processor,
    bus:Bus<A, D, I>
}

impl<A, D, I> Chip8<A, D, I>
        where
            A: Audio,
            D: Display,
            I: Input {
    pub fn new(audio:A, display:D, input:I) -> Chip8<A, D, I> {
        Chip8{
            processor:Processor::default(),
            bus:Bus::new(audio, display, input),
        }
    }
}

impl<A, D, I> Chip8<A, D, I>
        where
            A: Audio,
            D: Display,
            I: Input {
    pub fn load_rom(&mut self, buff:&[u8]){
        self.bus.memory.set_range(0x200, buff)
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.processor.cycle(&mut self.bus);
        }
    }
}
