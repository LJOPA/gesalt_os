use core::arch::asm;
use conquer_once::spin::OnceCell;
use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTable;

static PROCESSES: OnceCell<Mutex<Vec<Process>>> = OnceCell::uninit();
static NEXT_PID: OnceCell<Mutex<u64>> = OnceCell::uninit();

pub fn init_sched() {
    NEXT_PID.try_init_once(|| Mutex::new(1));
    PROCESSES.try_init_once(|| Mutex::new(Vec::new()));
}

pub struct Process {
    pid: u64,
    context: Context,
    page_table: Box<PageTable>
}

#[derive(Default)]
struct Context {
    rbp: u64,
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

#[inline(always)]
unsafe fn get_context() -> *const Context {
    let ctxp: *const Context;
    asm!("push r15; push r14; push r13; push r12; push r11; push r10; push r9;\
    push r8; push rdi; push rsi; push rdx; push rcx; push rbx; push rax; push rbp;\
    mov {}, rsp; sub rsp, 0x400;",
    out(reg) ctxp);
    ctxp
}

#[inline(always)]
unsafe fn restore_context(ctxr: &Context) {
    asm!("mov rsp, {};\
    pop rbp; pop rax; pop rbx; pop rcx; pop rdx; pop rsi; pop rdi; pop r8; pop r9;\
    pop r10; pop r11; pop r12; pop r13; pop r14; pop r15; iretq;",
    in(reg) ctxr);
}

#[inline(never)]
pub unsafe fn jmp_to_usermode(code: VirtAddr, stack_end: VirtAddr) {
    use crate::gdt::set_usermode_segs;
    
    let (cs_idx, ds_idx) = set_usermode_segs();
    x86_64::instructions::tlb::flush_all(); // flush the TLB after address-space switch
    asm!(
    "push rax",   // stack segment
    "push rsi",   // rsp
    "push 0x200", // rflags (only interrupt bit set)
    "push rdx",   // code segment
    "push rdi",   // ret to virtual addr
    "iretq",
    in("rdi") code.as_u64(), in("rsi") stack_end.as_u64(), in("dx") cs_idx, in("ax") ds_idx
    );
}

unsafe fn create_thread(pt: Box<PageTable>) -> Option<u64> {
    let mut returnval = None;
    
    if let Ok(proc_list) = PROCESSES.try_get() {
        if let Ok(next_pid) = NEXT_PID.try_get() {
            let mut next_pid_ptr = next_pid.lock();
            let new_proc = Process {
                pid: *next_pid_ptr,
                context: Context { ..Default::default() },
                page_table: pt
            };
    
            let created_pid = *next_pid_ptr;

            *next_pid_ptr = *next_pid_ptr + 1;
            proc_list
                .lock()
                .push(new_proc);
            returnval = Some(created_pid)
        } else {
            returnval = None
        }
    } else {
        returnval = None
    }

    returnval
}