use std::cell::RefCell;
use std::rc::Rc;

// I have to found a correct gpui-component way to use booleans ...
#[derive(Clone)]
pub struct BooleanState {
    inner: Rc<RefCell<bool>>,
}

impl BooleanState {
    pub fn new(initial: bool) -> Self {
        Self {
            inner: Rc::new(RefCell::new(initial)),
        }
    }

    pub fn get(&self) -> bool {
        *self.inner.borrow()
    }

    pub fn set(&self, value: bool) {
        let mut inner = self.inner.borrow_mut();
        *inner = value;
    }

    pub fn toggle(&self) {
        let new_val = !self.get();
        self.set(new_val);
    }
}
