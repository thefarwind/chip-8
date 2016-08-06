extern crate rand;

use super::std;

use super::bus::Bus;
use super::memory::Memory;
use super::ncursesio::{Audio, Display, Input, Pixel};

// Constants
///////////////////////////////////////////////////////////////////////

const SCREEN_WIDTH:usize  = 0x40;
const SCREEN_HEIGHT:usize = 0x20;
const FRAME_RATE:u64 = 17;

// Processor
////////////////////////////////////////////////////////////////////////

pub struct Processor {
    oc:u16, // Opt Code
    pc:u16, // Program Counter
    sp:u16, // Stack Pointer
    index:u16, // Index

    delay_timer:u8,
    sound_timer:u8,

    draw_flag:bool,

    v:[u8;0x10],
    stack:[u16;0x10],
    screen:[bool;SCREEN_WIDTH*SCREEN_HEIGHT],
    keys:[bool;0x10],
}

impl Default for Processor {
    fn default() -> Processor {
        Processor{
            oc:0x0,
            pc:0x200,
            sp:0x0,
            index:0x0,

            delay_timer:0x0,
            sound_timer:0x0,

            draw_flag:false,

            v:[0x0;0x10],
            stack:[0x0;0x10],
            screen:[false;SCREEN_WIDTH*SCREEN_HEIGHT],
            keys:[false;0x10],
        }
    }
}

impl Processor {

    // non-self functions

    fn read_address(pointer:u16, memory:&Memory) -> u16 {
        let top = (memory.read_memory(pointer) as u16) << 0x8;
        let bot = (memory.read_memory(pointer + 0x1) & 0xFF) as u16;
        top | bot
    }

    // &self functions



    // &mut self functions

    fn load_optcode(&mut self, memory:&Memory){
        self.oc = Processor::read_address(self.pc, memory);
    }

    fn run_optcode(&mut self, bus:&mut Bus){
        let b1 = (self.oc & 0xF000) >> 0xC;
        let b2 = ((self.oc & 0x0F00) >> 0x8) as usize;
        let b3 = ((self.oc & 0x00F0) >> 0x4) as usize;
        let b4 = (self.oc & 0x000F) >> 0x0;

        match (b1, b2, b3, b4) {
            (0x0,0x0,0xE,0x0) => { // clear the screen
                self.screen[..]
                        .copy_from_slice(&[false;SCREEN_WIDTH*SCREEN_HEIGHT]);
                self.pc += 2;
            },
            (0x0,0x0,0xE,0xE) => { // return from subroutine
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
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
                self.stack[self.sp as usize] = self.pc;
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
                self.index = self.oc & 0x0FFF;
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
                self.draw_sprite(x, y, n as usize, &bus.memory);
                self.pc += 2;
            },
            (0xE,x,0x9,0xE) => { // skip next if key in VX is pressed;
                match self.keys[self.v[x] as usize] {
                    true  => self.pc += 4,
                    false => self.pc += 2,
                }
            },
            (0xE,x,0xA,0x1) => { // skip next if key in VX is not pressed;
                match self.keys[self.v[x] as usize] {
                    true  => self.pc += 2,
                    false => self.pc += 4,
                }
            },
            (0xF,x,0x0,0x7) => {
                self.v[x] = self.delay_timer;
                self.pc += 2;
            },
            (0xF,x,0x0,0xA) => { // a keypress is awaited, then stored in v[x]
                self.v[x] = bus.input.get_key();
            },
            (0xF,x,0x1,0x5) => { // set delay timer to VX
                self.delay_timer = self.v[x];
                self.pc += 2;
            },
            (0xF,x,0x1,0x8) => {
                self.sound_timer = self.v[x];
                self.pc += 2;
            },
            (0xF,x,0x1,0xE) => {
                let (value, flag) = self.index.overflowing_add(self.v[x] as u16);
                self.index = value;
                self.v[0xF] = if flag {1} else {0};
                self.pc += 2;
            },
            (0xF,x,0x2,0x9) => {
                self.index = (self.v[x] as u16)*5;
                self.pc += 2;
            },
            (0xF,x,0x3,0x3) => {
                let i = self.index;
                let vx = self.v[x];
                bus.memory.write_memory(i,vx/100);
                bus.memory.write_memory(i+1,(vx/10)%100);
                bus.memory.write_memory(i+2,(vx%100)%10);
                self.pc += 2;
            },
            (0xF,x,0x5,0x5) => { // stores V0 to VX (inclusive) starting at I
                let index = self.index;
                bus.memory.set_range(index, &self.v[0..(x+1)]);
                self.pc += 2;
            },
            (0xF,x,0x6,0x5) => { // fills V0 to VX (inclusive) starting from I
                for i in 0..(x+1) as u16{
                    self.v[i as usize] = bus.memory.read_memory(self.index + i);
                }
                self.pc += 2;
            },
            _ => panic!("unknown instruction received"),
        };
    }

    fn decrement_delay_timer(&mut self){
        if self.delay_timer > 0x0 {
            self.delay_timer -= 0x1;
        }
    }

    fn decrement_sound_timer(&mut self, audio:&Audio){
        if self.sound_timer > 0x0 {
            if self.sound_timer == 0x1 {
                audio.beep();
            }
            self.sound_timer -= 0x1;
        }
    }

    fn print_screen(&self, display:&Display){
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                let pixel = match self.screen[SCREEN_WIDTH*row + col] {
                    true => Pixel::On,
                    false => Pixel::Off,
                };
                let _ = display.set(row, col, pixel);
            }
        }
        display.refresh();
    }

    fn draw_sprite(&mut self, x:usize, y:usize, height:usize, memory:&Memory){
        self.v[0xF] = 0x0;
        let location = x + y*SCREEN_WIDTH;
        let index = self.index as usize;
        for i in 0..height {
            if (y + i >= 0) || (y + i < SCREEN_HEIGHT) {
                let byte = memory.read_memory((index + i) as u16) as usize;
                self.draw_byte(location + i*SCREEN_WIDTH, byte);
            }
        }
    }

    fn draw_byte(&mut self, loc:usize, byte:usize){
        for i in 0..8 {
            if (byte & (0x80 >> i)) != 0 {
                self.draw_flag = true;
                if self.screen[loc+i] {self.v[0xF] = 0x1};
                self.screen[loc+i] ^= true;
            }
        }
    }

    fn set_pushed(&mut self, input:&Input){
        self.keys[..].copy_from_slice(&[false;0x10]);
        for key in input.get_keys(){
            self.keys[key as usize] = true;
        }
    }
    // pub &mut self functions

    pub fn cycle(&mut self, bus:&mut Bus){
        self.load_optcode(&bus.memory);
        self.run_optcode(bus);
        self.decrement_delay_timer();
        self.decrement_sound_timer(&bus.audio);

        self.set_pushed(&bus.input);
        if self.draw_flag {
            self.print_screen(&bus.display);
            self.draw_flag = false;
            std::thread::sleep(std::time::Duration::from_millis(FRAME_RATE));
        }
    }
}
