//! Tock kernel for the PineTime smart watch based on the Nordic nRF52832 MCU.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::{
    capabilities,
    common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState},
    component::Component,
    create_capability, debug, hil, static_init,
};
use nrf52832::gpio::Pin;

pub(crate) const LED_PIN: Pin = Pin::P0_27;

/// Panic indicator.
pub mod io;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// Supported drivers by the platform
pub struct Platform {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, nrf52832::rtc::Rtc<'static>>,
    >,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52832::init();

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Make non-volatile memory writable and activate the reset button
    nrf52832::nvmc::NVMC.erase_uicr();
    nrf52832::nvmc::NVMC.configure_writeable();

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(Some(&nrf52832::gpio::PORT[LED_PIN]), None, None);

    ////////////////////////////////////////////////////////////////////////
    // Timers                                                             //
    ////////////////////////////////////////////////////////////////////////
    //
    // RTC for Timers
    //
    let rtc = &nrf52832::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf52832::rtc::RTC)
    );
    hil::time::Alarm::set_client(rtc, mux_alarm);

    //
    // Timer/Alarm
    //

    // Virtual alarm for the userspace timers
    let alarm_driver_virtual_alarm = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );

    // Userspace timer driver
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52832::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(
            alarm_driver_virtual_alarm,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    hil::time::Alarm::set_client(alarm_driver_virtual_alarm, alarm);

    ////////////////////////////////////////////////////////////////////////
    // RTT and Console and `debug!()`                                     //
    ////////////////////////////////////////////////////////////////////////
    let console = {
        // RTT communication channel
        let rtt_memory = components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());
        let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory)
            .finalize(components::segger_rtt_component_helper!(nrf52832::rtc::Rtc));

        // Create a shared UART channel for the console and for kernel debug.
        let uart_mux = components::console::UartMuxComponent::new(rtt, 0, dynamic_deferred_caller)
            .finalize(());

        // Setup the console.
        let console =
            components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
        // Create the debugger object that handles calls to `debug!()`.
        components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());
        console
    };

    ////////////////////////////////////////////////////////////////////////
    // Clocks                                                             //
    ////////////////////////////////////////////////////////////////////////
    {
        // Start all of the clocks. Low power operation will require a better
        // approach than this.
        use nrf52832::clock::{self, CLOCK};
        CLOCK.low_stop();
        CLOCK.high_stop();
        CLOCK.low_set_source(clock::LowClockSource::XTAL);
        CLOCK.low_start();
        CLOCK.high_set_source(clock::HighClockSource::XTAL);
        CLOCK.high_start();
        while !CLOCK.low_started() {}
        while !CLOCK.high_started() {}
    }

    let platform = Platform {
        console: console,
        alarm: alarm,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let chip = static_init!(nrf52832::chip::Chip, nrf52832::chip::new());

    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52832::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    for (proc_num, proc) in PROCESSES.as_ref().iter().enumerate() {
        if let Some(proc) = proc {
            load_process_hook(
                proc_num as u32,
                proc.get_process_name(),
                proc.flash_non_protected_start() as u32,
            )
        }
    }

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}

/// This function's sole purpose is to allow GDB to observe process
/// loading and load debug symbols from the `.elf` that corresponds to
/// that process.
#[no_mangle]
pub extern "C" fn load_process_hook(proc_num: u32, name: &str, text_addr: u32) {
    debug!(
        "Loading app {}: name \"{}\", `.text` 0x{:08x}",
        proc_num, name, text_addr
    );
}
