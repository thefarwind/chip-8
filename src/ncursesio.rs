extern crate ncurses;

use super::io;

#[derive(Default)]
pub struct Audio{}

impl io::Audio for Audio {
    fn beep(&self){
        ncurses::beep();
    }
}

enum Key {
    Key(u8),
    Und(i32),
}

pub struct Input<'a> {
    screen:&'a ncurses::SCREEN,
}

impl<'a> Input<'a> {
    fn map_key(key:i32) -> Key {
        match key {
            0x31 => Key::Key(0x0), // 1
            0x32 => Key::Key(0x1), // 2
            0x33 => Key::Key(0x2), // 3
            0x34 => Key::Key(0x3), // 4
            0x71 => Key::Key(0x4), // q
            0x77 => Key::Key(0x5), // w
            0x65 => Key::Key(0x6), // e
            0x72 => Key::Key(0x7), // r
            0x61 => Key::Key(0x8), // a
            0x73 => Key::Key(0x9), // s
            0x64 => Key::Key(0xA), // d
            0x66 => Key::Key(0xB), // f
            0x7A => Key::Key(0xC), // z
            0x78 => Key::Key(0xD), // x
            0x63 => Key::Key(0xE), // c
            0x76 => Key::Key(0xF), // v
            key => Key::Und(key),
        }
    }

    pub fn new(screen:&'a ncurses::SCREEN) -> Input<'a>{
        Input{screen:screen}
    }
}

impl<'a> io::Input for Input<'a> {
    fn get_keys(&self) -> Vec<u8> {
        let mut keys = Vec::<u8>::new();
        ncurses::nodelay(*self.screen, true);
        loop {
            match Input::map_key(ncurses::wgetch(*self.screen)) {
                Key::Key(key) => keys.push(key),
                Key::Und(ncurses::ERR) => break,
                _ =>{},
            }
        }
        ncurses::nodelay(*self.screen, false);
        keys
    }

    fn get_key(&self) -> u8 {
        loop {
            if let Key::Key(x) = Input::map_key(ncurses::wgetch(*self.screen)){
                return x;
            }
        }
    }
}

pub struct Display<'a> {
    screen:&'a ncurses::SCREEN,
}

impl<'a> Display<'a> {
    pub fn new(screen:&'a ncurses::SCREEN) -> Display<'a>{
        Display{screen:screen}
    }
}

impl<'a> io::Display for Display<'a> {
    fn set(&self, row:usize, col:usize, state:io::Pixel)
            -> Result<(),()> {
        let pixel:ncurses::chtype = match state {
            io::Pixel::On => 0x34,
            io::Pixel::Off => 0x20,
        };
        match ncurses::mvwaddch(*self.screen, row as i32, col as i32, pixel){
            ncurses::ERR => Err(()),
            _ => Ok(()),
        }
    }

    fn refresh(&self){
        ncurses::wrefresh(*self.screen);
    }
}
