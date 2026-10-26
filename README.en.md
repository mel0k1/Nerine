<div align="center">

<img src="assets/nerine.jpg" alt="Nerine — the flower the system is named after" width="640">

# Nerine

**An operating system that is grown, not built.**

[![kernel](https://github.com/mel0k1/Nerine/actions/workflows/kernel.yml/badge.svg)](https://github.com/mel0k1/Nerine/actions/workflows/kernel.yml)
[![stage](https://img.shields.io/badge/stage-bulb-7ac142)](#roadmap)
[![kernel](https://img.shields.io/badge/kernel-Rust-DEA584?logo=rust)](https://www.rust-lang.org)
[![license](https://img.shields.io/badge/license-Apache--2.0-D22128)](LICENSE)

[Русский](README.md) · **English**

</div>

**Nerine** is a hobby operating system focused on speed, whose kernel is, first of all, **one large hypervisor**. The kernel does not "run programs": it isolates, schedules and watches, while subsystems — drivers, networking, the shell — grow on top as independent guest blooms. A regular application gets no VM of its own: it lives inside a light Sepal domain. A crashed driver is not a kernel panic here; it is a flower to be replanted.

Named after [Nerine](https://en.wikipedia.org/wiki/Nerine) — a South African flower with slender, curled petals that blooms into the autumn dark.

## Architecture

```
┌───────────────────────────────────────────────────┐
│                     Nerine OS                     │
│                                                   │
│      ┌───────┐      ┌───────┐      ┌───────┐      │
│      │ Leaf  │      │ Leaf  │      │ Leaf  │      │
│      │ Sepal │      │ Sepal │      │ Sepal │      │
│      └───┬───┘      └───┬───┘      └───┬───┘      │
│          ┘──────────────┼──────────────┘          │
│   ┌───────┐   ┌───────┐ │ ┌───────┐   ┌───────┐   │
│   │ Bloom │   │ Bloom │ │ │ Bloom │   │ Bloom │   │
│   │  net  │   │  fs   │ │ │  gpu  │   │ shell │   │
│   └───┬───┘   └───┬───┘ │ └───┬───┘   └───┬───┘   │
│       ┘───────────┴─────┼─────┴───────────┘       │
│       Nectar · IPC          Pollen · bulk         │
│   ┌───────────────────────────────────────────┐   │
│   │               Bulb · kernel               │   │
│   │    scheduling · memory · VT-x / SVM       │   │
│   └───────────────────────────────────────────┘   │
│                  Tunic · security                 │
│                 Hardware · x86_64                 │
└───────────────────────────────────────────────────┘
```

## The garden of names (approved)

| Component | Name | Image |
|---|---|---|
| Operating system | **Nerine** | the flower everything grows from |
| Kernel / hypervisor | **Bulb** | the bulb that feeds and shelters the plant |
| Guest subsystem (VM) | **Bloom** | one isolated bloom on the bulb |
| Application | **Leaf** | a light process without a VM of its own |
| Protection domain | **Sepal** | the casing that shelters a Leaf |
| Message IPC | **Nectar** | what blooms exchange |
| Bulk transfer | **Pollen** | zero-copy data carried by the wind |
| Shell | **Corolla** | the part you actually touch |
| Security layer | **Tunic** | the dry skin of the bulb |
| Hard isolation mode | **Greenhouse** | Qubes-style isolation, opt-in |

## Lineage

The design is cross-checked against real hypervisors and microkernels:

- **Xen** — the domain model and the road to a minimal dom0 (dom0less)
- **Jailhouse** — static partitioning of resources between guests
- **NOVA** — the microhypervisor: a compact TCB as a goal in itself
- **seL4** — formal verification as the benchmark of trust
- **managarm** — the Thor microkernel and async Hel: lanes, queues and kernlets as the fast path; VirtualizedSpace / VirtualizedCpu — virtualization inside a microkernel; the bragi IDL on top
- **KVM / Firecracker** — the Linux host and a Rust VMM as industrial references

## Principles

- **Rust first** — the kernel is `#![no_std]` Rust; C++ only where it has earned its place.
- **Code over commentary** — comments answer "why", they never re-narrate "what".
- **Isolation by default** — if it can be a guest, it will be a guest.
- **Fast by default** — a Leaf lives in a light Sepal domain, not a VM; bulk data flows through Pollen without a single copy.
- **A garden, not a monolith** — subsystems are planted, replaced and pruned independently.

## Roadmap

- [x] **Seedling** — a freestanding `no_std` kernel that builds for x86_64
- [x] **Stem** — a custom `x86_64-nerine.json` target, 16550 UART output
- [x] **Nectar** — lanes and queues in the spirit of Hel: an SPSC lane, a shared-memory queue with a doorbell, spinlocks — all unit-tested in CI
- [x] **Bulb** — boot via Limine, VT-x: a real VMX path and the first guest; in CI the guest runs through the shadow backend under TCG
- [x] **Pollen** — zero-copy bulk channels: a shared-memory ring of grains (grant → publish → poll → release, not a single copy), a doorbell and a petal — the in-kernel fast path, inspired by kernlets ← *we are here*
- [ ] **First blooms** — drivers as guest Blooms
- [ ] **First leaves** — Leaf processes inside Sepal domains
- [ ] **Corolla** — an interactive shell
- [ ] **Greenhouse** — opt-in Qubes-style hard isolation
- [ ] **Perennial** — a self-hosting toolchain

## Building

```sh
cargo build
```

The nightly toolchain and `rust-src` come automatically via `rust-toolchain.toml`; the build target is `x86_64-nerine.json`. The result is `target/x86_64-nerine/debug/bulb` — a kernel ELF image.

## Running

The kernel is booted by the [Limine](https://github.com/Limine-Bootloader/Limine) bootloader and prints to COM1:

```sh
cargo build

mkdir -p isoroot/boot
cp target/x86_64-nerine/debug/bulb isoroot/boot/nerine.elf
cp ci/limine.conf isoroot/
cp limine/limine-binary/* isoroot/
xorriso -as mkisofs -o nerine.iso \
  -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table \
  -eltorito-alt-boot -b limine-uefi-cd.bin -no-emul-boot \
  isoroot

qemu-system-x86_64 -M q35 -cpu max -m 512M -cdrom nerine.iso \
  -display none -serial stdio -no-reboot \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04
```

On a machine with real VT-x the kernel brings up VMX and runs the guest for real; under TCG (no VMX) the same scenario goes through the shadow backend — both paths end with `guest:done` on COM1.

## License

[Apache-2.0](LICENSE)
