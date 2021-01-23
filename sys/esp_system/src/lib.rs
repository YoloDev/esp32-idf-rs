const fn bit(bit_nr: usize) -> usize {
  1 << bit_nr
}

pub const TWO_UNIVERSAL_MAC_ADDR: usize = 2usize;
pub const FOUR_UNIVERSAL_MAC_ADDR: usize = 4usize;
/// Chip has embedded flash memory
pub const CHIP_FEATURE_EMB_FLASH: usize = bit(0);
/// Chip has 2.4GHz WiFi
pub const CHIP_FEATURE_WIFI_BGN: usize = bit(1);
/// Chip has Bluetooth LE
pub const CHIP_FEATURE_BLE: usize = bit(4);
/// Chip has Bluetooth Classic
pub const CHIP_FEATURE_BT: usize = bit(5);

mod bindings;

pub use bindings::*;
