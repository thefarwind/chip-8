#![cfg(test)]
extern crate rand;

use super::*;
use super::std;
use self::rand::Rng;

use super::io::{Audio, Display, Input};

// Mock IO Devices
////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct MockAudio {
    beeped:std::cell::Cell<bool>,
}

impl Default for MockAudio {
    fn default() -> MockAudio {
        MockAudio{beeped:std::cell::Cell::new(false)}
    }
}

impl Audio for MockAudio {
    fn beep(&self){
        self.beeped.set(true);
    }
}

const SCREEN_SIZE:usize = io::SCREEN_WIDTH*io::SCREEN_HEIGHT;

struct MockDisplay {
    waiting:[io::Pixel;SCREEN_SIZE],
    drawn:[io::Pixel;SCREEN_SIZE],
}

impl Default for MockDisplay {
    fn default() -> MockDisplay{
        MockDisplay{
            waiting:[io::Pixel::Off;SCREEN_SIZE],
            drawn:[io::Pixel::Off;SCREEN_SIZE],
        }
    }
}

impl Display for MockDisplay {
    fn set(&mut self, row:usize, col:usize, state:io::Pixel) -> Result<(),()>{

        match ((row >= io::SCREEN_HEIGHT),(col >= io::SCREEN_WIDTH)){
            (true,_)|(_,true) => {
                Err(())
            },
            (false,false) => {
                self.waiting[row*io::SCREEN_WIDTH + col] = state;
                Ok(())
            },
        }
    }
    fn refresh(&mut self){
        self.drawn[..].copy_from_slice(&self.waiting);
    }
}

#[derive(Default, Debug)]
struct MockInput {
    pressed:Vec<u8>,
}

impl MockInput {
    fn set(&mut self, value:u8){
        self.pressed.push(value)
    }
    fn clear(&mut self){
        self.pressed.clear()
    }
}

impl Input for MockInput {
    fn get_keys(&self) -> Vec<u8> {
        self.pressed.clone()
    }
    fn get_key(&self) -> u8 {
        self.pressed[0]
    }
}

// Mock Device Tests
////////////////////////////////////////////////////////////////////////

#[test]
fn test_mock_audio(){
    let mock = MockAudio::default();
    assert!(!mock.beeped.get());
    mock.beep();
    assert!(mock.beeped.get());
}

#[test]
fn test_mock_input_set(){
    let mut mock = MockInput::default();

    for i in 0..0x10 {
        mock.set(i);
    }

    for i in 0..0x10 {
        assert_eq!(mock.pressed[i],i as u8);
    }
}

#[test]
fn test_mock_input_clear(){
    let mut mock = MockInput::default();

    for i in 0..0x10 {
        mock.set(i);
    }

    mock.clear();
    assert!(mock.pressed.is_empty());
}

#[test]
fn test_mock_input_get_keys(){
    let mut mock = MockInput::default();

    for i in 0..0x10 {
        mock.set(i);
    }

    let keys = mock.get_keys();

    for i in 0..0x10 {
        assert_eq!(i as u8,keys[i]);
    }
}

#[test]
fn test_mock_input_get_key(){
    let mut mock = MockInput::default();

    for i in 0..0x10 {
        mock.set(i);
        assert_eq!(mock.get_key(),i);
        mock.clear();
    }
}

#[test]
fn test_mock_display_set(){
    let mut mock = MockDisplay::default();
    let mut tracker = [io::Pixel::Off;SCREEN_SIZE];

    for i in 0..SCREEN_SIZE {

        let pixel = match rand::random::<bool>() {
            true => io::Pixel::On,
            false => io::Pixel::Off,
        };

        tracker[i] = pixel;
        mock.set(i/io::SCREEN_WIDTH, i%io::SCREEN_WIDTH, pixel).unwrap();
    }

    for i in 0..SCREEN_SIZE {
        assert_eq!(tracker[i], mock.waiting[i]);
        assert_eq!(io::Pixel::Off, mock.drawn[i]);
    }
}

#[test]
fn test_mock_display_refresh(){
    let mut mock = MockDisplay::default();
    let mut tracker = [io::Pixel::Off;SCREEN_SIZE];

    for i in 0..SCREEN_SIZE {

        let pixel = match rand::random::<bool>() {
            true => io::Pixel::On,
            false => io::Pixel::Off,
        };

        tracker[i] = pixel;
        mock.set(i/io::SCREEN_WIDTH, i%io::SCREEN_WIDTH, pixel).unwrap();
    }

    mock.refresh();

    for i in 0..SCREEN_SIZE {
        assert_eq!(tracker[i], mock.waiting[i]);
        assert_eq!(tracker[i], mock.drawn[i]);
    }
}

// Processor Tests
////////////////////////////////////////////////////////////////////////
/*
#[test]
#[ignore]
fn test_0nnn(){
    assert!(false);
}

#[test]
#[ignore]
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
#[ignore]
fn test_3xnn(){
    assert!(false);
}

#[test]
#[ignore]
fn test_4xnn(){
    assert!(false);
}

#[test]
#[ignore]
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
#[ignore]
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
#[ignore]
fn test_8xye(){
    assert!(false);
}

#[test]
#[ignore]
fn test_9xy0(){
    assert!(false);
}

#[test]
#[ignore]
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
#[ignore]
fn test_cxnn(){
    assert!(false);
}

#[test]
#[ignore]
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
#[ignore]
fn test_exa1(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx07(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx0a(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx15(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx18(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx1e(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx29(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx33(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx55(){
    assert!(false);
}

#[test]
#[ignore]
fn test_fx65(){
    assert!(false);
}*/
