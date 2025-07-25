pub const PAGE_SIZE: usize = 4096;
#[derive(Clone, Copy)]
pub struct VAddr(usize);
impl VAddr {
    pub fn new(inner: usize) -> Self {
        Self(inner)
    }
    fn vpn(&self) -> [usize; 3] {
        const VPN_MASK: usize = 0x1ff;
        [
            // VPN[0] = vaddr[20:12]
            (self.0 >> 12) & VPN_MASK,
            // VPN[1] = vaddr[29:21]
            (self.0 >> 12 >> 9) & VPN_MASK,
            // VPN[2] = vaddr[38:30]
            (self.0 >> 12 >> 9 >> 9) & VPN_MASK,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct PAddr(pub usize);

impl PAddr {
    pub fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn ppn(&self) -> [usize; 3] {
        [
            // PPN[0] = paddr[20:12]
            (self.0 >> 12) & 0x1ff,
            // PPN[1] = paddr[29:21]
            (self.0 >> (12 + 9)) & 0x1ff,
            // PPN[2] = paddr[55:30]
            (self.0 >> (12 + 9 + 9)) & 0x3ff_ffff,
        ]
    }
}

pub mod alloc {
    use crate::page::{PAGE_SIZE, PAddr};

    #[repr(C)]
    union LinkedPage {
        page: Page,
        node: Node,
    }

    type Page = [u8; PAGE_SIZE];
    type Node = Option<usize>;

    #[repr(transparent)]
    struct Allocator<const N: usize> {
        pages: [LinkedPage; N],
    }

    impl<const N: usize> Allocator<N> {
        unsafe fn init(&mut self, head: &mut Node) {
            for (i, page) in self.pages.iter_mut().enumerate() {
                if i == N - 1 {
                    page.node = None;
                } else {
                    page.node = Some(i + 1)
                }
            }

            *head = Some(0);
        }
        unsafe fn alloc(&mut self, head: &mut Node) -> usize {
            let allocated = head.unwrap();
            *head = self.pages[allocated].node;
            allocated
        }

        unsafe fn zalloc(&mut self, head: &mut Node) -> usize {
            let allocated = self.alloc(head);
            self.pages[allocated].page = [0; PAGE_SIZE];
            allocated
        }

        unsafe fn free(&mut self, idx: usize, head: &mut Node) {
            let freed: usize = idx;
            self.pages[freed].node = *head;
            *head = Some(freed);
        }

        fn get_index_addr(&self, idx: usize) -> PAddr {
            let val = &self.pages[idx];
            PAddr::new(val as *const _ as usize)
        }
        fn get_addr_index(&self, addr: PAddr) -> usize {
            let base = self as *const _;
            let other = addr.0 as *const Allocator<N>;
            unsafe { other.offset_from_unsigned(base) }
        }
    }

    unsafe extern "C" {
        #[link_name = "_heap_start"]
        #[allow(improper_ctypes)]
        static mut HEAP: Allocator<16384>;
    }

    static mut LIST_HEAD: Node = None;

    pub fn init() {
        unsafe { HEAP.init(&mut LIST_HEAD) }
    }

    pub fn alloc() -> PAddr {
        unsafe {
            let idx = HEAP.alloc(&mut LIST_HEAD);
            HEAP.get_index_addr(idx)
        }
    }

    pub fn zalloc() -> PAddr {
        unsafe {
            let idx = HEAP.zalloc(&mut LIST_HEAD);
            HEAP.get_index_addr(idx)
        }
    }

    pub fn free(addr: PAddr) {
        unsafe {
            let idx = HEAP.get_addr_index(addr);
            HEAP.free(idx, &mut LIST_HEAD)
        }
    }
}

pub mod vmm {
    use core::{
        ops::{Index, IndexMut},
        ptr::null_mut,
    };

    use crate::page::{
        PAddr, VAddr,
        alloc::{free, zalloc},
    };

    #[derive(Default, Clone, Copy)]
    pub struct Entry(usize);

    impl Entry {
        pub fn new(inner: usize) -> Self {
            Self(inner)
        }
        fn is_valid(&self) -> bool {
            self.0 & (EntryFlags::V.bits() as usize) != 0
        }

        fn is_branch(&self) -> bool {
            self.0 & (EntryFlags::RWX.bits() as usize) == 0
        }

        fn as_table_mut(&mut self) -> Option<&mut Table> {
            if self.is_branch() && self.is_valid() {
                Some(unsafe { (self.as_address().0 as *mut Table).as_mut_unchecked() })
            } else {
                None
            }
        }

        pub fn as_address(&self) -> PAddr {
            PAddr::new((self.0 << 2) & !0xFFF)
        }
    }

    bitflags::bitflags! {
        #[derive(Clone, Copy)]
        pub struct EntryFlags: u8 {
            const V = 1 << 0;

            const R = 1 << 1;
            const W = 1 << 2;
            const X = 1 << 3;

            const U = 1 << 4;
            const G = 1 << 5;

            const A = 1 << 6;
            const D = 1 << 7;

            const RW = Self::R.bits() | Self::W.bits();
            const RX = Self::R.bits() | Self::X.bits();
            const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();

            const URW = Self::R.bits() | Self::W.bits() | Self::U.bits();
            const URE = Self::R.bits() | Self::X.bits() | Self::U.bits();
            const URWE = Self::R.bits() | Self::W.bits() | Self::X.bits() | Self::U.bits();
        }
    }

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct Table([Entry; 512]);
    impl IndexMut<usize> for Table {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.0[index]
        }
    }

    impl Index<usize> for Table {
        type Output = Entry;

        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }

    impl Table {
        fn len() -> usize {
            512
        }
    }

    #[derive(Debug)]
    #[repr(transparent)]
    pub struct AddrSpaceHandle {
        ptr: *mut Table,
    }

    impl AddrSpaceHandle {
        pub const fn null() -> Self {
            Self { ptr: null_mut() }
        }
    }

    impl Default for AddrSpaceHandle {
        fn default() -> Self {
            Self {
                ptr: zalloc().0 as *mut _,
            }
        }
    }

    impl Drop for AddrSpaceHandle {
        fn drop(&mut self) {
            if let Some(root) = unsafe { self.ptr.as_mut() } {
                for entry_lvl2 in root.0.iter_mut() {
                    // This is a valid entry, so drill down and free.
                    if let Some(table) = entry_lvl2.as_table_mut() {
                        // Make table_lv1 a mutable reference instead of a pointer.
                        for entry_lv1 in table.0.iter_mut() {
                            if let Some(table) = entry_lv1.as_table_mut() {
                                // The next level is level 0, which
                                // cannot have branches, therefore,
                                // we free here.
                                free(PAddr::new(table as *mut _ as usize));
                            }
                        }
                        free(PAddr::new(table as *mut _ as usize));
                    }
                }
            }
        }
    }

    impl AddrSpaceHandle {
        pub fn map(&self, vaddr: VAddr, paddr: PAddr, bits: EntryFlags) {
            let vpn = vaddr.vpn();
            let ppn = paddr.ppn();
            let root = unsafe { self.ptr.as_mut_unchecked() };

            let mut entry = &mut root[vpn[2]];

            for i in (0..2).rev() {
                if !entry.is_valid() {
                    // Allocate a page
                    let page = zalloc();
                    // The page is stored in the entry shifted right by 2 places.
                    entry.0 = (page.0 >> 2) | EntryFlags::V.bits() as usize;
                }

                // Cast the address to a table
                let table: *mut Table = entry.as_address().0 as *mut Table;
                entry = unsafe { ((&mut (&mut *table)[vpn[i]]) as *mut Entry).as_mut_unchecked() };
            }
            // When we get here, we should be at VPN[0] and v should be pointing to
            // our entry.
            let bits: usize = (ppn[2] << 10 << 9 << 9) |   // PPN[2] = [53:28]
			(ppn[1] << 10 << 9) |   // PPN[1] = [27:19]
			(ppn[0] << 10) |   // PPN[0] = [18:10]
			(bits | EntryFlags::V).bits() as usize;
            *entry = Entry::new(bits);
        }
        pub fn get_ptr(&self) -> *mut Table {
            self.ptr
        }
    }
}
