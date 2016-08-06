use super::bus::Bus;
use super::ncursesio::{Audio, Display, Input};
use super::processor::Processor;

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8<'a> {
    processor:Processor,
    bus:Bus<'a>
}

impl<'a> Chip8<'a> {
    pub fn new(audio:Audio, display:Display<'a>, input:Input<'a>) -> Chip8<'a> {
        Chip8{
            processor:Processor::default(),
            bus:Bus::new(audio, display, input),
        }
    }
}

impl<'a> Chip8<'a> {
    pub fn load_rom(&mut self, buff:&[u8]){
        self.bus.memory.set_range(0x200, buff)
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.processor.cycle(&mut self.bus);
        }
    }
}
