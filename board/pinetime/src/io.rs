use core::panic::PanicInfo;
use kernel::{debug, hil::led};

/// Panic.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    let led = &mut led::LedLow::new(&mut nrf52832::gpio::PORT[crate::LED_PIN]);
    debug::panic_blink_forever(&mut [led])
}
