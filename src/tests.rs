#![cfg(test)]
extern crate rand;

use super::*;
use super::std;
use self::rand::Rng;

use super::bus;
use super::io::{Audio, Display, Input};
use super::memory;
use super::processor;

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

// Helper Functions
////////////////////////////////////////////////////////////////////////
fn new_mock_bus() -> bus::Bus<MockAudio, MockDisplay, MockInput>{
    bus::Bus::new(
        MockAudio::default(),
        MockDisplay::default(),
        MockInput::default())
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

// Memory Tests
////////////////////////////////////////////////////////////////////////

#[test]
fn test_memory_write(){
    let mut memory = memory::Memory::default();
    let mut values = [0x0u8;memory::RAM_SIZE];

    for i in 0x0..memory::RAM_SIZE {
        for j in 0x0..0x100 {
            let index = i as u16;;
            let value = j as u8;

            values[i] = value;
            memory.write_memory(index, value);
            assert_eq!(values[i], memory.read_memory(index));
        }
    }
}

#[test]
fn test_memory_set_range(){
    let mut memory = memory::Memory::default();
    let mut values = [0x0u8;memory::RAM_SIZE];

    for i in 0x0..(memory::RAM_SIZE - 0x200) {
        values[i] = rand::random::<u8>();
    }

    memory.set_range(0x200, &values[0x200..]);

    for i in 0x200..memory::RAM_SIZE {
        assert_eq!(values[i], memory.read_memory(i as u16));
    }
}

// Processor Tests
////////////////////////////////////////////////////////////////////////

#[test]
#[should_panic(expected="TODO -- 0x0NNN")]
fn test_0nnn(){
    let mut bus = new_mock_bus();
    let mut processor = processor::Processor::default();
    processor.cycle(&mut bus);
}

#[test]
fn test_00e0(){
    let mut bus = new_mock_bus();
    for i in 0x0..SCREEN_SIZE {
        bus.display.waiting[i] = if (i & 0x1) == 0x1 {
            io::Pixel::On
        } else {
            io::Pixel::Off
        };
    }
    bus.display.drawn[..].copy_from_slice(&bus.display.waiting);
    bus.memory.write_memory(0x201, 0xE0);

    let mut processor = processor::Processor::default();
    processor.cycle(&mut bus);

    for i in 0x0..SCREEN_SIZE {
        assert_eq!(bus.display.waiting[i], io::Pixel::Off);
        assert_eq!(bus.display.drawn[i], io::Pixel::Off);
    }
}

#[test]
fn test_1nnn(){
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let nnn = rng.gen_range(0x204, 0xFFC);
        let zn = ((nnn >> 0x8) | 0x10) as u8;
        let nn = (nnn & 0x0FF) as u8;

        let mut memory:[u8;0xE00] = [0;0xE00];
        memory[0x0..0x4].copy_from_slice(&[
            0xA3, 0x00,
            zn | 0x10, nn,
        ]);

        // set V0 to 0x55
        let start = nnn - 0x200;
        memory[start..(start+0x4)].copy_from_slice(&[
            0x60, 0x55,
            0xF0, 0x55,
        ]);

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), 0x55);
    }
}

#[test]
fn test_2nnn_00ee_values(){
    for index in 0x206..0xFFC {
        let n = (index >> 0x8) as u8;
        let nn = (index & 0xFF) as u8;

        let mut memory = [0x0u8;0xE00];

        let block = [
            0xA3, 0x00,     // Set Index to 0x300
            0x20 | n, nn,   // call function at location NNN
            0xFF, 0x55,     // Read V[0-F] to 0x300-F
        ];
        memory[..0x6].copy_from_slice(&block);

        let block = [
            0x60, 0x55,     // copy 0x55 into v0
            0x00, 0xEE,     // return from function call
        ];
        memory[(index-0x200)..(index-0x1FC)].copy_from_slice(&block);

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), 0x55);
    }
}

#[test]
fn test_2nnn_00ee_depth_and_back(){
    let mut memory = [0x0u8;0x200];

    let mut base = 0x0;
    for _ in 0x0..0x10 {
        let next = base + 0x10;
        // call subroutine
        memory[base] = (next >> 0x8) as u8 + 0x22;
        memory[base + 1] = (next & 0xFF) as u8;
        // exit subroutine
        memory[base + 2] = 0x00;
        memory[base + 3] = 0xEE;
        base = next;
    }
    // exit subroutine
    memory[base] = 0x00;
    memory[base+1] = 0xEE;


    let mut bus = new_mock_bus();
    bus.memory.set_range(0x200, &memory);

    let mut processor = processor::Processor::default();
    for _ in 0x0..0x20 {
        processor.cycle(&mut bus);
    }
}

#[test]
#[should_panic(expected="subtract with overflow")]
fn test_2nnn_00ee_panic_stack_overflow_bottom(){
    let mut memory = [0x0u8;0x200];

    let mut base = 0x0;
    for _ in 0x0..0x10 {
        let next = base + 0x10;
        // call subroutine
        memory[base] = (next >> 0x8) as u8 + 0x22;
        memory[base + 1] = (next & 0xFF) as u8;
        // exit subroutine
        memory[base + 2] = 0x00;
        memory[base + 3] = 0xEE;
        base = next;
    }
    // exit subroutine
    memory[base] = 0x00;
    memory[base+1] = 0xEE;


    let mut bus = new_mock_bus();
    bus.memory.set_range(0x200, &memory);

    // the 0x11th 00ee call panics due to popping too much off the stack
    let mut processor = processor::Processor::default();
    for _ in 0x0..0x21 {
        processor.cycle(&mut bus);
    }
}

#[test]
#[should_panic(expected="index out of bounds")]
fn test_2nnn_00ee_panic_stack_overflow_top(){
    let mut memory = [0x0u8;0x200];

    let mut base = 0x0;
    for _ in 0x0..0x11 {
        let next = base + 0x10;
        // call subroutine
        memory[base] = (next >> 0x8) as u8 + 0x22;
        memory[base + 1] = (next & 0xFF) as u8;
        base = next;
    }

    let mut bus = new_mock_bus();
    bus.memory.set_range(0x200, &memory);

    // the 0x11th 2nnn call panics due to exceeding stack size
    let mut processor = processor::Processor::default();
    for _ in 0x0..0x11 {
        processor.cycle(&mut bus);
    }
}

#[test]
fn test_3xnn_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let val = rand::random::<u8>();

        let mut test = rand::random::<u8>();
        while (test == val)|(test == 0) {test = rand::random::<u8>()};
        let test = test;

        let memory = [
            0xA3, 0x00,     // set index to 0x300
            0x60 | x, val,  // set vx to val
            0x30 | x, val,  // compare vx value to val
            0x00, 0x00,     // skipped
            0x60, test,     // set v0 to test value where test != val
            0xFF, 0x55,     // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_3xnn_not_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let x_val = rand::random::<u8>();

        let mut nn = rand::random::<u8>();
        while nn == x_val {nn = rand::random::<u8>()};
        let nn = nn;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == nn) | (test == 0) {
            test = rand::random::<u8>()
        };
        let test = test;

        let memory = [
            0xA3, 0x00,         // set index to 0x300
            0x60 | x, x_val,    // set vx to val
            0x30 | x, nn,       // compare vx value to val
            0x60, test,         // set v0 to test value where test != val
            0xFF, 0x55,         // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_4xnn_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let val = rand::random::<u8>();

        let mut test = rand::random::<u8>();
        while (test == val)|(test == 0) {test = rand::random::<u8>()};
        let test = test;

        let memory = [
            0xA3, 0x00,     // set index to 0x300
            0x60 | x, val,  // set vx to val
            0x40 | x, val,  // compare vx value to val
            0x60, test,     // set v0 to test value where test != val
            0xFF, 0x55,     // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_4xnn_not_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let x_val = rand::random::<u8>();

        let mut nn = rand::random::<u8>();
        while nn == x_val {nn = rand::random::<u8>()};
        let nn = nn;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == nn) | (test == 0) {
            test = rand::random::<u8>()
        };
        let test = test;

        let memory = [
            0xA3, 0x00,         // set index to 0x300
            0x60 | x, x_val,    // set vx to val
            0x40 | x, nn,       // compare vx value to val
            0x00, 0x00,         // skipped
            0x60, test,         // set v0 to test value where test != val
            0xFF, 0x55,         // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_5xy0_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val = rand::random::<u8>();
        let y_val = x_val;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == 0) {test = rand::random::<u8>()};
        let test = test;

        let memory = [
            0xA3, 0x00,                     // set index to 0x300
            0x60 | x, x_val,                // set vx to x_val
            0x60 | y, y_val,                // set vy to y_val
            0x50 | x, (y << 0x4),           // check skip
            0x00 , 0x00,                    // nop
            0x60 , test,                    // set v0 to test
            0xFF, 0x55,                     // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_5xy0_not_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;

        let mut y = rand::random::<u8>() & 0x0F;
        while y == x {y = rand::random::<u8>() & 0x0F};
        let y = y;

        let x_val = rand::random::<u8>();

        let mut y_val = rand::random::<u8>();
        while y_val == x_val {y_val = rand::random::<u8>()};
        let y_val = y_val;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == y_val) | (test == 0) {
            test = rand::random::<u8>()
        };
        let test = test;

        let memory = [
            0xA3, 0x00,             // set index to 0x300
            0x60 | x, x_val,        // set vx to x_val
            0x60 | y, y_val,        // set vy to y_val
            0x50 | x, (y << 0x4),   // check skip
            0x60 , test,            // set v0 to test
            0xFF, 0x55,             // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_6xnn(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let n = rand::random::<u8>() & 0xFF;

        let memory = [
            0xA3, 0x00,     // set index to 300
            0x60 | x, n,    // set VX to NN
            0xFF, 0x55,     // set 300-30F to V0-VF
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300 + (x as u16)),n);
    }
}

#[test]
fn test_7xnn(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let x_base = rand::random::<u8>();
        let n = rand::random::<u8>();

        let memory = [
            0xA3, 0x00,         // set index to 300
            0x60 | x, x_base,   // set VX to a x_base
            0x70 | x, n,        // add n to VX
            0xFF, 0x55,         // copy V0-VF to 300-30F
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300 + (x as u16)),
                x_base.wrapping_add(n));
    }
}

#[test]
fn test_8xy0(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;
        let y_val = rand::random::<u8>();

        let memory = [
            0xA3, 0x00,
            0x60 | y, y_val,
            0x80 | x, 0x00 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300 + x as u16), y_val);
    }
}

#[test]
fn test_8xy1(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x01 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300 + x as u16), x_val | y_val);
    }
}

#[test]
fn test_8xy2(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x02 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300+ x as u16), x_val & y_val);
    }
}

#[test]
fn test_8xy3(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x03 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300 + x as u16), x_val ^ y_val);
    }
}

#[test]
fn test_8xy4(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x04 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        let (sum, flag) = x_val.overflowing_add(y_val);

        if x != 0xF {
            assert_eq!(bus.memory.read_memory(0x300 + x as u16), sum);
        }

        assert_eq!(bus.memory.read_memory(0x30F), if flag {1} else {0});
    }
}

#[test]
fn test_8xy5(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x05 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        let (sum, flag) = x_val.overflowing_sub(y_val);
        if x != 0xF {
            assert_eq!(bus.memory.read_memory(0x300 + x as u16), sum);
        }
        assert_eq!(bus.memory.read_memory(0x30F), if flag {0} else {1});
    }
}

#[test]
fn test_8xy6(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val = rand::random::<u8>();

        let memory = [
            0xA3, 0x00,                     // set index to 0x300
            0x60 | x, x_val,                // set vx to x_val
            0x80 | x, 0x06 | (y << 0x4),    // do rshift
            0xFF, 0x55,                     // set 0x300-0x30F to V[0..F]
        ];

        let f_val = x_val & 0x1;
        let t_val = x_val >> 0x1;

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        if x != 0xF {
            assert_eq!(bus.memory.read_memory(0x300 + x as u16), t_val);
        }
        assert_eq!(bus.memory.read_memory(0x30F), f_val);
    }
}

#[test]
fn test_8xy7(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val:u8 = rand::random::<u8>();
        let y_val:u8 = if y == x {x_val} else {rand::random::<u8>()};

        let memory = [
            0xA3, 0x00,
            0x60 | x, x_val,
            0x60 | y, y_val,
            0x80 | x, 0x07 | (y << 0x4),
            0xFF, 0x55,
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        let (sum, flag) = y_val.overflowing_sub(x_val);
        if x != 0xF {
            assert_eq!(bus.memory.read_memory(0x300 + x as u16), sum);
        }
        assert_eq!(bus.memory.read_memory(0x30F), if flag {0} else {1});
    }
}

#[test]
fn test_8xye(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val = rand::random::<u8>();

        let memory = [
            0xA3, 0x00,                     // set index to 0x300
            0x60 | x, x_val,                // set vx to x_val
            0x80 | x, 0x0E | (y << 0x4),    // do rshift
            0xFF, 0x55,                     // set 0x300-0x30F to V[0..F]
        ];

        let f_val = if x_val > 0x7F {1} else {0};
        let t_val = x_val << 0x1;

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        if x != 0xF {
            assert_eq!(bus.memory.read_memory(0x300 + x as u16), t_val);
        }
        assert_eq!(bus.memory.read_memory(0x30F), f_val);
    }
}

#[test]
fn test_9xy0_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;
        let y = rand::random::<u8>() & 0x0F;

        let x_val = rand::random::<u8>();
        let y_val = x_val;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == 0) {test = rand::random::<u8>()};
        let test = test;

        let memory = [
            0xA3, 0x00,                     // set index to 0x300
            0x60 | x, x_val,                // set vx to x_val
            0x60 | y, y_val,                // set vy to y_val
            0x90 | x, (y << 0x4),           // check skip
            0x60 , test,                    // set v0 to test
            0xFF, 0x55,                     // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_9xy0_not_equal(){
    for _ in 0..1000 {
        let x = rand::random::<u8>() & 0x0F;

        let mut y = rand::random::<u8>() & 0x0F;
        while y == x {y = rand::random::<u8>() & 0x0F};
        let y = y;

        let x_val = rand::random::<u8>();

        let mut y_val = rand::random::<u8>();
        while y_val == x_val {y_val = rand::random::<u8>()};
        let y_val = y_val;

        let mut test = rand::random::<u8>();
        while (test == x_val) | (test == y_val) | (test == 0) {
            test = rand::random::<u8>()
        };
        let test = test;

        let memory = [
            0xA3, 0x00,             // set index to 0x300
            0x60 | x, x_val,        // set vx to x_val
            0x60 | y, y_val,        // set vy to y_val
            0x90 | x, (y << 0x4),   // check skip
            0x00 , 0x00,            // nop
            0x60 , test,            // set v0 to test
            0xFF, 0x55,             // set 0x300-0x30F to V[0..F]
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x300), test);
    }
}

#[test]
fn test_annn(){
    for index in 0x206u16..0x1000 {
        let n = (index >> 0x8) as u8;
        let nn = (index & 0xFF) as u8;

        let memory = [
            0x60, 0x55,     // set v0 to 0x55
            0xA0 | n, nn,   // set index to NNN
            0xF0, 0x55,     // set index to v0
        ];

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(index), 0x55);
    }
}

#[test]
fn test_bnnn(){
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {

        let nnn:u16 = rng.gen_range(0x206, 0xEFD);
        let val:u8 = rand::random::<u8>();

        let zn = ((nnn >> 0x8) | 0xB0) as u8;
        let nn = (nnn & 0x0FF) as u8;

        let mut memory:[u8;0xE00] = [0;0xE00];

        memory[0..0x6].copy_from_slice(&[
            0xA3, 0x00,
            0x60, val,
            zn, nn,
        ]);

        let start = (nnn as usize) - 0x200 + (val as usize);
        memory[start..(start+4)].copy_from_slice(&[
            0x65, 0xFF,
            0xFF, 0x55,
        ]);

        let mut bus = new_mock_bus();
        bus.memory.set_range(0x200, &memory);

        let mut processor = processor::Processor::default();
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);
        processor.cycle(&mut bus);

        assert_eq!(bus.memory.read_memory(0x305), 0xFF);
    }
}

#[test]
fn test_cxnn(){
    let mut difference = false;
    for x in 0x0u8..0x10 {
        for nn in 0x0..0x100 {
            let nn = nn as u8;
            let memory = [
                0xA3, 0x00,
                0xC0 | x, nn,
                0xFF, 0x55,
            ];

            let mut bus = new_mock_bus();
            bus.memory.set_range(0x200, &memory);

            let mut processor = processor::Processor::default();
            processor.cycle(&mut bus);
            processor.cycle(&mut bus);
            processor.cycle(&mut bus);

            let value = bus.memory.read_memory(0x300 | x as u16);
            let test  = value & nn;
            assert_eq!(value, test);
            difference = value != nn;
        };
    }
    assert!(difference);
}

#[test]
#[ignore]
fn test_dxyn(){
    assert!(false);
}

#[test]
fn test_ex9e(){
    for key in 0x0u8..0x10 {
        for reg in 0x0u8..0x10 {
            for &pressed in &[true, false] {
                let mut input = MockInput::default();

                let memory = if pressed {
                    input.set(key);
                    [
                        0xA3, 0x00,
                        0x60 | reg, key,
                        0xE0 | reg, 0x9E,
                        0x00, 0x00,
                        0x60, 0x55,
                        0xFF, 0x55,
                    ]
                } else {
                    [
                        0xA3, 0x00,
                        0x60 | reg, key,
                        0xE0 | reg, 0x9E,
                        0x60, 0x55,
                        0xFF, 0x55,
                        0x00, 0x00,
                    ]
                };

                let mut bus = bus::Bus::new(
                    MockAudio::default(),
                    MockDisplay::default(),
                    input);

                bus.memory.set_range(0x200, &memory);

                let mut processor = processor::Processor::default();
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                assert_eq!(bus.memory.read_memory(0x300), 0x55);
            }
        }
    }
}

#[test]
fn test_exa1(){
    for key in 0x0u8..0x10 {
        for reg in 0x0u8..0x10 {
            for &pressed in &[true, false] {
                let mut input = MockInput::default();

                let memory = if pressed {
                    input.set(key);
                    [
                        0xA3, 0x00,
                        0x60 | reg, key,
                        0xE0 | reg, 0xA1,
                        0x60, 0x55,
                        0xFF, 0x55,
                        0x00, 0x00,
                    ]
                } else {
                    [
                        0xA3, 0x00,
                        0x60 | reg, key,
                        0xE0 | reg, 0xA1,
                        0x00, 0x00,
                        0x60, 0x55,
                        0xFF, 0x55,
                    ]
                };

                let mut bus = bus::Bus::new(
                    MockAudio::default(),
                    MockDisplay::default(),
                    input);

                bus.memory.set_range(0x200, &memory);

                let mut processor = processor::Processor::default();
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                processor.cycle(&mut bus);
                assert_eq!(bus.memory.read_memory(0x300), 0x55);
            }
        }
    }
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
}
