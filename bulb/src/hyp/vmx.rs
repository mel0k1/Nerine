use crate::gdt;
use crate::idt;
use crate::mem;
use crate::uart::Uart;
use core::arch::{asm, global_asm};
use core::fmt::Write as _;
use core::ptr::addr_of;

const MSR_FEATURE_CONTROL: u32 = 0x3A;
const MSR_VMX_BASIC: u32 = 0x480;
const MSR_VMX_PIN: u32 = 0x481;
const MSR_VMX_PROC: u32 = 0x482;
const MSR_VMX_EXIT: u32 = 0x483;
const MSR_VMX_ENTRY: u32 = 0x484;
const MSR_VMX_CR0_FIXED0: u32 = 0x486;
const MSR_VMX_CR0_FIXED1: u32 = 0x487;
const MSR_VMX_CR4_FIXED0: u32 = 0x488;
const MSR_VMX_CR4_FIXED1: u32 = 0x489;
const MSR_VMX_TRUE_PIN: u32 = 0x48D;
const MSR_VMX_TRUE_PROC: u32 = 0x48E;
const MSR_VMX_TRUE_EXIT: u32 = 0x48F;
const MSR_VMX_TRUE_ENTRY: u32 = 0x490;

const FLD_GUEST_SEL_ES: u64 = 0x0800;
const FLD_GUEST_SEL_CS: u64 = 0x0802;
const FLD_GUEST_SEL_SS: u64 = 0x0804;
const FLD_GUEST_SEL_DS: u64 = 0x0806;
const FLD_GUEST_SEL_FS: u64 = 0x0808;
const FLD_GUEST_SEL_GS: u64 = 0x080A;
const FLD_GUEST_SEL_LDTR: u64 = 0x080C;
const FLD_GUEST_SEL_TR: u64 = 0x080E;
const FLD_HOST_SEL_ES: u64 = 0x0C00;
const FLD_HOST_SEL_CS: u64 = 0x0C02;
const FLD_HOST_SEL_SS: u64 = 0x0C04;
const FLD_HOST_SEL_DS: u64 = 0x0C06;
const FLD_HOST_SEL_FS: u64 = 0x0C08;
const FLD_HOST_SEL_GS: u64 = 0x0C0A;
const FLD_HOST_SEL_TR: u64 = 0x0C0E;
const FLD_VMCS_LINK: u64 = 0x2800;
const FLD_GUEST_DEBUGCTL: u64 = 0x2802;
const FLD_GUEST_EFER: u64 = 0x2806;
const FLD_PIN: u64 = 0x4000;
const FLD_PROC: u64 = 0x4002;
const FLD_EXC_BITMAP: u64 = 0x4004;
const FLD_CR3_TARGET_COUNT: u64 = 0x400A;
const FLD_EXIT_CTLS: u64 = 0x400C;
const FLD_ENTRY_CTLS: u64 = 0x4012;
const FLD_EXIT_REASON: u64 = 0x4402;
const FLD_EXIT_QUAL: u64 = 0x6400;
const FLD_LIMIT_ES: u64 = 0x4800;
const FLD_LIMIT_CS: u64 = 0x4802;
const FLD_LIMIT_SS: u64 = 0x4804;
const FLD_LIMIT_DS: u64 = 0x4806;
const FLD_LIMIT_FS: u64 = 0x4808;
const FLD_LIMIT_GS: u64 = 0x480A;
const FLD_LIMIT_LDTR: u64 = 0x480C;
const FLD_LIMIT_TR: u64 = 0x480E;
const FLD_LIMIT_GDTR: u64 = 0x4810;
const FLD_LIMIT_IDTR: u64 = 0x4812;
const FLD_AR_ES: u64 = 0x4814;
const FLD_AR_CS: u64 = 0x4816;
const FLD_AR_SS: u64 = 0x4818;
const FLD_AR_DS: u64 = 0x481A;
const FLD_AR_FS: u64 = 0x481C;
const FLD_AR_GS: u64 = 0x481E;
const FLD_AR_LDTR: u64 = 0x4820;
const FLD_AR_TR: u64 = 0x4822;
const FLD_GUEST_INTERRUPTIBILITY: u64 = 0x4824;
const FLD_GUEST_ACTIVITY: u64 = 0x4826;
const FLD_GUEST_SYSENTER_CS: u64 = 0x482A;
const FLD_HOST_SYSENTER_CS: u64 = 0x4C00;
const FLD_CR0_MASK: u64 = 0x6000;
const FLD_CR4_MASK: u64 = 0x6002;
const FLD_CR0_SHADOW: u64 = 0x6004;
const FLD_CR4_SHADOW: u64 = 0x6006;
const FLD_GUEST_CR0: u64 = 0x6800;
const FLD_GUEST_CR3: u64 = 0x6802;
const FLD_GUEST_CR4: u64 = 0x6804;
const FLD_GUEST_BASE_ES: u64 = 0x6806;
const FLD_GUEST_BASE_CS: u64 = 0x6808;
const FLD_GUEST_BASE_SS: u64 = 0x680A;
const FLD_GUEST_BASE_DS: u64 = 0x680C;
const FLD_GUEST_BASE_FS: u64 = 0x680E;
const FLD_GUEST_BASE_GS: u64 = 0x6810;
const FLD_GUEST_BASE_LDTR: u64 = 0x6812;
const FLD_GUEST_BASE_TR: u64 = 0x6814;
const FLD_GUEST_BASE_GDTR: u64 = 0x6816;
const FLD_GUEST_BASE_IDTR: u64 = 0x6818;
const FLD_GUEST_DR7: u64 = 0x681A;
const FLD_GUEST_RSP: u64 = 0x681C;
const FLD_GUEST_RIP: u64 = 0x681E;
const FLD_GUEST_RFLAGS: u64 = 0x6820;
const FLD_GUEST_PENDING_DBG: u64 = 0x6822;
const FLD_GUEST_SYSENTER_ESP: u64 = 0x6824;
const FLD_GUEST_SYSENTER_EIP: u64 = 0x6826;
const FLD_HOST_CR0: u64 = 0x6C00;
const FLD_HOST_CR3: u64 = 0x6C02;
const FLD_HOST_CR4: u64 = 0x6C04;
const FLD_HOST_BASE_FS: u64 = 0x6C06;
const FLD_HOST_BASE_GS: u64 = 0x6C08;
const FLD_HOST_BASE_TR: u64 = 0x6C0A;
const FLD_HOST_BASE_GDTR: u64 = 0x6C0C;
const FLD_HOST_BASE_IDTR: u64 = 0x6C0E;
const FLD_HOST_SYSENTER_ESP: u64 = 0x6C10;
const FLD_HOST_SYSENTER_EIP: u64 = 0x6C12;
const FLD_HOST_RSP: u64 = 0x6C14;
const FLD_HOST_RIP: u64 = 0x6C16;

const HLT_EXITING: u32 = 1 << 7;
const IO_EXITING: u32 = 1 << 24;
const HOST_IA32E: u32 = 1 << 9;
const ENTRY_IA32E: u32 = 1 << 9;
const ENTRY_LOAD_DEBUG: u32 = 1 << 2;
const ENTRY_LOAD_EFER: u32 = 1 << 15;

const GUEST_CODE: [u8; 7] = [0xB0, 0x47, 0xBA, 0xF8, 0x03, 0xEE, 0xF4];

#[repr(align(16))]
struct HostStack([u8; 0x8000]);

static mut HOST_STACK: HostStack = HostStack([0; 0x8000]);

extern "C" {
    fn vmx_exit_stub();
}

global_asm!(
    ".intel_syntax noprefix",
    ".text",
    ".globl vmx_exit_stub",
    "vmx_exit_stub:",
    "push rax",
    "push rbx",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push rbp",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov rdi, rsp",
    "call vmx_exit_rust",
    "cmp al, 0",
    "jne 2f",
    "add rsp, 120",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rbp",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rbx",
    "pop rax",
    "vmresume",
    "setna al",
    "test al, al",
    "jnz 3f",
    "ud2",
    "2:",
    "add rsp, 120",
    "vmxoff",
    "jmp nerine_vm_terminated",
    "3:",
    "add rsp, 120",
    "jmp nerine_vm_terminated",
);

#[repr(C)]
struct Regs {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

#[no_mangle]
pub extern "C" fn vmx_exit_rust(regs: *const Regs) -> u8 {
    let reason = vmread(FLD_EXIT_REASON) as u32;
    match reason & 0xFFFF {
        3 => {
            let q = vmread(FLD_EXIT_QUAL) as u32;
            let port = (q >> 16) & 0xFFFF;
            let is_in = q & 0x8 != 0;
            let size = q & 0x7;
            if port == 0x3F8 && !is_in && size == 0 {
                let b = unsafe { (*regs).rax } as u8;
                crate::uart::put(b);
                0
            } else {
                crate::uart::puts(b"io:");
                crate::uart::hex32(port);
                crate::uart::put(b'\n');
                1
            }
        }
        5 => 1,
        r => {
            crate::uart::puts(b"exit:");
            crate::uart::hex32(r);
            crate::uart::put(b'\n');
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn nerine_vm_terminated() -> ! {
    crate::uart::puts(b"guest:done\n");
    crate::uart::exit(0x1F);
    loop {
        unsafe { asm!("hlt") }
    }
}

pub fn supported() -> bool {
    let c: u32;
    unsafe {
        asm!(
            "cpuid",
            inlateout("eax") 1u32 => _,
            out("ebx") _,
            lateout("ecx") c,
            out("edx") _,
            options(nomem, nostack)
        );
    }
    c & (1 << 5) != 0
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

fn cr_read(n: u32) -> u64 {
    let v: u64;
    unsafe {
        match n {
            0 => asm!("mov {}, cr0", out(reg) v, options(nomem, nostack)),
            3 => asm!("mov {}, cr3", out(reg) v, options(nomem, nostack)),
            _ => asm!("mov {}, cr4", out(reg) v, options(nomem, nostack)),
        }
    }
    v
}

fn cr_write(n: u32, v: u64) {
    unsafe {
        match n {
            0 => asm!("mov cr0, {}", in(reg) v, options(nostack)),
            _ => asm!("mov cr4, {}", in(reg) v, options(nostack)),
        }
    }
}

fn ok(f: u8) -> Result<(), ()> {
    if f == 0 {
        Ok(())
    } else {
        Err(())
    }
}

fn vmxon(phys: u64) -> Result<(), ()> {
    let f: u8;
    unsafe {
        asm!("vmxon [{0}]", "setna {1}", in(reg) phys, out(reg_byte) f);
    }
    ok(f)
}

fn vmclear(phys: u64) -> Result<(), ()> {
    let f: u8;
    unsafe {
        asm!("vmclear [{0}]", "setna {1}", in(reg) phys, out(reg_byte) f);
    }
    ok(f)
}

fn vmptrld(phys: u64) -> Result<(), ()> {
    let f: u8;
    unsafe {
        asm!("vmptrld [{0}]", "setna {1}", in(reg) phys, out(reg_byte) f);
    }
    ok(f)
}

fn vmxoff() {
    unsafe {
        asm!("vmxoff");
    }
}

fn vmlaunch() -> Result<(), ()> {
    let f: u8;
    unsafe {
        asm!("vmlaunch", "setna {0}", out(reg_byte) f);
    }
    ok(f)
}

fn vmwrite(field: u64, value: u64) -> Result<(), ()> {
    let f: u8;
    unsafe {
        asm!("vmwrite {0}, {1}", "setna {2}", in(reg) field, in(reg) value, out(reg_byte) f);
    }
    ok(f)
}

fn vmread(field: u64) -> u64 {
    let v: u64;
    unsafe {
        asm!("vmread {0}, {1}", out(reg) v, in(reg) field);
    }
    v
}

macro_rules! w {
    ($f:expr, $v:expr) => {
        vmwrite($f, $v as u64)?;
    };
}

fn ctl(msr: u32, desired: u32) -> u32 {
    let v = rdmsr(msr);
    let must = v as u32;
    let may = (v >> 32) as u32;
    must | (desired & may)
}

fn host_rsp() -> u64 {
    (addr_of!(HOST_STACK) as u64) + 0x8000
}

fn build_guest() -> Result<(u64, u64, u64, u64, u64), ()> {
    let code = mem::zalloc().ok_or(())?;
    let stack = mem::zalloc().ok_or(())?;
    let pml4 = mem::zalloc().ok_or(())?;
    let pdpt = mem::zalloc().ok_or(())?;
    let gdt = mem::zalloc().ok_or(())?;
    let tss = mem::zalloc().ok_or(())?;
    unsafe {
        (mem::hhdm_virt(code) as *mut u8)
            .copy_from_nonoverlapping(GUEST_CODE.as_ptr(), GUEST_CODE.len());
        (mem::hhdm_virt(pml4) as *mut u64).write_volatile(pdpt | 0x3);
        (mem::hhdm_virt(pdpt) as *mut u64).write_volatile(0x83);
        let g = mem::hhdm_virt(gdt) as *mut u64;
        g.write_volatile(0);
        g.add(1).write_volatile(0x00AF_9B00_0000_FFFF);
        g.add(2).write_volatile(0x00CF_9300_0000_FFFF);
        g.add(3).write_volatile(
            0x67
                | ((tss & 0xFFFF) << 16)
                | (((tss >> 16) & 0xFF) << 32)
                | (0x89u64 << 40)
                | (((tss >> 24) & 0xFF) << 56),
        );
        g.add(4).write_volatile(tss >> 32);
    }
    Ok((code, pml4, gdt, tss, stack + 0x1000))
}

fn build_vmcs(
    vmcs_r: u64,
    host_cr0: u64,
    host_cr4: u64,
    guest_cr0: u64,
    guest_cr4: u64,
    true_ctls: bool,
) -> Result<(), ()> {
    vmclear(vmcs_r)?;
    vmptrld(vmcs_r)?;
    let pin = ctl(
        if true_ctls { MSR_VMX_TRUE_PIN } else { MSR_VMX_PIN },
        0,
    );
    let proc = ctl(
        if true_ctls { MSR_VMX_TRUE_PROC } else { MSR_VMX_PROC },
        HLT_EXITING | IO_EXITING,
    );
    let exit_c = ctl(
        if true_ctls { MSR_VMX_TRUE_EXIT } else { MSR_VMX_EXIT },
        HOST_IA32E,
    );
    let entry_c = ctl(
        if true_ctls { MSR_VMX_TRUE_ENTRY } else { MSR_VMX_ENTRY },
        ENTRY_IA32E | ENTRY_LOAD_DEBUG | ENTRY_LOAD_EFER,
    );
    let (rip, cr3, gdt_base, tss_base, rsp) = build_guest()?;
    w!(FLD_PIN, pin as u64);
    w!(FLD_PROC, proc as u64);
    w!(FLD_EXIT_CTLS, exit_c as u64);
    w!(FLD_ENTRY_CTLS, entry_c as u64);
    w!(FLD_EXC_BITMAP, 0u64);
    w!(FLD_CR3_TARGET_COUNT, 0u64);
    w!(FLD_CR0_MASK, 0u64);
    w!(FLD_CR4_MASK, 0u64);
    w!(FLD_CR0_SHADOW, guest_cr0);
    w!(FLD_CR4_SHADOW, guest_cr4);
    w!(FLD_VMCS_LINK, u64::MAX);
    w!(FLD_HOST_CR0, host_cr0);
    w!(FLD_HOST_CR3, cr_read(3));
    w!(FLD_HOST_CR4, host_cr4);
    w!(FLD_HOST_SEL_CS, 0x08u64);
    w!(FLD_HOST_SEL_SS, 0x10u64);
    w!(FLD_HOST_SEL_DS, 0x10u64);
    w!(FLD_HOST_SEL_ES, 0x10u64);
    w!(FLD_HOST_SEL_FS, 0u64);
    w!(FLD_HOST_SEL_GS, 0u64);
    w!(FLD_HOST_SEL_TR, 0x18u64);
    w!(FLD_HOST_BASE_FS, 0u64);
    w!(FLD_HOST_BASE_GS, 0u64);
    w!(FLD_HOST_BASE_TR, gdt::tss_base());
    w!(FLD_HOST_BASE_GDTR, gdt::gdt_base());
    w!(FLD_HOST_BASE_IDTR, idt::idt_base());
    w!(FLD_HOST_SYSENTER_CS, 0u64);
    w!(FLD_HOST_SYSENTER_ESP, 0u64);
    w!(FLD_HOST_SYSENTER_EIP, 0u64);
    w!(FLD_HOST_RSP, host_rsp());
    w!(FLD_HOST_RIP, vmx_exit_stub as usize as u64);
    w!(FLD_GUEST_CR0, guest_cr0);
    w!(FLD_GUEST_CR3, cr3);
    w!(FLD_GUEST_CR4, guest_cr4);
    w!(FLD_GUEST_SEL_CS, 0x08u64);
    w!(FLD_GUEST_SEL_SS, 0x10u64);
    w!(FLD_GUEST_SEL_DS, 0x10u64);
    w!(FLD_GUEST_SEL_ES, 0x10u64);
    w!(FLD_GUEST_SEL_FS, 0u64);
    w!(FLD_GUEST_SEL_GS, 0u64);
    w!(FLD_GUEST_SEL_LDTR, 0u64);
    w!(FLD_GUEST_SEL_TR, 0x18u64);
    w!(FLD_LIMIT_CS, 0xFFFF_FFFFu64);
    w!(FLD_LIMIT_SS, 0xFFFFFu64);
    w!(FLD_LIMIT_DS, 0xFFFFFu64);
    w!(FLD_LIMIT_ES, 0xFFFFFu64);
    w!(FLD_LIMIT_FS, 0u64);
    w!(FLD_LIMIT_GS, 0u64);
    w!(FLD_LIMIT_LDTR, 0u64);
    w!(FLD_LIMIT_TR, 0x67u64);
    w!(FLD_LIMIT_GDTR, 0x27u64);
    w!(FLD_LIMIT_IDTR, 0xFFFFu64);
    w!(FLD_AR_CS, 0xA09Bu64);
    w!(FLD_AR_SS, 0x93u64);
    w!(FLD_AR_DS, 0x93u64);
    w!(FLD_AR_ES, 0x93u64);
    w!(FLD_AR_FS, 0x10000u64);
    w!(FLD_AR_GS, 0x10000u64);
    w!(FLD_AR_LDTR, 0x10000u64);
    w!(FLD_AR_TR, 0x8Bu64);
    w!(FLD_GUEST_BASE_CS, 0u64);
    w!(FLD_GUEST_BASE_SS, 0u64);
    w!(FLD_GUEST_BASE_DS, 0u64);
    w!(FLD_GUEST_BASE_ES, 0u64);
    w!(FLD_GUEST_BASE_FS, 0u64);
    w!(FLD_GUEST_BASE_GS, 0u64);
    w!(FLD_GUEST_BASE_LDTR, 0u64);
    w!(FLD_GUEST_BASE_TR, tss_base);
    w!(FLD_GUEST_BASE_GDTR, gdt_base);
    w!(FLD_GUEST_BASE_IDTR, 0u64);
    w!(FLD_GUEST_DEBUGCTL, 0u64);
    w!(FLD_GUEST_DR7, 0x400u64);
    w!(FLD_GUEST_RFLAGS, 2u64);
    w!(FLD_GUEST_RSP, rsp);
    w!(FLD_GUEST_RIP, rip);
    w!(FLD_GUEST_INTERRUPTIBILITY, 0u64);
    w!(FLD_GUEST_ACTIVITY, 0u64);
    w!(FLD_GUEST_PENDING_DBG, 0u64);
    w!(FLD_GUEST_SYSENTER_CS, 0u64);
    w!(FLD_GUEST_SYSENTER_ESP, 0u64);
    w!(FLD_GUEST_SYSENTER_EIP, 0u64);
    w!(FLD_GUEST_EFER, 0x500u64);
    Ok(())
}

pub fn enter(console: &mut Uart) -> Result<(), ()> {
    let fc = rdmsr(MSR_FEATURE_CONTROL);
    if fc & 1 == 0 {
        wrmsr(MSR_FEATURE_CONTROL, fc | 0x5);
    } else if fc & 4 == 0 {
        let _ = writeln!(console, "vmx:fc-locked");
        return Err(());
    }
    let basic = rdmsr(MSR_VMX_BASIC);
    let revid = (basic & 0x7FFF_FFFF) as u32;
    let true_ctls = basic & (1u64 << 55) != 0;
    let cr0f0 = rdmsr(MSR_VMX_CR0_FIXED0);
    let cr0f1 = rdmsr(MSR_VMX_CR0_FIXED1);
    let cr4f0 = rdmsr(MSR_VMX_CR4_FIXED0);
    let cr4f1 = rdmsr(MSR_VMX_CR4_FIXED1);
    let host_cr0 = (cr_read(0) | cr0f0) & cr0f1;
    let host_cr4 = (cr_read(4) | cr4f0) & cr4f1;
    cr_write(0, host_cr0);
    cr_write(4, host_cr4);
    let guest_cr0 = (0x8001_0001u64 | cr0f0) & cr0f1;
    let guest_cr4 = (0x2020u64 | cr4f0) & cr4f1;
    let Some(vmxon_r) = mem::zalloc() else {
        return Err(());
    };
    let Some(vmcs_r) = mem::zalloc() else {
        return Err(());
    };
    unsafe {
        (mem::hhdm_virt(vmxon_r) as *mut u32).write_volatile(revid);
        (mem::hhdm_virt(vmcs_r) as *mut u32).write_volatile(revid);
    }
    vmxon(vmxon_r)?;
    let built = build_vmcs(vmcs_r, host_cr0, host_cr4, guest_cr0, guest_cr4, true_ctls);
    if built.is_err() {
        vmxoff();
        return Err(());
    }
    match vmlaunch() {
        Ok(()) => Ok(()),
        Err(()) => {
            vmxoff();
            let _ = writeln!(console, "vmx:launch-fail");
            Err(())
        }
    }
}
