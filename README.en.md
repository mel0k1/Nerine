<div align="center">

<img src="assets/nerine.jpg" alt="Nerine — the flower the system is named after" width="640">

# Nerine

**An operating system that is grown, not built.**

[![kernel](https://github.com/mel0k1/Nerine/actions/workflows/kernel.yml/badge.svg)](https://github.com/mel0k1/Nerine/actions/workflows/kernel.yml)
[![stage](https://img.shields.io/badge/stage-seedling-ff6f9c)](#roadmap)
[![kernel](https://img.shields.io/badge/kernel-Rust-DEA584?logo=rust)](https://www.rust-lang.org)
[![license](https://img.shields.io/badge/license-Apache--2.0-D22128)](LICENSE)

[Русский](README.md) · **English**

</div>

**Nerine** is a hobby operating system whose kernel is, first of all, **one large hypervisor**. The kernel does not "run programs": it isolates, schedules and watches, while subsystems — drivers, networking, the shell — grow on top as independent guest blooms. A crashed driver is not a kernel panic here; it is a flower to be replanted.

Named after [Nerine](https://en.wikipedia.org/wiki/Nerine) — a South African flower with slender, curled petals that blooms into the autumn dark.

## Architecture

```
┌───────────────────────────────────────────────────┐
│                     Nerine OS                     │
│                                                   │
│   ┌───────┐   ┌───────┐   ┌───────┐   ┌───────┐   │
│   │ Bloom │   │ Bloom │   │ Bloom │   │ Bloom │   │
│   │  net  │   │  fs   │   │  gpu  │   │ shell │   │
│   └───┬───┘   └───┬───┘   └───┬───┘   └───┬───┘   │
│      ┘───────────┴──────┬────┴───────────┘        │
│                   Nectar · IPC                    │
│    ┌───────────────────────────────────────────┐   │
│    │               Bulb · kernel               │   │
│    │     scheduling · memory · VT-x / SVM      │   │
│    └───────────────────────────────────────────┘   │
│                 Tunic · security                   │
│                  Hardware · x86_64                 │
└───────────────────────────────────────────────────┘
```

## The garden of names

| Component | Name | Image |
|---|---|---|
| Operating system | **Nerine** | the flower everything grows from |
| Kernel / hypervisor | **Bulb** | the bulb that feeds and shelters the plant |
| Guest subsystem | **Bloom** | one isolated bloom on the bulb |
| IPC | **Nectar** | what blooms exchange |
| Shell | **Corolla** | the part you actually touch |
| Security layer | **Tunic** | the dry skin of the bulb |

## Lineage

The design is cross-checked against real hypervisors:

- **Xen** — the domain model and the road to a minimal dom0 (dom0less)
- **Jailhouse** — static partitioning of resources between guests
- **NOVA** — the microhypervisor: a compact TCB as a goal in itself
- **seL4** — formal verification as the benchmark of trust
- **KVM / Firecracker** — the Linux host and a Rust VMM as industrial references

## Principles

- **Rust first** — the kernel is `#![no_std]` Rust; C++ only where it has earned its place.
- **Code over commentary** — comments answer "why", they never re-narrate "what".
- **Isolation by default** — if it can be a guest, it will be a guest.
- **A garden, not a monolith** — subsystems are planted, replaced and pruned independently.

## Roadmap

- [x] **Seedling** — a freestanding `no_std` kernel that builds for x86_64 ← *we are here*
- [ ] **Stem** — a custom target, UART output, the bootloader memory map
- [ ] **Bulb** — VT-x / SVM: the first guest inside a virtual machine
- [ ] **Nectar** — minimal IPC between guests
- [ ] **First blooms** — drivers as guest Blooms
- [ ] **Corolla** — an interactive shell
- [ ] **Perennial** — a self-hosting toolchain

## Building

```sh
rustup target add x86_64-unknown-none
cargo build
```

The result is `target/x86_64-unknown-none/debug/bulb` — a kernel ELF image.

## License

[Apache-2.0](LICENSE)
