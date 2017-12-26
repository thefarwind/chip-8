extern crate termion;

use super::chip_8::io;


#[derive(Default)]
pub struct Audio{}

#[derive(Default)]
pub struct Display{}

#[derive(Default)]
pub struct Input{}

impl io::Audio for Audio {
    fn beep(&self){}
}

impl io::Display for Display {
    fn set(&mut self, row:usize, col:usize, state: io::Pixel) -> Result<(), ()> {
        Err(())
    }
    fn refresh(&mut self){}
}


impl io::Input for Input {
    fn get_keys(&self) -> Vec<u8> {
        Vec::new()
    }

    fn get_key(&self) -> u8 {
        0
    }
}
