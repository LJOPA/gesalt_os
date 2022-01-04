pub unsafe fn userspace_prog_1() {
    asm!("\
        nop
        nop
        nop
    ":::: "intel");
}