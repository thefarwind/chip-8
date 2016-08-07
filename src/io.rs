pub const SCREEN_WIDTH:usize  = 0x40;
pub const SCREEN_HEIGHT:usize = 0x20;

pub enum Pixel {
    On,
    Off,
}

pub trait Audio {
    fn beep(&self);
}

pub trait Display {
    fn set(&mut self, row:usize, col:usize, state:Pixel)
            -> Result<(),()>;
    fn refresh(&mut self);
}

pub trait Input {
    fn get_keys(&self) -> Vec<u8>;
    fn get_key(&self) -> u8;
}
