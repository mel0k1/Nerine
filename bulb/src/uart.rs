use core::arch::asm;
use core::fmt::{Result, Write};

const COM1: u16 = 0x3F8;
const EXIT_PORT: u16 = 0xF4;
const LSR_THRE: u8 = 0x20;

fn outb(port: u16, byte: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") byte, options(nomem, nostack, preserves_flags));
    }
}

fn inb(port: u16) -> u8 {
    let byte: u8;
    unsafe {
        asm!("in al, dx", out("al") byte, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    byte
}

pub fn put(byte: u8) {
    while inb(COM1 + 5) & LSR_THRE == 0 {}
    outb(COM1, byte);
}

pub fn puts(s: &[u8]) {
    for b in s {
        if *b == b'\n' {
            put(b'\r');
        }
        put(*b);
    }
}

fn nib(v: u8) {
    put(if v < 10 { b'0' + v } else { b'a' + v - 10 });
}

pub fn hex8(v: u8) {
    nib(v >> 4);
    nib(v & 0xF);
}

pub fn hex32(v: u32) {
    hex8((v >> 24) as u8);
    hex8((v >> 16) as u8);
    hex8((v >> 8) as u8);
    hex8(v as u8);
}

pub fn hex64(v: u64) {
    hex32((v >> 32) as u32);
    hex32(v as u32);
}

pub fn exit(code: u8) {
    outb(EXIT_PORT, code);
}

pub struct Uart;

impl Uart {
    pub fn new() -> Self {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
        Uart
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                put(b'\r');
            }
            put(byte);
        }
        Ok(())
    }
}
