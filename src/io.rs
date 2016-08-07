pub trait Audio {
    fn beep(&self);
}

pub enum Pixel {
    On,
    Off,
}

pub trait Display {
    fn set(&self, row:usize, col:usize, state:Pixel) -> Result<(),()>;
    fn refresh(&self);
}

pub trait Input {
    fn get_keys(&self) -> Vec<u8>;
    fn get_key(&self) -> u8;
}
