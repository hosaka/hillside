use rp2040_hal as hal;

use hal::{pac, timer::Alarm};

use fugit::ExtU32;

use usb_device::device::UsbDeviceBuilder;
use usb_device::{bus::UsbBusAllocator, device::UsbVidPid};

use crate::common::{UsbClass, UsbDevice};

pub fn init_clocks(
    watchdog: pac::WATCHDOG,
    xosc_freq: u32,
    xosc: pac::XOSC,
    clocks: pac::CLOCKS,
    pll_sys: pac::PLL_SYS,
    pll_usb: pac::PLL_USB,
    resets: &mut pac::RESETS,
) -> (hal::Watchdog, hal::clocks::ClocksManager) {
    let mut watchdog = hal::watchdog::Watchdog::new(watchdog);
    watchdog.pause_on_debug(false);

    let clocks = hal::clocks::init_clocks_and_plls(
        xosc_freq,
        xosc,
        clocks,
        pll_sys,
        pll_usb,
        resets,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    return (watchdog, clocks);
}

pub fn init_timer(
    timer: pac::TIMER,
    resets: &mut pac::RESETS,
    clocks: &hal::clocks::ClocksManager,
) -> (hal::timer::Timer, hal::timer::Alarm0) {
    let mut timer = hal::timer::Timer::new(timer, resets, &clocks);
    let mut alarm = timer.alarm_0().unwrap();

    let _ = alarm.schedule(1_000.micros());
    alarm.enable_interrupt();
    return (timer, alarm);
}

pub fn init_uart() {}

pub fn init_usb(
    usb_bus: &'static UsbBusAllocator<hal::usb::UsbBus>,
    vid: u16,
    pid: u16,
    manufacturer: &'static str,
    product: &'static str,
) -> (UsbDevice, UsbClass) {
    let usb_class = UsbClass::new(keyberon::keyboard::Keyboard::new(()), usb_bus);

    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(vid, pid))
        .manufacturer(manufacturer)
        .product(product)
        .serial_number(env!("CARGO_PKG_VERSION"))
        .build();

    return (usb_dev, usb_class);
}
