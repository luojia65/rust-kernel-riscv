#![macro_use]
use alloc::vec;

use alloc::vec::Vec;
use bitflags::bitflags;

use crate::mem::frame_alloc;
use crate::mem::frame_allocator::FrameTracker;
use crate::mem::FrameNumber;
use crate::mem::PageNumber;

bitflags! {
  pub struct PTEFlags: u8 {
    const V = 1 << 0;
    const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
  }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(frame_number: FrameNumber, flag: PTEFlags) -> Self {
        PageTableEntry {
            bits: frame_number.bits << 10 | flag.bits as usize,
        }
    }

    pub fn frame_number(&self) -> FrameNumber {
        (self.bits >> 10 & ((1 << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn is_readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn is_writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn is_executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

#[repr(C)]
pub struct PageTable {
    root_frame_number: FrameNumber,
    frame_list: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_frame_number: frame.frame_number,
            frame_list: vec![frame],
        }
    }

    pub fn from_token(satp: usize) -> Self {
        Self {
            root_frame_number: FrameNumber::from(satp & ((1 << 44) - 1)),
            frame_list: Vec::new(),
        }
    }

    pub fn map(&mut self, page_number: PageNumber, frame_number: FrameNumber, flags: PTEFlags) {
        let pte = self.create_pte(page_number).unwrap();
        *pte = PageTableEntry::new(frame_number, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, page_number: PageNumber) {
        let pte = self.find_pte(page_number).unwrap();
        *pte = PageTableEntry::default();
    }

    pub fn translate(&self, page_number: PageNumber) -> Option<PageTableEntry> {
        self.find_pte(page_number).map(|pte| pte.clone())
    }

    fn find_pte(&self, page_number: PageNumber) -> Option<&mut PageTableEntry> {
        let index = page_number.get_index();
        let mut frame_number = self.root_frame_number;
        for (i, pte_index) in index.iter().enumerate() {
            let pte = &mut frame_number.get_pte_mut()[*pte_index];
            if i == 2 {
                return Some(pte);
            }

            if pte.is_valid() {
                frame_number = pte.frame_number();
            } else {
                break;
            }
        }
        None
    }

    fn create_pte(&mut self, page_number: PageNumber) -> Option<&mut PageTableEntry> {
        let index = page_number.get_index();
        let mut frame_number = self.root_frame_number;
        for (i, pte_index) in index.iter().enumerate() {
            let pte = &mut frame_number.get_pte_mut()[*pte_index];
            if i == 2 {
                return Some(pte);
            }

            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.frame_number, PTEFlags::V);
                self.frame_list.push(frame);
            }
            frame_number = pte.frame_number();
        }
        None
    }
}
