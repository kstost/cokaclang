use std::cell::RefCell;

thread_local! {
    static OUTPUT_BUFFER: RefCell<String> = RefCell::new(String::new());
}

pub fn capture_write(s: &str) {
    OUTPUT_BUFFER.with(|buf| {
        buf.borrow_mut().push_str(s);
    });
}

pub fn take_output() -> String {
    OUTPUT_BUFFER.with(|buf| {
        let mut b = buf.borrow_mut();
        let out = b.clone();
        b.clear();
        out
    })
}
