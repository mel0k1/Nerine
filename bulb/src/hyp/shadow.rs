use crate::uart;

pub fn run() {
    uart::put(b'G');
    uart::put(b'\n');
}
