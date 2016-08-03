extern crate ncurses;
extern crate rand;

mod memory;

use std::fs::File;
use std::io::prelude::Read;

use memory::Memory;

const SCREEN_WIDTH:usize  = 64;
const SCREEN_HEIGHT:usize = 32;

const FRAME_RATE:u64 = 17;

trait Screen {
    fn draw_screen(&mut self, pixels:[bool; SCREEN_WIDTH*SCREEN_HEIGHT]);
}

trait Input {
    fn set_pushed(&self, &mut [bool;0x10]);
    fn get_key(&self) -> u8;
}

// NCurses Implementation of screen and Input
////////////////////////////////////////////////////////////////////////

struct NCursesScreen {}

impl Screen for NCursesScreen {
    fn draw_screen(&mut self, pixels:[bool; SCREEN_WIDTH*SCREEN_HEIGHT]){
        ncurses::wmove(ncurses::stdscr, 0, 0);
        let mut screen = String::new();
        for i in 0..SCREEN_HEIGHT {
            let line:String = pixels[SCREEN_WIDTH*i..SCREEN_WIDTH*(i+1)]
                .into_iter()
                .map(|&val| if val {'4'} else {' '})
                .collect();
            screen = screen + &(line + "\r\n");
        }
        let screen = screen;
        ncurses::printw(&screen);
        ncurses::refresh();
    }
}

struct NCursesInput {}

impl Input for NCursesInput {
    fn set_pushed(&self, keys:&mut [bool;0x10]){
        keys[..].copy_from_slice(&[false;0x10]);
        ncurses::nodelay(ncurses::stdscr, true);

        loop {
            match ncurses::getch() {
                0x31 => keys[0x1] = true, // 1 -> 1
                0x32 => keys[0x2] = true, // 2 -> 2
                0x33 => keys[0x3] = true, // 3 -> 3
                0x34 => keys[0xC] = true, // 4 -> C
                0x71 => keys[0x4] = true, // q -> 4
                0x77 => keys[0x5] = true, // w -> 5
                0x65 => keys[0x6] = true, // e -> 6
                0x72 => keys[0xD] = true, // r -> D
                0x61 => keys[0x7] = true, // a -> 7
                0x73 => keys[0x8] = true, // s -> 8
                0x64 => keys[0x9] = true, // d -> 9
                0x66 => keys[0xE] = true, // f -> E
                0x7A => keys[0xA] = true, // z -> A
                0x78 => keys[0x0] = true, // x -> 0
                0x63 => keys[0xB] = true, // c -> B
                0x76 => keys[0xF] = true, // v -> F
                ncurses::ERR => break,
                _ =>{},
            }
        }

        ncurses::nodelay(ncurses::stdscr, false);
    }

    fn get_key(&self) -> u8 {
        match ncurses::getch() {
            0x31 => 0x1, // 1 -> 1
            0x32 => 0x2, // 2 -> 2
            0x33 => 0x3, // 3 -> 3
            0x34 => 0xC, // 4 -> C
            0x71 => 0x4, // q -> 4
            0x77 => 0x5, // w -> 5
            0x65 => 0x6, // e -> 6
            0x72 => 0xD, // r -> D
            0x61 => 0x7, // a -> 7
            0x73 => 0x8, // s -> 8
            0x64 => 0x9, // d -> 9
            0x66 => 0xE, // f -> E
            0x7A => 0xA, // z -> A
            0x78 => 0x0, // x -> 0
            0x63 => 0xB, // c -> B
            0x76 => 0xF, // v -> F
            x => x as u8,
        }
    }
}


// Simple ASCII print screen
////////////////////////////////////////////////////////////////////////

struct AsciiScreen {}

impl Screen for AsciiScreen {
    fn draw_screen(&mut self, pixels:[bool; SCREEN_WIDTH*SCREEN_HEIGHT]){
        let mut screen = String::new();
        for i in 0..SCREEN_HEIGHT {
            let line:String = pixels[SCREEN_WIDTH*i..SCREEN_WIDTH*(i+1)]
                .into_iter()
                .map(|&val| if val {'%'} else {'_'})
                .collect();
            screen = screen + &line;
        }
        println!("{}", &screen);
    }
}

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8 {
    v:[u8;0x10], //registers
    i:u16,

    oc:u16, // optcode
    pc:u16, // program counter
    mem:Memory, //memory

    // Stack Variables
    stack:[u16;0x10],
    sp:usize,

    // delay and sound timer
    dt:u8, // delay timer
    st:u8, // sound timer

    g:[bool;SCREEN_WIDTH * SCREEN_HEIGHT], // screen pixels (false == off, true == on)
    key:[bool;0x10], // key state (false == down, true == up)
    df:bool, // draw flag (false == no draw, true == draw)
}

impl Default for Chip8 {
    fn default() -> Chip8 {

        Chip8{
            v:[0;0x10],
            i:0x0,

            oc:0x0,
            pc:0x200,
            mem:Memory::default(),

            stack:[0;0x10],
            sp:0x0,

            dt:0x0,
            st:0x0,

            g:[false;SCREEN_WIDTH * SCREEN_HEIGHT],
            key:[false;0x10],
            df:false,
        }
    }
}

impl Chip8 {
    fn load_rom(&mut self, buff:&[u8]){
        self.mem.set_range(0x200, buff)
    }

    fn read_address(&self, pointer:u16) -> u16{
        let top = (self.mem.read_memory(pointer) as u16) << 0x8;
        let bot = (self.mem.read_memory(pointer + 0x1) & 0xFF) as u16;
        top | bot
    }

    fn fetch_op(&mut self){
        self.oc = self.read_address(self.pc);
    }

    fn decode_op(&mut self){
        let b1 = (self.oc & 0xF000) >> 0xC;
        let b2 = ((self.oc & 0x0F00) >> 0x8) as usize;
        let b3 = ((self.oc & 0x00F0) >> 0x4) as usize;
        let b4 = (self.oc & 0x000F) >> 0x0;

        match (b1, b2, b3, b4) {
            (0x0,0x0,0xE,0x0) => { // clear the screen
                for i in 0..self.g.len() {
                    self.g[i] = false;
                }
                self.pc += 2;
            },
            (0x0,0x0,0xE,0xE) => { // return from subroutine
                self.sp -= 1;
                self.pc = self.stack[self.sp];
                self.pc += 2;
            },
            (0x0,_,_,_) => { // call RCA 1802 program at address NNN
                /* TODO */
                panic!("TODO -- 0x0NNN");
            },
            (0x1,_,_,_) => { // jump to address NNN
                self.pc = self.oc & 0x0FFF;
            },
            (0x2,_,_,_) => { // call subroutine at NNN
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = self.oc & 0x0FFF;
            },
            (0x3,x,_,_) => { // skip next instruction if VX == NN
                match self.v[x] == (self.oc & 0x00FF) as u8 {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0x4,x,_,_) => { // skip next instruction if VX != NN
                match self.v[x] != (self.oc & 0x00FF) as u8 {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0x5,x,y,0x0) => { // skip next instruction if VX == VY
                match self.v[x] == self.v[y] {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0x6,x,_,_) => { // set VX to NN
                self.v[x] = (self.oc & 0x00FF) as u8;
                self.pc += 2;
            },
            (0x7,x,_,_) => { // add NN to VX
                self.v[x] = self.v[x].wrapping_add((self.oc & 0x00FF) as u8);
                self.pc += 2;
            },
            (0x8,x,y,0x0) => { // set VX to VY
                self.v[x] = self.v[y];
                self.pc += 2;
            },
            (0x8,x,y,0x1) => { // or VX with VY
                self.v[x] |= self.v[y];
                self.pc += 2;
            },
            (0x8,x,y,0x2) => { // and VX with VY
                self.v[x] &= self.v[y];
                self.pc += 2;
            },
            (0x8,x,y,0x3) => { // xor VX with VY
                self.v[x] ^= self.v[y];
                self.pc += 2;
            },
            (0x8,x,y,0x4) => { // add VY to VX. VF = if carry {1} else {0}
                let (value, flag) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = value;
                self.v[0xF] = if flag {1} else {0};
                self.pc += 2;
            },
            (0x8,x,y,0x5) => { // sub VY from VX. VF = if borrow {0} else {1}
                let (value, flag) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = value;
                self.v[0xF] = if flag {0} else {1};
                self.pc += 2;
            },
            (0x8,x,_,0x6) => { // shift VX right. VF set to dropped bit.
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
                self.pc += 2;
            },
            (0x8,x,y,0x7) => { // set VX to VY - VX. if borrow {0} else {1}
                let (value, flag) =  self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = value;
                self.v[0xF] = if flag {0} else {1};
                self.pc += 2;
            },
            (0x8,x,_,0xE) => { // shift VX left. VF set to dropped bit.
               self.v[0xF] = (self.v[x] >> 0x7) & 0x1;
               self.v[x] <<= 1;
               self.pc += 2;
            },
            (0x9,x,y,0x0) => { // skip next instruction if VX != VY
                match self.v[x] != self.v[y] {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0xA,_,_,_) => { // set I to NNN
                self.i = self.oc & 0x0FFF;
                self.pc += 2;
            },
            (0xB,_,_,_) => { // jump to address NNN + V0
                self.pc = (self.oc & 0x0FFF) + (self.v[0] as u16);
            },
            (0xC,x,_,_) => { // set VX to random number & NN
                self.v[x] = rand::random::<u8>() & (self.oc & 0xFF) as u8;
                self.pc += 2;
            },
            (0xD,x,y,n) => { // complicated
                let x = self.v[x] as usize;
                let y = self.v[y] as usize;
                self.draw_sprite(x, y, n as usize);
                self.pc += 2;
            },
            (0xE,x,0x9,0xE) => { // skip next if key in VX is pressed;
                match self.key[self.v[x] as usize] {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0xE,x,0xA,0x1) => { // skip next if key in VX is not pressed;
                match self.key[self.v[x] as usize] {
                    true  => self.pc += 2,
                    false => self.pc += 4,
                }
            },
            (0xF,x,0x0,0x7) => {
                self.v[x] = self.dt;
                self.pc += 2;
            },
            (0xF,x,0x0,0xA) => { // a keypress is awaited, then stored in v[x]
                self.v[x] = ncurses::getch() as u8;
            },
            (0xF,x,0x1,0x5) => { // set delay timer to VX
                self.dt = self.v[x];
                self.pc += 2;
            },
            (0xF,x,0x1,0x8) => {
                self.st = self.v[x];
                self.pc += 2;
            },
            (0xF,x,0x1,0xE) => {
                let (value, flag) = self.i.overflowing_add(self.v[x] as u16);
                self.i = value;
                self.v[0xF] = if flag {1} else {0};
                self.pc += 2;
            },
            (0xF,x,0x2,0x9) => {
                self.i = (self.v[x] as u16)*5;
                self.pc += 2;
            },
            (0xF,x,0x3,0x3) => {
                let i = self.i;
                let vx = self.v[x];
                self.mem.write_memory(i,vx/100);
                self.mem.write_memory(i+1,(vx/10)%100);
                self.mem.write_memory(i+2,(vx%100)%10);
                self.pc += 2;
            },
            (0xF,x,0x5,0x5) => { // stores V0 to VX (inclusive) starting at I
                let index = self.i;
                self.mem.set_range(index, &self.v[0..(x+1)]);
                self.pc += 2;
            },
            (0xF,x,0x6,0x5) => { // fills V0 to VX (inclusive) starting from I
                for i in 0..(x+1) as u16{
                    self.v[i as usize] = self.mem.read_memory(self.i + i);
                }
                self.pc += 2;
            },
            _ => panic!("unknown instruction received"),
        };
    }

    fn dec_dt(&mut self){
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    fn dec_st(&mut self){
        if self.st > 0 {
            if self.st == 1 {
                ncurses::beep();
            }
            self.st -= 1;
        }
    }

    fn cycle(&mut self){
        self.fetch_op();
        self.decode_op();
        self.dec_dt();
        self.dec_st();

        self.set_pushed();
        if self.df {
            self.print_screen();
            self.df = false;
            std::thread::sleep(std::time::Duration::from_millis(FRAME_RATE));
        }
    }

    fn print_screen(&self){
        ncurses::wmove(ncurses::stdscr, 0, 0);
        let mut screen = String::new();
        for i in 0..SCREEN_HEIGHT {
            let line:String = self.g[SCREEN_WIDTH*i..SCREEN_WIDTH*(i+1)]
                .into_iter()
                .map(|&val| if val {'4'} else {' '})
                .collect();
            screen = screen + &(line + "\n");
        }
        ncurses::printw(&screen);
        ncurses::refresh();
    }

    fn draw_sprite(&mut self, x:usize, y:usize, height:usize){
        self.v[0xF] = 0x0;
        let location = x + y*SCREEN_WIDTH;
        let index = self.i as usize;
        for i in 0..height {
            if (y + i >= 0) || (y + i < SCREEN_HEIGHT) {
                let byte = self.mem.read_memory((index + i) as u16) as usize;
                self.draw_byte(location + i*SCREEN_WIDTH, byte);
            }
        }
    }

    fn draw_byte(&mut self, loc:usize, byte:usize){
        for i in 0..8 {
            if (byte & (0x80 >> i)) != 0 {
                self.df = true;
                if self.g[loc+i] {self.v[0xF] = 0x1};
                self.g[loc+i] ^= true;
            }
        }
    }

    fn set_pushed(&mut self){
        self.key[..].copy_from_slice(&[false;0x10]);
        ncurses::nodelay(ncurses::stdscr, true);

        loop {
            match ncurses::getch() {
                0x31 => self.key[0x0] = true, // 1
                0x32 => self.key[0x1] = true, // 2
                0x33 => self.key[0x2] = true, // 3
                0x34 => self.key[0x3] = true, // 4
                0x71 => self.key[0x4] = true, // q
                0x77 => self.key[0x5] = true, // w
                0x65 => self.key[0x6] = true, // e
                0x72 => self.key[0x7] = true, // r
                0x61 => self.key[0x8] = true, // a
                0x73 => self.key[0x9] = true, // s
                0x64 => self.key[0xA] = true, // d
                0x66 => self.key[0xB] = true, // f
                0x7A => self.key[0xC] = true, // z
                0x78 => self.key[0xD] = true, // x
                0x63 => self.key[0xE] = true, // c
                0x76 => self.key[0xF] = true, // v
                ncurses::ERR => break,
                _ =>{},
            }
        }

        ncurses::nodelay(ncurses::stdscr, false);
    }
}

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

    let mut machine = Chip8::default();
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
