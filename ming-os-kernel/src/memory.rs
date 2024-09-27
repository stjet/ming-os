use bootloader_api::info::{ MemoryRegions, MemoryRegionKind };
use x86_64::{
  structures::paging::{ FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB },
  PhysAddr, VirtAddr,
};

static mut PHYSICAL_MEMORY_OFFSET: VirtAddr = VirtAddr::zero();

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
  let level_4_table = active_level_4_table(physical_memory_offset);
  unsafe { PHYSICAL_MEMORY_OFFSET = physical_memory_offset; }
  OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
  use x86_64::registers::control::Cr3;

  let (level_4_table_frame, _) = Cr3::read();

  let phys = level_4_table_frame.start_address();
  let virt = physical_memory_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  &mut *page_table_ptr // unsafe
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    None
  }
}

pub struct BootInfoFrameAllocator {
  memory_regions: &'static MemoryRegions,
  next: usize,
}

impl BootInfoFrameAllocator {
  pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
    BootInfoFrameAllocator {
      memory_regions,
      next: 0,
    }
  }

  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    // get usable regions from memory map
    let regions = self.memory_regions.iter();
    let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
    // map each region to its address range
    let addr_ranges = usable_regions.map(|r| r.start..r.end);
    // transform to an iterator of frame start addresses
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
    // create `PhysFrame` types from the start addresses
    frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    let frame = self.usable_frames().nth(self.next);
    self.next += 1;
    frame
  }
}

pub fn phys_to_virt(phys_addr: PhysAddr) -> VirtAddr {
  unsafe {
    VirtAddr::new(phys_addr.as_u64() + PHYSICAL_MEMORY_OFFSET.as_u64())
  }
}
