use acpi::{ AcpiHandler, PhysicalMapping };
use core::ptr::NonNull;
use x86_64::PhysAddr;

use crate::memory::phys_to_virt;

#[derive(Debug, Clone, Copy)]
pub struct Handler;

impl AcpiHandler for Handler {
  unsafe fn map_physical_region<T>(
    &self,
    physical_address: usize,
    size: usize,
  ) -> acpi::PhysicalMapping<Self, T> {
    let phys_addr = PhysAddr::new(physical_address as u64);
    let virt_addr = phys_to_virt(phys_addr);
    let ptr = NonNull::new(virt_addr.as_mut_ptr()).unwrap();
    PhysicalMapping::new(physical_address, ptr, size, size, Self)
  }

  fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}
