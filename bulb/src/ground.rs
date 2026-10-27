use crate::mem;
use crate::uart::Uart;
use core::arch::asm;
use core::fmt::Write as _;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use soil::lapic;
use soil::pit;

const MSR_APIC_BASE: u32 = 0x1B;
const APIC_ENABLE: u64 = 1 << 11;
const X2_ENABLE: u64 = 1 << 10;
const TICK_VECTOR: u8 = 0x20;
const SPURIOUS_VECTOR: u8 = 0x2F;
const TICKS_GOAL: u32 = 3;
const SPIN_BUDGET: u32 = 200_000_000;

static mut MMIO: u64 = 0;
static mut X2: bool = false;
static mut TICKS: u32 = 0;
static mut APIC_ON: bool = false;
static mut SLEEP_OK: bool = false;

pub const HZ: u64 = 128;

#[no_mangle]
pub extern "C" fn ground_irq(v: usize) {
    if v == usize::from(TICK_VECTOR) {
        tick();
    }
}

fn tick() {
    unsafe {
        let p = addr_of_mut!(TICKS);
        p.write_volatile(p.read_volatile() + 1);
    }
    lapic_write(lapic::EOI, 0);
}

pub fn init(console: &mut Uart) {
    if !has_apic() {
        let _ = writeln!(console, "soil:apic:0");
        return;
    }
    let mut base = rdmsr(MSR_APIC_BASE);
    if base & APIC_ENABLE == 0 {
        base |= APIC_ENABLE;
        wrmsr(MSR_APIC_BASE, base);
    }
    unsafe {
        addr_of_mut!(X2).write_volatile(base & X2_ENABLE != 0);
        addr_of_mut!(MMIO).write_volatile(mem::hhdm_virt(base & 0xFFFF_F000));
        addr_of_mut!(APIC_ON).write_volatile(true);
    }
    pic_mask();
    lapic_write(lapic::SVR, lapic::svr(SPURIOUS_VECTOR));
    let _ = writeln!(console, "soil:apic:1");
    let reload = calibrate();
    lapic_write(lapic::TIMER_DCR, lapic::DIV_16);
    lapic_write(lapic::TIMER_LVT, lapic::lvt_periodic(TICK_VECTOR));
    lapic_write(lapic::TIMER_ICR, reload);
    unsafe { asm!("sti", options(nostack)) };
    let got = wait_ticks(TICKS_GOAL);
    unsafe { asm!("cli", options(nostack)) };
    lapic_write(lapic::TIMER_LVT, lapic::lvt_masked(TICK_VECTOR));
    let n = unsafe { addr_of!(TICKS).read_volatile() };
    let _ = writeln!(console, "soil:tick:{}", n);
    if got {
        let _ = writeln!(console, "soil:timer:1");
    }
}

pub fn apic_on() -> bool {
    unsafe { addr_of!(APIC_ON).read_volatile() }
}

pub fn x2() -> bool {
    unsafe { addr_of!(X2).read_volatile() }
}

pub fn ticks() -> u32 {
    unsafe { addr_of!(TICKS).read_volatile() }
}

pub fn serve() {
    if !apic_on() {
        return;
    }
    lapic_write(lapic::TIMER_LVT, lapic::lvt_periodic(TICK_VECTOR));
    unsafe {
        addr_of_mut!(SLEEP_OK).write_volatile(true);
        asm!("sti", options(nostack));
    }
}

pub fn sleep_ok() -> bool {
    unsafe { addr_of!(SLEEP_OK).read_volatile() }
}

fn has_apic() -> bool {
    let d: u32;
    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            inlateout("eax") 1u32 => _,
            out("ecx") _,
            lateout("edx") d,
            options(nomem)
        );
    }
    d & (1 << 9) != 0
}

fn rdmsr(idx: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!("rdmsr", in("ecx") idx, out("eax") lo, out("edx") hi, options(nomem, nostack));
    }
    ((hi as u64) << 32) | lo as u64
}

fn wrmsr(idx: u32, v: u64) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") idx,
            in("eax") v as u32,
            in("edx") (v >> 32) as u32,
            options(nomem, nostack)
        );
    }
}

fn outb(port: u16, v: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") v, options(nomem, nostack, preserves_flags));
    }
}

fn inb(port: u16) -> u8 {
    let v: u8;
    unsafe {
        asm!("in al, dx", out("al") v, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    v
}

fn pic_mask() {
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);
}

fn lapic_read(off: u32) -> u32 {
    unsafe {
        if addr_of!(X2).read_volatile() {
            rdmsr(lapic::x2_msr(off)) as u32
        } else {
            let p = (addr_of!(MMIO).read_volatile() + u64::from(off)) as *mut u32;
            p.read_volatile()
        }
    }
}

fn lapic_write(off: u32, v: u32) {
    unsafe {
        if addr_of!(X2).read_volatile() {
            wrmsr(lapic::x2_msr(off), u64::from(v));
        } else {
            let p = (addr_of!(MMIO).read_volatile() + u64::from(off)) as *mut u32;
            p.write_volatile(v);
        }
    }
}

fn calibrate() -> u32 {
    let c = pit::counts(10);
    outb(0x43, 0x30);
    outb(0x40, c as u8);
    outb(0x40, (c >> 8) as u8);
    lapic_write(lapic::TIMER_DCR, lapic::DIV_16);
    lapic_write(lapic::TIMER_LVT, lapic::lvt_masked(TICK_VECTOR));
    lapic_write(lapic::TIMER_ICR, 0xFFFF_FFFF);
    pit_wait();
    let ccr = lapic_read(lapic::TIMER_CCR);
    let elapsed = u64::from(0xFFFF_FFFFu32.wrapping_sub(ccr));
    lapic::reload(elapsed, HZ)
}

fn pit_wait() {
    let mut i = 0;
    while i < 100_000_000 {
        outb(0x43, 0x00);
        let lo = u16::from(inb(0x40));
        let hi = u16::from(inb(0x40));
        if lo | (hi << 8) == 0 {
            return;
        }
        i += 1;
    }
}

fn wait_ticks(goal: u32) -> bool {
    let mut i = 0;
    while i < SPIN_BUDGET {
        core::hint::spin_loop();
        if unsafe { addr_of!(TICKS).read_volatile() } >= goal {
            return true;
        }
        i += 1;
    }
    false
}
