use std::io::{
    Write,
    stdout,
};

use termion::raw::IntoRawMode;

use super::chip_8::io;

#[derive(Default)]
pub struct Audio;

pub struct Display {
    buffer: String,
}

impl Default for Display {
    fn default() -> Self {
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
        Display { buffer: String::default() }
    }
}

impl Drop for Display {
    fn drop(&mut self){
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        write!(stdout, "{}{}{}",
            termion::clear::All,
            termion::style::Reset,
            termion::cursor::Goto(1, 1)
        ).unwrap();
    }
}

#[derive(Default)]
pub struct Input;

impl io::Audio for Audio {
    fn beep(&self){}
}


impl io::Display for Display {
    fn set(&mut self, row:usize, col:usize, state: io::Pixel) -> Result<(), ()> {
        let goto = termion::cursor::Goto( col as u16 + 1, row as u16 + 1);
        let item = if let io::Pixel::On = state {
            format!("{} ", goto)
        } else {
            format!("{}{} {}", termion::style::Invert, goto, termion::style::NoInvert)
        };
        self.buffer.push_str(&item);
        Ok(())
    }
    fn refresh(&mut self){
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        write!(stdout, "{}", self.buffer).unwrap();
        self.buffer.clear();
    }
}


impl io::Input for Input {
    fn get_keys(&self) -> Vec<u8> {
        Vec::new()
    }

    fn get_key(&self) -> u8 {
        0
    }
}
