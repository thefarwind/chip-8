extern crate ncurses;

mod bus;
mod chip8;
mod io;
mod memory;
mod ncursesio;
mod processor;
mod tests;

use std::fs::File;
use std::io::prelude::Read;

use io::Audio;

use chip8::Chip8;
use ncursesio::{Display, Input};

fn main() {

    // Open file
    let filename = std::env::args().nth(1).expect("missing file name");
    let mut file = match File::open(&filename){
        Ok(file) => file,
        Err(_) => panic!("failed to open file {}", filename),
    };

    // Extract bytes from file;
    let mut data = Vec::<u8>::new();
    match file.read_to_end(&mut data){
        Ok(_) => {},
        Err(_) => panic!("failed to read file"),
    };

    let data = data;
    /*
    for (i,byte) in data.iter().enumerate(){
        print!("{:02X}", byte);
        if i & 1 == 1 {print!("\n")}
    }*/


    ncurses::initscr();
    ncurses::noecho();
    ncurses::cbreak();

    let mut machine = Chip8::new(
        ncursesio::Audio::default(),
        Display::new(&ncurses::stdscr),
        Input::new(&ncurses::stdscr),
    );

    machine.load_rom(&data);
    machine.run();
}
