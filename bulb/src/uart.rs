use core::arch::asm;
use core::fmt::{Result, Write};

const COM1: u16 = 0x3F8;
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

    fn send(&self, byte: u8) {
        while inb(COM1 + 5) & LSR_THRE == 0 {}
        outb(COM1, byte);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send(b'\r');
            }
            self.send(byte);
        }
        Ok(())
    }
}
