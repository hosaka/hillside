#![no_std]
#![no_main]

use panic_probe as _;

// use cortex_m::prelude::_embedded_hal_watchdog_Watchdog;
use cortex_m::prelude::_embedded_hal_watchdog_WatchdogEnable;
use fugit::{ExtU32, RateExtU32};
use keyberon::debounce::Debouncer;
use keyberon::layout::Layout;
use keyberon::matrix::Matrix;
use rtic::app;
use usb_device::class_prelude::UsbBusAllocator;

use bsp::{
    hal::{
        clocks::init_clocks_and_plls,
        gpio::{FunctionUart, Pin, PullDown},
        pac,
        sio::Sio,
        uart::{DataBits, Reader, StopBits, UartConfig, UartPeripheral, Writer},
        usb::UsbBus,
        watchdog::Watchdog,
        Clock,
    },
    XOSC_CRYSTAL_FREQ,
};
use sparkfun_pro_micro_rp2040 as bsp;

mod layout;

#[app(device = sparkfun_pro_micro_rp2040::hal::pac)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: usb_device::device::UsbDevice<'static, UsbBus>,
        usb_class: keyberon::hid::HidClass<'static, UsbBus, keyberon::keyboard::Keyboard<()>>,
        #[lock_free]
        layout: Layout<16, 4, 1, ()>,
    }

    #[local]
    struct Local {
        // todo:  matrix: Matrix<, , 6, 4>,
        debouncer: Debouncer<[[bool; 6]; 4]>,
        watchdog: Watchdog,
        is_right: bool,
        // tx: Reader<
        //     pac::UART0,
        //     (
        //         Pin<Gpio0, FunctionUart, PullDown>,
        //         Pin<Gpio0, FunctionUart, PullDown>,
        //     ),
        // >,
        // rx: Writer<pac::UART0, >,
    }

    #[init(local = [bus: Option<UsbBusAllocator<UsbBus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // mcu setup
        let mut resets = ctx.device.RESETS;
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        watchdog.pause_on_debug(false);

        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = Sio::new(ctx.device.SIO);
        let pins = bsp::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // uart setup
        let uart_pins = (pins.tx0.into_function(), pins.rx0.into_function());
        let uart = UartPeripheral::new(ctx.device.UART0, uart_pins, &mut resets)
            .enable(
                UartConfig::new(38_400.Hz(), DataBits::Eight, None, StopBits::One),
                clocks.peripheral_clock.freq(),
            )
            .unwrap();

        // rxne
        // uart.enable_rx_interrupt();
        // uart.enable_tx_interrupt();
        let (tx, rx) = uart.split();

        // keeb setup
        let layout = Layout::new(&crate::layout::LAYERS);
        let debouncer = Debouncer::new([[false; 6]; 4], [[false; 6]; 4], 5);

        // note: ws2812 pin is D3

        // usb setup
        let usb_bus = UsbBusAllocator::new(UsbBus::new(
            ctx.device.USBCTRL_REGS,
            ctx.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut resets,
        ));

        *ctx.local.bus = Some(usb_bus);
        let usb = ctx.local.bus.as_ref().unwrap();
        let usb_class = keyberon::new_class(usb, ());
        let usb_dev = keyberon::new_device(usb);

        watchdog.start(10_000.micros());

        return (
            Shared {
                usb_dev,
                usb_class,
                layout,
                // tx,
                // rx,
            },
            Local {
                debouncer,
                watchdog,
                is_right: true,
            },
            init::Monotonics(),
        );
    }
}
