extern crate ncurses;

#[derive(Default)]
pub struct Audio{}

impl Audio {
    pub fn beep(&self){
        ncurses::beep();
    }
}
