const RAM_SIZE:usize = 0x1000;

const C8_FONT:[u8;0x50] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Memory {
    memory:[u8;RAM_SIZE],
}

impl Default for Memory {
    fn default() -> Memory {
        let mut memory = [0x0;RAM_SIZE];
        memory[..0x50].copy_from_slice(&C8_FONT);
        Memory{memory:memory}
    }
}

impl Memory {
    pub fn read_memory(&self, pointer:u16) -> u8{
        self.memory[pointer as usize]
    }
    pub fn write_memory(&mut self, pointer:u16, value:u8){
        self.memory[pointer as usize] = value
    }
    pub fn set_range(&mut self, pointer:u16, values:&[u8]){
        let pointer = pointer as usize;
        self.memory[pointer..(pointer + values.len())].copy_from_slice(values)
    }
}
