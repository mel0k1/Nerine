#!/bin/sh
set -eu

LIMINE_VER=v12.9.4
LIMINE_URL=https://github.com/Limine-Bootloader/Limine/releases/download/$LIMINE_VER/limine-binary.tar.gz
KERNEL=target/x86_64-nerine/debug/bulb
QEMU=${QEMU:-qemu-system-x86_64}
QEMU_L=${QEMU_L:-}
LEAD=${LEAD:-8}

export CARGO_UNSTABLE_JSON_TARGET_SPEC=true
export CARGO_UNSTABLE_BUILD_STD=core,compiler_builtins
export CARGO_UNSTABLE_BUILD_STD_FEATURES=compiler-builtins-mem

say() {
    printf 'perennial:%s\n' "$1"
}

need() {
    command -v "$1" >/dev/null 2>&1
}

usage() {
    cat <<'EOF'
perennial - the nerine toolchain that grows itself

  ./perennial.sh setup    gather soil: nightly+rust-src, qemu, xorriso, limine
  ./perennial.sh build    grow the bulb (cargo build)
  ./perennial.sh iso      bind the bootable nerine.iso
  ./perennial.sh smoke    boot and verify every kernel marker
  ./perennial.sh shell    boot and verify a full corolla session
  ./perennial.sh doctor   weather report of the toolchain
  ./perennial.sh clean    return to seed
  ./perennial.sh all      setup build iso smoke shell

env: QEMU=bin  QEMU_L="-L dirs..."  LEAD=seconds-before-keystrokes
EOF
}

gather_limine() {
    [ -f limine/limine-binary/limine-bios-cd.bin ] && return 0
    say setup:limine
    rm -rf limine limine.tar.gz
    curl -sL -o limine.tar.gz "$LIMINE_URL"
    mkdir limine
    tar xzf limine.tar.gz -C limine
    rm -f limine.tar.gz
}

cmd_setup() {
    if ! need cargo || ! rustup show 2>/dev/null | grep -q nightly; then
        say setup:rustup
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --profile minimal -c rust-src
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    rustup component add rust-src >/dev/null 2>&1 || true
    for t in "$QEMU" xorriso cc curl tar; do
        if ! need "$t"; then
            say "setup:missing:$t"
            exit 1
        fi
    done
    gather_limine
    {
        printf 'rustc %s\n' "$(rustc --version)"
        printf 'qemu %s\n' "$("$QEMU" --version | head -1 | sed 's/QEMU emulator version //;s/ .*//')"
        printf 'limine %s\n' "$LIMINE_VER"
    } > perennial.lock
    say setup:ok
}

cmd_build() {
    cargo build
    [ -f "$KERNEL" ] || {
        say build:missing
        exit 1
    }
    say build:ok
}

cmd_iso() {
    gather_limine
    rm -rf isoroot
    mkdir -p isoroot/boot
    cp "$KERNEL" isoroot/boot/nerine.elf
    cp ci/limine.conf isoroot/limine.conf
    cp limine/limine-binary/limine-bios-cd.bin isoroot/
    cp limine/limine-binary/limine-bios.sys isoroot/
    cp limine/limine-binary/limine-uefi-cd.bin isoroot/
    xorriso -as mkisofs -o nerine.iso \
        -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table \
        -eltorito-alt-boot -b limine-uefi-cd.bin -no-emul-boot \
        isoroot >/dev/null 2>&1
    say iso:ok
}

boot() {
    feed="$1"
    log="$2"
    rm -f "$log"
    { sleep "$LEAD"; cat "$feed"; } | timeout 180 "$QEMU" -M q35 -cpu max -m 512M -cdrom nerine.iso \
        -display none -serial stdio -no-reboot $QEMU_L \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 >"$log" 2>&1 || true
}

verify() {
    log="$1"
    shift
    for m in "$@"; do
        if ! grep -q "$m" "$log"; then
            say "fail:$m"
            exit 1
        fi
    done
    say "verify:ok"
}

cmd_smoke() {
    cmd_build
    cmd_iso
    tmp=$(mktemp -d)
    printf 'exit\n' >"$tmp/feed"
    boot "$tmp/feed" target/perennial-smoke.log
    cat target/perennial-smoke.log
    rm -rf "$tmp"
    verify target/perennial-smoke.log \
        "Nerine / Bulb" nectar:1:7 pollen:1:549 soil:apic:1 soil:tick: soil:timer:1 \
        bloom:ok leaf:done sepal:caught:06 sepal:alive guest:done \
        corolla:ready corolla:bye
}

cmd_shell() {
    cmd_build
    cmd_iso
    tmp=$(mktemp -d)
    {
        printf 'help\n'
        printf 'soil\n'
        printf 'uptime\n'
        printf 'nectar\n'
        printf 'pollen\n'
        printf 'bloom\n'
        printf 'vm\n'
        printf 'gh on\n'
        printf 'gh\n'
        printf 'gh vm\n'
        printf 'gh off\n'
        printf 'gh\n'
        printf 'exit\n'
    } >"$tmp/feed"
    boot "$tmp/feed" target/perennial-shell.log
    cat target/perennial-shell.log
    rm -rf "$tmp"
    verify target/perennial-shell.log \
        "Nerine / Bulb" nectar:1:7 pollen:1:549 soil:apic:1 soil:tick: soil:timer:1 \
        bloom:ok leaf:done sepal:caught:06 sepal:alive guest:done \
        corolla:ready corolla:help corolla:soil uptime: \
        corolla:nectar corolla:pollen corolla:bloom corolla:vmx \
        greenhouse:hard corolla:gh:domains:2 greenhouse:copy:9 \
        greenhouse:deny:zerocopy corolla:gh:hard:2 gh:vm:shadow \
        greenhouse:perf corolla:bye
}

cmd_doctor() {
    say doctor
    if need cargo; then
        printf 'cargo     %s\n' "$(cargo --version)"
        if rustup show 2>/dev/null | grep -q nightly; then
            printf 'nightly   yes\n'
        else
            printf 'nightly   no\n'
        fi
        if rustup component list 2>/dev/null | grep -q 'rust-src'; then
            printf 'rust-src  yes\n'
        else
            printf 'rust-src  no\n'
        fi
    else
        printf 'cargo     missing\n'
    fi
    if need "$QEMU"; then
        printf 'qemu      %s\n' "$("$QEMU" --version | head -1 | sed 's/QEMU emulator version //;s/ .*//')"
    else
        printf 'qemu      missing\n'
    fi
    if need xorriso; then
        printf 'xorriso   %s\n' "$(xorriso --version 2>&1 | head -1 | sed 's/ : .*//')"
    else
        printf 'xorriso   missing\n'
    fi
    if [ -f limine/limine-binary/limine-bios.sys ]; then
        printf 'limine    %s\n' "$LIMINE_VER"
    else
        printf 'limine    missing\n'
    fi
    if need cc; then
        printf 'cc        yes\n'
    else
        printf 'cc        missing\n'
    fi
    say doctor:done
}

cmd_clean() {
    cargo clean
    rm -rf isoroot nerine.iso perennial.lock
    say clean:ok
}

case "${1:-}" in
    setup) cmd_setup ;;
    build) cmd_build ;;
    iso) cmd_iso ;;
    smoke) cmd_smoke ;;
    shell) cmd_shell ;;
    doctor) cmd_doctor ;;
    clean) cmd_clean ;;
    all)
        cmd_setup
        cmd_smoke
        cmd_shell
        ;;
    *)
        usage
        exit 1
        ;;
esac
