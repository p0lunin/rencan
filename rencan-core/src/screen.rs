#[repr(transparent)]
#[derive(Debug, Clone, PartialEq)]
pub struct Screen(pub [u32; 2]);

impl Screen {
    pub fn new(width: u32, height: u32) -> Self {
        Screen([width, height])
    }
}

impl Screen {
    pub fn width(&self) -> u32 {
        self.0[0]
    }
    pub fn height(&self) -> u32 {
        self.0[1]
    }
    pub fn size(&self) -> u32 { self.width() * self.height() }
}
