extern crate rand;

use super::std;
use super::std::time::{Duration, Instant};

use super::bus::Bus;
use super::io::{Audio, Display, Input, Pixel};
use super::io::{SCREEN_WIDTH, SCREEN_HEIGHT};
use super::memory::Memory;

// Constants
///////////////////////////////////////////////////////////////////////

const FRAME_RATE:u64 = 17;

// Processor
////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
enum Key {
    Up,
    Down(Instant),
}

pub struct Processor {
    oc:u16, // Operational Code
    pc:u16, // Program Counter
    sp:u16, // Stack Pointer
    index:u16, // Index

    delay_timer:u8,
    sound_timer:u8,

    draw_flag:bool,

    v:[u8;0x10],
    stack:[u16;0x10],
    screen:[bool;SCREEN_WIDTH*SCREEN_HEIGHT],
    keys:[Key;0x10],
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
            keys:[Key::Up;0x10],
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

    fn load_opcode(&mut self, memory:&Memory){
        self.oc = Processor::read_address(self.pc, memory);
    }

    fn run_opcode<A, D, I>(&mut self, bus:&mut Bus<A, D, I>)
            where
                A: Audio,
                D: Display,
                I: Input {
        let b1 = (self.oc & 0xF000) >> 0xC;
        let b2 = ((self.oc & 0x0F00) >> 0x8) as usize;
        let b3 = ((self.oc & 0x00F0) >> 0x4) as usize;
        let b4 = self.oc & 0x000F;

        match (b1, b2, b3, b4) {
            (0x0,0x0,0xE,0x0) => { // clear the screen
                self.screen[..]
                        .copy_from_slice(&[false;SCREEN_WIDTH*SCREEN_HEIGHT]);
                self.draw_flag = true;
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
                self.pc += if self.v[x] as u16 == (self.oc & 0x00FF) {
                    4
                } else {
                    2
                };
            },
            (0x4,x,_,_) => { // skip next instruction if VX != NN
                self.pc += if self.v[x] as u16 != (self.oc & 0x00FF) {
                    4
                } else {
                    2
                };
            },
            (0x5,x,y,0x0) => { // skip next instruction if VX == VY
                self.pc += if self.v[x] == self.v[y] {
                    4
                } else {
                    2
                };
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
                let bit = self.v[x] & 0x1;
                self.v[x] >>= 1;
                self.v[0xF] = bit;
                self.pc += 2;
            },
            (0x8,x,y,0x7) => { // set VX to VY - VX. if borrow {0} else {1}
                let (value, flag) =  self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = value;
                self.v[0xF] = if flag {0} else {1};
                self.pc += 2;
            },
            (0x8,x,_,0xE) => { // shift VX left. VF set to dropped bit.
               let bit = (self.v[x] >> 0x7) & 0x1;
               self.v[x] <<= 1;
               self.v[0xF] = bit;
               self.pc += 2;
            },
            (0x9,x,y,0x0) => { // skip next instruction if VX != VY
                self.pc += if self.v[x] != self.v[y] {
                    4
                } else {
                    2
                };
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
                    Key::Down(_) => self.pc += 4,
                    Key::Up => self.pc += 2,
                }
                self.keys[self.v[x] as usize] = Key::Up;
            },
            (0xE,x,0xA,0x1) => { // skip next if key in VX is not pressed;
                match self.keys[self.v[x] as usize] {
                    Key::Down(_)  => self.pc += 2,
                    Key::Up => self.pc += 4,
                }
                self.keys[self.v[x] as usize] = Key::Up;
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

    fn decrement_sound_timer<A:Audio>(&mut self, audio:&A){
        if self.sound_timer > 0x0 {
            if self.sound_timer == 0x1 {
                audio.beep();
            }
            self.sound_timer -= 0x1;
        }
    }

    fn print_screen<D:Display>(&self, display:&mut D){
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                let pixel = if self.screen[SCREEN_WIDTH*row + col] {
                    Pixel::On
                } else {
                    Pixel::Off
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
            if y + i < SCREEN_HEIGHT {
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

    fn set_pushed<I:Input>(&mut self, input:&I){
        let timeout = Duration::new(1,0);

        for key in &mut self.keys {
            *key = match *key {
                Key::Down(x) if x.elapsed() > timeout => Key::Up,
                _ => *key,
            };
        }

        for key in input.get_keys(){
            self.keys[key as usize] = Key::Down(Instant::now());
        }
    }
    // pub &mut self functions

    pub fn cycle<A, D, I>(&mut self, bus:&mut Bus<A, D, I>)
            where
                A: Audio,
                D: Display,
                I: Input {
        self.load_opcode(&bus.memory);
        self.run_opcode(bus);
        self.decrement_delay_timer();
        self.decrement_sound_timer(&bus.audio);

        if self.draw_flag {
            self.print_screen(&mut bus.display);
            self.draw_flag = false;
            std::thread::sleep(Duration::from_millis(FRAME_RATE));
        }
        self.set_pushed(&bus.input);
    }
}
