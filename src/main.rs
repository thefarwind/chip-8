extern crate ncurses;

mod chip8;
mod memory;
mod ncursesio;

use std::fs::File;
use std::io::prelude::Read;

use chip8::Chip8;
use ncursesio::{Audio, Display, Input};


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
        Audio::default(),
        Display::new(&ncurses::stdscr),
        Input::new(&ncurses::stdscr),
    );
    machine.load_rom(&data);
    loop {
        machine.cycle();
    }

    ncurses::nocbreak();
    ncurses::echo();
    ncurses::endwin();
}


// TESTS
#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use rand::Rng;

    #[test]
    fn test_0nnn(){
        assert!(false);
    }

    #[test]
    fn test_00e0(){
        assert!(false);
    }

    #[test]
    fn test_1nnn(){
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            // start at 202 to not overwrite first instruction
            let nnn = rng.gen_range(0x202,0xFFF);
            let zn = ((nnn >> 0x8) | 0x10) as u8;
            let nn = (nnn & 0x0FF) as u8;

            let mut memory:[u8;0xE00] = [0;0xE00];
            // set jump command
            memory[0] = zn | 0x10;
            memory[1] = nn;

            // set jumped to command
            memory[nnn - 0x200] = 0x65;
            memory[nnn - 0x200 + 1] = 0x55;

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();

            // verify two cycles caused jumped to command to be executed
            assert!(machine.v[0x5] == 0x55);
        }
    }

    #[test]
    fn test_2nnn(){
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            // start at 202 to not overwrite first instruction
            let nnn = rng.gen_range(0x202,0xFFF);
            let zn = ((nnn >> 8) | 0x20) as u8;
            let nn = (nnn & 0x0FF) as u8;

            let mut memory:[u8;0xE00] = [0;0xE00];
            // set call command
            memory[0] = zn;
            memory[1] = nn;

            // set jumped to command
            memory[nnn - 0x200] = 0x65;
            memory[nnn - 0x200 + 1] = 0x55;

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();

            // verify two cycles caused jumped to command to be executed
            assert!(machine.v[0x5] == 0x55);
            assert!(machine.sp == 1);
            assert!(machine.stack[0] == 0x200);
        }
    }

    #[test]
    fn test_3xnn(){
        assert!(false);
    }

    #[test]
    fn test_4xnn(){
        assert!(false);
    }

    #[test]
    fn test_5xy0(){
        assert!(false);
    }

    #[test]
    fn test_6xnn(){
        for _ in 0..1000 {
            let x = rand::random::<u8>() & 0x0F;
            let n = rand::random::<u8>() & 0xFF;

            let memory = [
                0x60 | x, n,
            ];

        let mut machine = Chip8::default();
        machine.load_rom(&memory);
        machine.cycle();

        assert!(machine.v[x as usize] == n);
        }
    }

    #[test]
    fn test_7xnn(){
        for _ in 0..1000 {
            let x = rand::random::<u8>() & 0x0F;
            let xbase = rand::random::<u8>();
            let n = rand::random::<u8>();

            let memory = [
                0x60 | x, xbase,
                0x70 | x, n,
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();

            assert!(machine.v[x as usize] == xbase.wrapping_add(n));
        }
    }

    #[test]
    fn test_8xy1(){
        for _ in 0..1000 {
            let x = rand::random::<u8>() & 0x0F;
            let mut y = x;
            while y == x {y = rand::random::<u8>() & 0x0F;}
            let y = y;

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = rand::random::<u8>();

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x01 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            assert!(machine.v[x as usize] == xval | yval);
        }
    }

    #[test]
    fn test_8xy2(){
        for _ in 0..1000 {
            let x = rand::random::<u8>() & 0x0F;
            let mut y = x;
            while y == x {y = rand::random::<u8>() & 0x0F;}
            let y = y;

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = rand::random::<u8>();

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x02 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            assert!(machine.v[x as usize] == xval & yval);
        }
    }

    #[test]
    fn test_8xy3(){
        for _ in 0..1000 {
            let x = rand::random::<u8>() & 0x0F;
            let mut y = x;
            while y == x {y = rand::random::<u8>() & 0x0F;}
            let y = y;

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = rand::random::<u8>();

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x03 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            assert!(machine.v[x as usize] == xval ^ yval);
        }
    }

    #[test]
    fn test_8xy4(){
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let x = rng.gen_range(0x0, 0x0F);
            let y = rng.gen_range(0x0, 0x0F);

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = if x != y {rand::random::<u8>()} else {xval};

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x04 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            let (sum, flag) = xval.overflowing_add(yval);
            assert!(machine.v[x as usize] == sum);
            assert!(machine.v[0xF] == if flag {1} else {0});
        }
    }

    #[test]
    fn test_8xy5(){
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let x = rng.gen_range(0x0, 0x0F);
            let y = rng.gen_range(0x0, 0x0F);

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = if x != y {rand::random::<u8>()} else {xval};

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x05 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            let (sum, flag) = xval.overflowing_sub(yval);
            assert!(machine.v[x as usize] == sum);
            assert!(machine.v[0xF] == if flag {0} else {1});
        }
    }

    #[test]
    fn test_8xy6(){
        assert!(false);
    }

    #[test]
    fn test_8xy7(){
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let x = rng.gen_range(0x0, 0x0F);
            let y = rng.gen_range(0x0, 0x0F);

            let xval:u8 = rand::random::<u8>();
            let yval:u8 = if x != y {rand::random::<u8>()} else {xval};

            let memory = [
                0x60 | x, xval,
                0x60 | y, yval,
                0x80 | x, 0x07 | (y << 0x4),
            ];

            let mut machine = Chip8::default();
            machine.load_rom(&memory);
            machine.cycle();
            machine.cycle();
            machine.cycle();

            let (sum, flag) = yval.overflowing_sub(xval);
            assert!(machine.v[x as usize] == sum);
            assert!(machine.v[0xF] == if flag {0} else {1});
        }
        assert!(false);
    }

    #[test]
    fn test_8xye(){
        assert!(false);
    }

    #[test]
    fn test_9xy0(){
        assert!(false);
    }

    #[test]
    fn test_annn(){
        assert!(false);
    }

    #[test]
    fn test_bnnn(){
        let mut rng = rand::thread_rng();

        let nnn:u16 = rng.gen_range(0x204, 0xF00);
        let val:u8 = rand::random::<u8>();

        let zn = ((nnn >> 0x8) | 0xB0) as u8;
        let nn = (nnn & 0x0FF) as u8;

        let mut memory:[u8;0xE00] = [0;0xE00];
        memory[0] = 0x60;
        memory[1] = val;
        memory[2] = zn;
        memory[3] = nn;

        memory[(nnn as usize) - 0x200 + (val as usize)] = 0x65;
        memory[(nnn as usize) - 0x200 + (val as usize) + 1] = 0x55;

        let mut machine = Chip8::default();
        machine.load_rom(&memory);
        machine.cycle();
        machine.cycle();
        machine.cycle();

        assert!(machine.v[5] == 0x55);
    }

    #[test]
    fn test_cxnn(){
        assert!(false);
    }

    #[test]
    fn test_dxyn(){
        assert!(false);
    }

    #[test]
    fn test_ex9e(){

        let opt:u16  = 0xE09E;
        let xval = rand::random::<u16>() & 0x0F00;
        let key = xval >> 0x8;

        // set up machine
        let mut machine = Chip8::default();
        machine.key[key as usize] = true;
        assert!(false);
    }

    #[test]
    fn test_exa1(){
        assert!(false);
    }

    #[test]
    fn test_fx07(){
        assert!(false);
    }

    #[test]
    fn test_fx0a(){
        assert!(false);
    }

    #[test]
    fn test_fx15(){
        assert!(false);
    }

    #[test]
    fn test_fx18(){
        assert!(false);
    }

    #[test]
    fn test_fx1e(){
        assert!(false);
    }

    #[test]
    fn test_fx29(){
        assert!(false);
    }

    #[test]
    fn test_fx33(){
        assert!(false);
    }

    #[test]
    fn test_fx55(){
        assert!(false);
    }

    #[test]
    fn test_fx65(){
        assert!(false);
    }
}
