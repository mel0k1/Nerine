use core::arch::asm;
use core::arch::x86_64::InterruptStackFrame;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

const ENTRIES: usize = 48;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Entry([u64; 2]);

static mut IDT: [Entry; ENTRIES] = [Entry([0; 2]); ENTRIES];

#[repr(C, align(8))]
struct Idtr {
    limit: u16,
    base: u64,
}

fn fault(v: u64) -> ! {
    crate::uart::puts(b"ex:");
    crate::uart::hex8(v as u8);
    crate::uart::put(b'\n');
    loop {
        unsafe { asm!("hlt") }
    }
}

macro_rules! exc {
    ($n:literal, $name:ident) => {
        extern "x86-interrupt" fn $name(_: InterruptStackFrame) -> ! {
            fault($n)
        }
    };
    ($n:literal, $name:ident, code) => {
        extern "x86-interrupt" fn $name(_: InterruptStackFrame, _: u64) -> ! {
            fault($n)
        }
    };
}

exc!(0, de0);
exc!(1, db1);
exc!(2, nmi2);
exc!(3, bp3);
exc!(4, of4);
exc!(5, br5);
exc!(6, ud6);
exc!(7, nm7);
exc!(8, df8, code);
exc!(9, mf9);
exc!(10, ts10, code);
exc!(11, np11, code);
exc!(12, ss12, code);
exc!(13, gp13, code);
exc!(14, pf14, code);
exc!(15, xf15);
exc!(16, mc16);
exc!(17, ac17, code);
exc!(18, mc18);
exc!(19, xm19);
exc!(20, ve20);
exc!(21, cp21, code);
exc!(22, rsv22);
exc!(23, rsv23);
exc!(24, rsv24);
exc!(25, rsv25);
exc!(26, rsv26);
exc!(27, rsv27);
exc!(28, hv28);
exc!(29, vc29, code);
exc!(30, sx30);
exc!(31, rsv31);

const CODED: [usize; 8] = [8, 10, 11, 12, 13, 14, 17, 21];

pub fn init() {
    let base = addr_of_mut!(IDT);
    for i in 0..ENTRIES {
        let f: usize = if CODED.contains(&i) {
            match i {
                8 => df8 as usize,
                10 => ts10 as usize,
                11 => np11 as usize,
                12 => ss12 as usize,
                13 => gp13 as usize,
                14 => pf14 as usize,
                17 => ac17 as usize,
                _ => cp21 as usize,
            }
        } else {
            match i {
                0 => de0 as usize,
                1 => db1 as usize,
                2 => nmi2 as usize,
                3 => bp3 as usize,
                4 => of4 as usize,
                5 => br5 as usize,
                6 => ud6 as usize,
                7 => nm7 as usize,
                9 => mf9 as usize,
                15 => xf15 as usize,
                16 => mc16 as usize,
                18 => mc18 as usize,
                19 => xm19 as usize,
                20 => ve20 as usize,
                28 => hv28 as usize,
                29 => vc29 as usize,
                30 => sx30 as usize,
                _ => rsv22 as usize,
            }
        };
        let lo = (f as u64 & 0xFFFF)
            | (0x08u64 << 16)
            | (0x8E00u64 << 40)
            | (((f as u64 >> 16) & 0xFFFF) << 48);
        let hi = f as u64 >> 32;
        unsafe {
            (*base.add(i)).0 = [lo, hi];
        }
    }
    let d = Idtr {
        limit: (ENTRIES * 16 - 1) as u16,
        base: base as u64,
    };
    unsafe {
        asm!("lidt [{}]", in(reg) &d, options(nostack));
    }
}

pub fn idt_base() -> u64 {
    addr_of!(IDT) as u64
}
