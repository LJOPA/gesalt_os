use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        let code_sel = gdt.add_entry(Descriptor::kernel_code_segment()); // kernel code segment
        let data_sel = gdt.add_entry(Descriptor::kernel_data_segment()); // kernel data segment
        let tss_sel = gdt.add_entry(Descriptor::tss_segment(&TSS)); // task state segment
        let user_data_sel = gdt.add_entry(Descriptor::user_data_segment()); // user data segment
        let user_code_sel = gdt.add_entry(Descriptor::user_code_segment()); // user code segment
        
        (gdt, Selectors {
            code_selector: code_sel,
            data_selector: data_sel,
            tss_selector: tss_sel,
            user_code_selector: user_code_sel,
            user_data_selector: user_data_sel })
    };
}

pub fn init() {
    use x86_64::registers::segmentation::{CS, DS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

pub unsafe fn init_syscalls() {
    use crate::syscall_wrapper;
    use x86_64::registers::model_specific::{Star, LStar, SFMask};
    use x86_64::registers::rflags::RFlags;
    
    SFMask::write(RFlags::INTERRUPT_FLAG);
    
    LStar::write(x86_64::addr::VirtAddr::from_ptr(syscall_wrapper as *const ()));
    
    // write segments to use on syscall/sysret to AMD'S MSR_STAR register
    Star::write(GDT.1.user_code_selector, GDT.1.tss_selector, GDT.1.code_selector, GDT.1.tss_selector);
}

#[inline(always)]
pub unsafe fn set_usermode_segs() -> (u16, u16) {
    use x86_64::instructions::segmentation::Segment;
    use x86_64::registers::segmentation::{DS};
    use x86_64::PrivilegeLevel;
    
    // set ds and tss, return cs and ds
    let (mut cs, mut ds) = (GDT.1.user_code_selector, GDT.1.user_data_selector);
    cs.0 |= PrivilegeLevel::Ring3 as u16;
    ds.0 |= PrivilegeLevel::Ring3 as u16;
    DS::set_reg(ds);
    (cs.0, ds.0)
}