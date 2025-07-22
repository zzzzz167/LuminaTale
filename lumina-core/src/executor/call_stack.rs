use crate::executor::frame::Frame;

#[derive(Debug,Default, Clone)]
pub struct CallStack {
    pub stack: Vec<Frame>
}

impl CallStack {
    pub fn push (&mut self, frame: Frame) {
        self.stack.push(frame);
    }
    pub fn pop (&mut self) -> Option<Frame> {
        self.stack.pop()
    }
    pub fn top_mut(&mut self) -> Option<&mut Frame> {
        self.stack.last_mut()
    }
    
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}