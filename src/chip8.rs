extern crate rand;

use super::std;

use super::memory::Memory;
use super::ncursesio::{Audio, Display, Input, Pixel};

// Constants
///////////////////////////////////////////////////////////////////////

const SCREEN_WIDTH:usize  = 0x40;
const SCREEN_HEIGHT:usize = 0x20;
const FRAME_RATE:u64 = 17;

// Chip-8 Implementation
///////////////////////////////////////////////////////////////////////

pub struct Chip8<'a> {
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
    audio:Audio,
    display:Display<'a>,
    input:Input<'a>,
}

impl<'a> Chip8<'a> {
    pub fn new(audio:Audio, display:Display<'a>, input:Input<'a>) -> Chip8<'a> {

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
            audio:audio,
            display:display,
            input:input,
        }
    }
}

impl<'a> Chip8<'a> {
    pub fn load_rom(&mut self, buff:&[u8]){
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
                self.v[x] = self.input.get_key();
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
                self.audio.beep();
            }
            self.st -= 1;
        }
    }

    pub fn cycle(&mut self){
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
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                let pixel = match self.g[SCREEN_WIDTH*row + col] {
                    true => Pixel::On,
                    false => Pixel::Off,
                };
                let _ = self.display.set(row, col, pixel);
            }
        }
        self.display.refresh();
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
        for key in self.input.get_keys(){
            self.key[key as usize] = true;
        }
    }
}
