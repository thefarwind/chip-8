extern crate ncurses;

#[derive(Default)]
pub struct Audio{}

impl Audio {
    pub fn beep(&self){
        ncurses::beep();
    }
}

enum Key {
    Key(u8),
    Und(i32),
}

#[derive(Default)]
pub struct Input {}

impl Input {
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

    pub fn get_keys(&self, screen:&ncurses::SCREEN) -> Vec<u8> {
        let mut keys = Vec::<u8>::new();
        ncurses::nodelay(*screen, true);
        loop {
            match Input::map_key(ncurses::wgetch(*screen)) {
                Key::Key(key) => keys.push(key),
                Key::Und(ncurses::ERR) => break,
                _ =>{},
            }
        }
        ncurses::nodelay(*screen, false);
        keys
    }

    pub fn get_key(&self, screen:&ncurses::SCREEN) -> u8 {
        loop {
            if let Key::Key(x) = Input::map_key(ncurses::wgetch(*screen)){
                return x;
            }
        }
    }
}

pub enum Pixel {
    On,
    Off,
}

#[derive(Default)]
pub struct Display {}

impl Display {
    pub fn set(&self, row:usize, col:usize, state:Pixel, screen:&ncurses::SCREEN)
            -> Result<(),()> {
        let pixel:ncurses::chtype = match state {
            Pixel::On => 0x34,
            Pixel::Off => 0x20,
        };
        match ncurses::mvwaddch(*screen, row as i32, col as i32, pixel){
            ncurses::ERR => Err(()),
            _ => Ok(()),
        }
    }
}
