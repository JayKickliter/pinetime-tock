//! Tock kernel for the [PineTime](https://www.pine64.org/pinetime) smartwatch.
//!
//! It is based on nRF52838 SoC (Cortex M4 core with a BLE transceiver) with many exported
//! I/O and peripherals.
//!
//! Author
//! -------------------
//! * Jay Kickliter <jay@kickliter.com>
//! * 28 March 2020

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
#[allow(unused_imports)]
use kernel::{debug, debug_gpio, debug_verbose, static_init};
use nrf52832::gpio::Pin;

const LED_PIN: Pin = Pin::P0_17;
const BUTTON_DRIVE_PIN: Pin = Pin::P0_03;
const BUTTON_SENSE_PIN: Pin = Pin::P0_13;

/// UART Writer
pub mod io;

// FIXME: Ideally this should be replaced with Rust's builtin tests by conditional compilation
//
// Also read the instructions in `tests` how to run the tests
// #[allow(dead_code)]
// mod tests;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps
static mut CHIP: Option<&'static nrf52832::chip::Chip> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52832::init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let gpio = components::gpio::GpioComponent::new(board_kernel).finalize(
        components::gpio_component_helper!(
            // Button out. High side of button voltage divider.
            &nrf52832::gpio::PORT[BUTTON_DRIVE_PIN]
        ),
    );

    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            &nrf52832::gpio::PORT[BUTTON_SENSE_PIN],
            kernel::hil::gpio::ActivationMode::ActiveHigh,
            kernel::hil::gpio::FloatingState::PullNone
        )),
    );

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!((
        &nrf52832::gpio::PORT[LED_PIN],
        kernel::hil::gpio::ActivationMode::ActiveHigh
    )));

    let chip = static_init!(nrf52832::chip::Chip, nrf52832::chip::new());
    CHIP = Some(chip);
}
