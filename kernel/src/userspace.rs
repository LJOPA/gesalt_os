use core::arch::asm;

#[naked]
pub unsafe extern "C" fn userspace_prog_1() {
    asm!(
        "2:",
        "nop",
        // "syscall",
        "jmp 2b",
        options(noreturn),
    )
}