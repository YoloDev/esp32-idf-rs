use core::{ffi::c_void, hint::unreachable_unchecked};

use bitflags::bitflags;
use esp_idf_system_sys as sys;

#[non_exhaustive]
pub enum EspMacType {
  WifiSta,
  WifiSoftAP,
  Bt,
  Eth,
  Unknown,
}

#[non_exhaustive]
pub enum EspResetReason {
  /// Reset reason can not be determined
  Unknown,
  /// Reset due to power-on event
  Poweron,
  /// Reset by external pin (not applicable for ESP32)
  ExternalPin,
  /// Software reset via esp_restart
  Software,
  /// Software reset due to exception/panic
  Panic,
  /// Reset (software or hardware) due to interrupt watchdog
  InterruptWatchdog,
  /// Reset due to task watchdog
  TaskWatchdog,
  /// Reset due to other watchdogs
  OtherWatchdog,
  /// Reset after exiting deep sleep mode
  Deepsleep,
  /// Brownout reset (software or hardware)
  Brownout,
  /// Reset over SDIO
  SDIO,
}

/// Restart PRO and APP CPUs.
///
/// This function can be called both from PRO and APP CPUs.
/// After successful restart, CPU reset reason will be SW_CPU_RESET.
/// Peripherals (except for WiFi, BT, UART0, SPI1, and legacy timers) are not reset.
/// This function does not return.
pub fn restart() -> ! {
  unsafe {
    sys::esp_restart();
    unreachable_unchecked()
  }
}

impl EspResetReason {
  /// Get reason of last reset
  pub fn get() -> Self {
    match unsafe { sys::esp_reset_reason() } {
      sys::esp_reset_reason_t_ESP_RST_POWERON => Self::Poweron,
      sys::esp_reset_reason_t_ESP_RST_EXT => Self::ExternalPin,
      sys::esp_reset_reason_t_ESP_RST_SW => Self::Software,
      sys::esp_reset_reason_t_ESP_RST_PANIC => Self::Panic,
      sys::esp_reset_reason_t_ESP_RST_INT_WDT => Self::InterruptWatchdog,
      sys::esp_reset_reason_t_ESP_RST_TASK_WDT => Self::TaskWatchdog,
      sys::esp_reset_reason_t_ESP_RST_WDT => Self::OtherWatchdog,
      sys::esp_reset_reason_t_ESP_RST_DEEPSLEEP => Self::Deepsleep,
      sys::esp_reset_reason_t_ESP_RST_BROWNOUT => Self::Brownout,
      sys::esp_reset_reason_t_ESP_RST_SDIO => Self::SDIO,
      _ => Self::Unknown,
    }
  }
}

/// Get the size of available heap.
///
/// Note that the returned value may be larger than the maximum contiguous block
/// which can be allocated.
pub fn free_heap_size() -> u32 {
  unsafe { sys::esp_get_free_heap_size() }
}

/// Get the minimum heap that has ever been available
pub fn minimum_free_heap_size() -> u32 {
  unsafe { sys::esp_get_minimum_free_heap_size() }
}

/// Get one random 32-bit word from hardware RNG
///
/// The hardware RNG is fully functional whenever an RF subsystem is running (ie Bluetooth or WiFi is enabled). For
/// random values, call this function after WiFi or Bluetooth are started.
///
/// If the RF subsystem is not used by the program, the function bootloader_random_enable() can be called to enable an
/// entropy source. bootloader_random_disable() must be called before RF subsystem or I2S peripheral are used. See these functions'
/// documentation for more details.
///
/// Any time the app is running without an RF subsystem (or bootloader_random) enabled, RNG hardware should be
/// considered a PRNG. A very small amount of entropy is available due to pre-seeding while the IDF
/// bootloader is running, but this should not be relied upon for any use.
pub fn random() -> u32 {
  unsafe { sys::esp_random() }
}

/// Fill a buffer with random bytes from hardware RNG
///
/// This function has the same restrictions regarding available entropy [random].
///
/// # Arguments
///
/// * `buf` - Buffer to fill with random numbers
pub fn fill_random(buf: &mut [u8]) {
  unsafe { sys::esp_fill_random(buf.as_mut_ptr() as *mut c_void, buf.len()) }
}

// TODO: Figure out how to deal with CString and alloc
// /// Trigger a software abort
// ///
// /// # Arguments
// ///
// /// * `details` - Details that will be displayed during panic handling
// pub fn system_abort(details: &str) -> ! {
//   let details = CString::new(details).unwrap();
//   unsafe {
//     sys::esp_system_abort(details.as_ptr());
//     unreachable_unchecked()
//   }
// }

/// Chip models
#[non_exhaustive]
pub enum EspChipModel {
  /// ESP32
  Esp32,
  /// ESP32-S2
  Esp32S2,
  /// Others
  Unknown,
}

impl EspChipModel {
  fn from_raw(raw: sys::esp_chip_model_t) -> Self {
    match raw {
      sys::esp_chip_model_t_CHIP_ESP32 => Self::Esp32,
      sys::esp_chip_model_t_CHIP_ESP32S2 => Self::Esp32S2,
      _ => Self::Unknown,
    }
  }
}

const fn bit(bit_num: u8) -> u32 {
  1u32 << bit_num
}

bitflags! {
  /// Chip feature flags, used in [EspChipInfo]
  pub struct ChipFeature: u32 {
    /// Chip has embedded flash memory
    const EMB_FLASH = bit(0);
    /// Chip has 2.4GHz WiFi
    const WIFI_BGN = bit(1);
    /// Chip has Bluetooth LE
    const BLE = bit(4);
    /// Chip has Bluetooth Classic
    const BT = bit(5);
  }
}

/// The structure represents information about the chip
pub struct EspChipInfo {
  /// chip model, one of [EspChipModel]
  pub model: EspChipModel,
  /// bit mask of [ChipFeature] flags
  pub features: ChipFeature,
  /// number of CPU cores
  pub cores: u8,
  /// chip revision number
  pub revision: u8,
}

impl EspChipInfo {
  pub fn get() -> Self {
    let mut info = core::mem::MaybeUninit::uninit();
    let info = unsafe {
      sys::esp_chip_info(info.as_mut_ptr());
      info.assume_init()
    };

    EspChipInfo {
      model: EspChipModel::from_raw(info.model),
      features: ChipFeature::from_bits_truncate(info.features),
      cores: info.cores,
      revision: info.revision,
    }
  }
}
