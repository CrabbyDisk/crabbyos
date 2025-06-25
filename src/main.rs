#![no_std]
#![no_main]
#![allow(static_mut_refs, unsafe_op_in_unsafe_fn)]
#![feature(sync_unsafe_cell, ptr_as_ref_unchecked)]

mod page;
mod uart;

use riscv::register::{self, mepc, mstatus, pmpaddr0, pmpcfg0, satp};

use page::{
    PAGE_SIZE, PAddr, VAddr,
    vmm::{AddrSpaceHandle, EntryFlags},
};

use core::{
    arch::{asm, global_asm},
    panic,
};

global_asm!(include_str!("./entry.s"));

unsafe extern "C" {
    #[link_name = "_text_start"]
    safe static TEXT_START: u8;
    #[link_name = "_text_end"]
    safe static TEXT_END: u8;
    #[link_name = "_data_start"]
    safe static DATA_START: u8;
    #[link_name = "_data_end"]
    safe static DATA_END: u8;
    #[link_name = "_bss_start"]
    safe static BSS_START: u8;
    #[link_name = "_bss_end"]
    safe static BSS_END: u8;
    #[link_name = "_rodata_start"]
    safe static RODATA_START: u8;
    #[link_name = "_rodata_end"]
    safe static RODATA_END: u8;
    #[link_name = "_stack_start"]
    safe static STACK_START: u8;
    #[link_name = "_stack_end"]
    safe static STACK_END: u8;
    #[link_name = "_heap_start"]
    safe static HEAP_START: u8;
    #[link_name = "_heap_end"]
    safe static HEAP_END: u8;
}

static mut KASPACE: AddrSpaceHandle = AddrSpaceHandle::null();

pub fn id_map_range(root: &AddrSpaceHandle, start: usize, end: usize, bits: EntryFlags) {
    let num_kb_pages = (end - start) / PAGE_SIZE;
    let mut memaddr = start;

    for _ in 0..num_kb_pages {
        page::vmm::AddrSpaceHandle::map(root, VAddr::new(memaddr), PAddr::new(memaddr), bits);
        memaddr += 1 << 12;
    }
}

#[unsafe(no_mangle)]
unsafe fn _init() -> ! {
    // Do NOT fucking forget to do this
    page::alloc::init();

    KASPACE = AddrSpaceHandle::default();
    let kaspace = unsafe { &KASPACE };

    id_map_range(
        kaspace,
        &TEXT_START as *const u8 as usize,
        &TEXT_END as *const u8 as usize,
        page::vmm::EntryFlags::RX,
    );
    // Map rodata sectionn
    id_map_range(
        kaspace,
        &RODATA_START as *const u8 as usize,
        &RODATA_END as *const u8 as usize,
        page::vmm::EntryFlags::R,
    );
    // Map data section
    id_map_range(
        kaspace,
        &DATA_START as *const u8 as usize,
        &DATA_END as *const u8 as usize,
        page::vmm::EntryFlags::RW,
    );
    // Map bss section
    id_map_range(
        kaspace,
        &BSS_START as *const u8 as usize,
        &BSS_END as *const u8 as usize,
        page::vmm::EntryFlags::RW,
    );
    // Map kernel stack
    id_map_range(
        kaspace,
        &STACK_START as *const u8 as usize,
        &STACK_END as *const u8 as usize,
        page::vmm::EntryFlags::RW,
    );
    // Map heap descriptors
    id_map_range(
        kaspace,
        &HEAP_START as *const u8 as usize,
        &HEAP_END as *const u8 as usize,
        page::vmm::EntryFlags::RW,
    );
    kaspace.map(
        VAddr::new(0x1000_0000),
        PAddr::new(0x1000_0000),
        EntryFlags::RW,
    );
    // Set the SATP register
    let root_ppn = KASPACE.get_ptr() as usize >> 12;
    satp::set(satp::Mode::Sv39, 0, root_ppn);
    // Set MPP
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    // Initialize the PMP
    // TODO
    pmpaddr0::write(usize::MAX);
    pmpcfg0::set_pmp(0, register::Range::NAPOT, register::Permission::RWX, false);

    let f: extern "C" fn() -> ! = kmain;
    mepc::write(f as usize);
    asm!("mret");
    panic!();
}

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    kprint!("hello from supervisor mode");
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kprint!("shit went wrong");
    loop {}
}
