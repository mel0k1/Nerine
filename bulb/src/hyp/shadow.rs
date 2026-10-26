use crate::uart;

pub fn run_bloom() {
    uart::puts(b"bloom:ok\n");
}

pub fn run() {
    uart::put(b'G');
    uart::put(b'\n');
}
