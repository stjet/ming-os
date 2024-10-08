use x2apic::lapic::{ LocalApic, LocalApicBuilder };
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

use crate::memory::phys_to_virt;

pub static mut LAPIC: LApic = LApic { lapic: None };

pub struct LApic {
  lapic: Option<LocalApic>,
}

impl LApic {
  pub fn init(&mut self, local_apic_address: u64) {
    unsafe {
      let mut cmd_8259a = Port::<u8>::new(0x20);
      let mut data_8259a = Port::<u8>::new(0x21);
      let mut cmd_8259b = Port::<u8>::new(0xa0);
      let mut data_8259b = Port::<u8>::new(0xa1);

      let mut spin_port = Port::<u8>::new(0x80);
      let mut spin = || spin_port.write(0);

      cmd_8259a.write(0x11);
      cmd_8259b.write(0x11);
      spin();

      data_8259a.write(0xf8);
      data_8259b.write(0xff);
      spin();

      data_8259a.write(0b100);
      spin();

      data_8259b.write(0b10);
      spin();

      data_8259a.write(0x1);
      data_8259b.write(0x1);
      spin();

      data_8259a.write(u8::MAX);
      data_8259b.write(u8::MAX);
    }

    let apic_virtual_address: u64 = phys_to_virt(PhysAddr::new(local_apic_address)).as_u64();

    self.lapic = LocalApicBuilder::default()
      .timer_vector(32)
      //.error_vector(51)
      .error_vector(49)
      .spurious_vector(0xff)
      .set_xapic_base(apic_virtual_address)
      .build()
      .ok();
  }

  pub fn enable(&mut self) {
    unsafe {
      self.lapic.as_mut().unwrap().enable();
    }
  }

  /*
  pub fn disable(&mut self) {
    unsafe {
      self.lapic.as_mut().unwrap().disable();
    }
  }
  */

  pub fn end_interrupt(&mut self) {
    unsafe {
      self.lapic.as_mut().unwrap().end_of_interrupt();
    }
  }

  pub fn id(&self) -> u32 {
    unsafe { self.lapic.as_ref().unwrap().id() }
  }
}
