use crate::gdt;
use crate::mem;
use crate::uart::Uart;
use core::arch::asm;
use core::fmt::Write as _;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

const LEAF_VA: u64 = 0x400000;
const LEAF_SP: u64 = 0x801000;

const LEAF_HELLO: [u8; 12] = [
    0xB0, 0x00, 0xCD, 0x80, 0xB0, 0x00, 0xCD, 0x80, 0xB0, 0x01, 0xCD, 0x80,
];
const LEAF_FAULT: [u8; 2] = [0x0F, 0x0B];

static mut KERNEL_CR3: u64 = 0;
static mut LEAF_CR3: u64 = 0;
static mut CODE_FRAME: u64 = 0;
static mut SEPAL_RSP: u64 = 0;
static mut PHASE: u8 = 0;

#[repr(align(16))]
struct Rsp0Stack([u8; 0x4000]);

static mut RSP0: Rsp0Stack = Rsp0Stack([0; 0x4000]);

fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub fn run() -> ! {
    let mut console = Uart::new();
    let _ = writeln!(console, "sepal:go");
    let parts = (
        mem::zalloc(),
        mem::zalloc(),
        mem::zalloc(),
        mem::zalloc(),
        mem::zalloc(),
        mem::zalloc(),
        mem::zalloc(),
    );
    let (pml4, pdpt, pd, ptc, pts, code, stack) = match parts {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g)) => (a, b, c, d, e, f, g),
        _ => {
            let _ = writeln!(console, "sepal:no-mem");
            halt();
        }
    };
    let cr3: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    }
    let kcr3 = cr3 & !0xFFF;
    unsafe {
        let kp = mem::hhdm_virt(kcr3) as *const u64;
        let lp = mem::hhdm_virt(pml4) as *mut u64;
        let mut i = 256;
        while i < 512 {
            lp.add(i).write_volatile(kp.add(i).read_volatile());
            i += 1;
        }
        lp.add(0).write_volatile(pdpt | 0x07);
        (mem::hhdm_virt(pdpt) as *mut u64).add(0).write_volatile(pd | 0x07);
        let p = mem::hhdm_virt(pd) as *mut u64;
        p.add(2).write_volatile(ptc | 0x07);
        p.add(4).write_volatile(pts | 0x07);
        (mem::hhdm_virt(ptc) as *mut u64).add(0).write_volatile(code | 0x07);
        (mem::hhdm_virt(pts) as *mut u64).add(0).write_volatile(stack | 0x07);
        addr_of_mut!(KERNEL_CR3).write_volatile(kcr3);
        addr_of_mut!(LEAF_CR3).write_volatile(pml4);
        addr_of_mut!(CODE_FRAME).write_volatile(code);
        gdt::set_rsp0(addr_of!(RSP0) as u64 + 0x4000);
    }
    let _ = writeln!(console, "sepal:tbl {:x} {:x} {:x}", pml4, code, stack);
    enter(&LEAF_HELLO);
}

fn enter(blob: &[u8]) -> ! {
    unsafe {
        let frame = addr_of!(CODE_FRAME).read_volatile();
        let dst = mem::hhdm_virt(frame) as *mut u8;
        core::ptr::copy_nonoverlapping(blob.as_ptr(), dst, blob.len());
        let rsp_now: u64;
        asm!("mov {}, rsp", out(reg) rsp_now, options(nomem, nostack));
        addr_of_mut!(SEPAL_RSP).write_volatile(rsp_now);
        let lcr3 = addr_of!(LEAF_CR3).read_volatile();
        asm!("mov cr3, {0}", in(reg) lcr3, options(nostack));
        crate::uart::puts(b"sepal:iret\n");
        asm!(
            "mov rax, 0x33",
            "push rax",
            "mov rax, 0x801000",
            "push rax",
            "mov rax, 2",
            "push rax",
            "mov rax, 0x2b",
            "push rax",
            "mov rax, 0x400000",
            "push rax",
            "iretq",
            options(noreturn)
        );
    }
}

static mut SYS_N: u32 = 0;

#[no_mangle]
pub extern "C" fn syscall_probe(cs: u64) -> u8 {
    let n = unsafe {
        let p = addr_of_mut!(SYS_N);
        p.write_volatile(p.read_volatile() + 1);
        p.read_volatile()
    };
    if n == 1 {
        crate::uart::puts(b"cs:");
        crate::uart::hex8(cs as u8);
        crate::uart::put(b'\n');
    }
    if n >= 3 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn sepal_abandon(v: usize) -> ! {
    let mut console = Uart::new();
    if v == 0 {
        let _ = writeln!(console, "leaf:done");
    } else {
        let _ = writeln!(console, "sepal:caught:{v:02x}");
    }
    unsafe {
        let p = addr_of_mut!(PHASE);
        p.write_volatile(p.read_volatile() + 1);
        let kcr3 = addr_of!(KERNEL_CR3).read_volatile();
        asm!("mov cr3, {0}", in(reg) kcr3, options(nostack));
        let rsp = addr_of!(SEPAL_RSP).read_volatile();
        asm!(
            "mov rsp, {0}",
            "jmp {1}",
            in(reg) rsp,
            sym sepal_resume,
            options(noreturn)
        );
    }
}

extern "C" fn sepal_resume() -> ! {
    let phase = unsafe { addr_of!(PHASE).read_volatile() };
    if phase == 1 {
        enter(&LEAF_FAULT);
    }
    let mut console = Uart::new();
    let _ = writeln!(console, "sepal:alive");
    crate::hyp::main_phase();
}
