#![no_std]
#![no_main]

use panic_probe as _;

// use cortex_m::prelude::_embedded_hal_watchdog_Watchdog;
use cortex_m::prelude::_embedded_hal_watchdog_WatchdogEnable;
use fugit::{ExtU32, RateExtU32};
use keyberon::debounce::Debouncer;
use keyberon::layout::{Event, Layout};
use keyberon::matrix::Matrix;
use rtic::app;
use usb_device::class_prelude::UsbBusAllocator;

use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal;
use bsp::hal::prelude::*;
use hal::gpio;
use hal::uart;

mod layout;

type GpioUartTx = gpio::bank0::Gpio0;
type GpioUartRx = gpio::bank0::Gpio1;
// note: ws2812 pin
// type GpioUsbLed = gpio::bank0::Gpio25;

type UartPins = (
    gpio::Pin<GpioUartTx, gpio::FunctionUart, gpio::PullDown>,
    gpio::Pin<GpioUartRx, gpio::FunctionUart, gpio::PullDown>,
);
type UartReader = uart::Reader<hal::pac::UART0, UartPins>;
type UartWriter = uart::Writer<hal::pac::UART0, UartPins>;

#[app(device = sparkfun_pro_micro_rp2040::hal::pac)]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: usb_device::device::UsbDevice<'static, hal::usb::UsbBus>,
        usb_class:
            keyberon::hid::HidClass<'static, hal::usb::UsbBus, keyberon::keyboard::Keyboard<()>>,
        #[lock_free]
        layout: Layout<16, 4, 1, ()>,
    }

    #[local]
    struct Local {
        matrix: Matrix<
            gpio::Pin<gpio::DynPinId, gpio::FunctionSioInput, gpio::PullUp>,
            gpio::Pin<gpio::DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
            1,
            1,
        >,
        debouncer: Debouncer<[[bool; 6]; 4]>,
        watchdog: hal::watchdog::Watchdog,
        transform: fn(Event) -> Event,
        is_left: bool,
        tx: UartReader,
        rx: UartWriter,
    }

    #[init(local = [bus: Option<UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // mcu setup
        let mut resets = ctx.device.RESETS;
        let mut watchdog = hal::watchdog::Watchdog::new(ctx.device.WATCHDOG);
        watchdog.pause_on_debug(false);

        let clocks = hal::clocks::init_clocks_and_plls(
            bsp::XOSC_CRYSTAL_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = hal::sio::Sio::new(ctx.device.SIO);
        let pins = bsp::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // uart setup
        let uart_pins = (pins.tx0.into_function(), pins.rx0.into_function());
        let uart = uart::UartPeripheral::new(ctx.device.UART0, uart_pins, &mut resets)
            .enable(
                uart::UartConfig::new(
                    38_400.Hz(),
                    uart::DataBits::Eight,
                    None,
                    uart::StopBits::One,
                ),
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

        // todo: check the right pin for sideness
        let is_left = true;

        let matrix = Matrix::new(
            [pins.gpio2.into_pull_up_input().into_dyn_pin()],
            [pins.gpio3.into_push_pull_output().into_dyn_pin()],
        )
        .unwrap();

        let transform: fn(Event) -> Event = if is_left {
            |e| e
        } else {
            |e| e.transform(|i, j| (i, 11 - j))
        };

        // usb setup
        let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
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
            },
            Local {
                matrix,
                debouncer,
                watchdog,
                transform,
                is_left,
                tx,
                rx,
            },
            init::Monotonics(),
        );
    }
}
