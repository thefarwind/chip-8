use super::bus::Bus;
use super::io::{Audio, Display};
use super::ncursesio::Input;
use super::processor::Processor;

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8<'a, A, D>
        where
            A: Audio,
            D: Display {
    processor:Processor,
    bus:Bus<'a, A, D>
}

impl<'a, A, D> Chip8<'a, A, D>
        where
            A: Audio,
            D: Display {
    pub fn new(audio:A, display:D, input:Input<'a>) -> Chip8<'a, A, D> {
        Chip8{
            processor:Processor::default(),
            bus:Bus::new(audio, display, input),
        }
    }
}

impl<'a, A, D> Chip8<'a, A, D>
        where
            A: Audio,
            D: Display {
    pub fn load_rom(&mut self, buff:&[u8]){
        self.bus.memory.set_range(0x200, buff)
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.processor.cycle(&mut self.bus);
        }
    }
}
