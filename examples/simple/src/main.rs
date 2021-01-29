#![no_std]
#![no_main]

extern crate panic_halt;
extern crate xtensa_lx_rt;

#[xtensa_lx_rt::entry]
fn main() -> ! {
  // let _ = esp_idf_system::EspChipInfo::get();

  #[allow(clippy::empty_loop)]
  loop {}
}
