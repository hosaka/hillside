#![no_std]
#![no_main]

use panic_probe as _;

use cortex_m::prelude::_embedded_hal_watchdog_Watchdog;
use cortex_m::prelude::_embedded_hal_watchdog_WatchdogEnable;
use embedded_hal::digital::v2::InputPin;
use fugit::{ExtU32, RateExtU32};
use keyberon::debounce::Debouncer;
use keyberon::hid::HidClass;
use keyberon::key_code;
use keyberon::layout::{CustomEvent, Event, Layout};
use keyberon::matrix::Matrix;
use rtic::app;
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;
use usb_device::device::UsbDeviceState;

use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal;
use bsp::hal::prelude::*;
use hal::gpio;
use hal::timer::Alarm;
use hal::uart;

mod hillside;

type GpioUartTx = gpio::bank0::Gpio0;
type GpioUartRx = gpio::bank0::Gpio1;
// note: ws2812 pin is not available on Sea Picro EXT
// type GpioUsbLed = gpio::bank0::Gpio25;

type UartPins = (
    gpio::Pin<GpioUartTx, gpio::FunctionUart, gpio::PullDown>,
    gpio::Pin<GpioUartRx, gpio::FunctionUart, gpio::PullDown>,
);
type UartReader = uart::Reader<hal::pac::UART0, UartPins>;
type UartWriter = uart::Writer<hal::pac::UART0, UartPins>;

#[app(device = sparkfun_pro_micro_rp2040::hal::pac, dispatchers = [PIO0_IRQ_0])]
mod app {

    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, hal::usb::UsbBus>,
        usb_class: HidClass<'static, hal::usb::UsbBus, keyberon::keyboard::Keyboard<()>>,
        #[lock_free]
        layout: Layout<16, 4, 7, hillside::CustomActions>,
    }

    #[local]
    struct Local {
        matrix: Matrix<
            gpio::Pin<gpio::DynPinId, gpio::FunctionSioInput, gpio::PullUp>,
            gpio::Pin<gpio::DynPinId, gpio::FunctionSioOutput, gpio::PullDown>,
            8,
            4,
        >,
        debouncer: Debouncer<[[bool; 8]; 4]>,
        watchdog: hal::watchdog::Watchdog,
        timer: hal::timer::Alarm0,
        transform: fn(Event) -> Event,
        rx: UartReader,
        tx: UartWriter,
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
        let pins = hal::gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // uart setup
        let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
        let mut uart = uart::UartPeripheral::new(ctx.device.UART0, uart_pins, &mut resets)
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

        uart.enable_rx_interrupt();
        // uart.enable_tx_interrupt();
        let (rx, tx) = uart.split();

        // keeb setup
        let layout = Layout::new(&hillside::LAYERS);
        let debouncer = Debouncer::new([[false; 8]; 4], [[false; 8]; 4], 5);

        let mut timer = hal::Timer::new(ctx.device.TIMER, &mut resets, &clocks)
            .alarm_0()
            .unwrap();
        let _ = timer.schedule(1_000.micros());
        timer.enable_interrupt();

        // todo: figureout pin numbering for hillside
        let matrix = Matrix::new(
            [
                pins.gpio2.into_pull_up_input().into_dyn_pin(),
                pins.gpio3.into_pull_up_input().into_dyn_pin(),
                pins.gpio4.into_pull_up_input().into_dyn_pin(),
                pins.gpio5.into_pull_up_input().into_dyn_pin(),
                pins.gpio6.into_pull_up_input().into_dyn_pin(),
                pins.gpio7.into_pull_up_input().into_dyn_pin(),
                pins.gpio8.into_pull_up_input().into_dyn_pin(),
                pins.gpio9.into_pull_up_input().into_dyn_pin(),
            ],
            [
                pins.gpio10.into_push_pull_output().into_dyn_pin(),
                pins.gpio11.into_push_pull_output().into_dyn_pin(),
                pins.gpio12.into_push_pull_output().into_dyn_pin(),
                pins.gpio13.into_push_pull_output().into_dyn_pin(),
            ],
        )
        .unwrap();

        // handedness
        let hand = pins.gpio14.into_floating_input();
        let is_left = hand.is_low().unwrap();
        let transform: fn(Event) -> Event = if is_left {
            |e| e
        } else {
            |e| e.transform(|i, j| (i, 15 - j))
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
                timer,
                transform,
                tx,
                rx,
            },
            init::Monotonics(),
        );
    }

    #[task(binds = TIMER_IRQ_0, priority = 1, local = [matrix, debouncer, watchdog, timer, transform, tx])]
    fn tick(ctx: tick::Context) {
        let timer = ctx.local.timer;
        timer.clear_interrupt();
        let _ = timer.schedule(1_000.micros());

        ctx.local.watchdog.feed();

        let keys = ctx.local.matrix.get().unwrap();
        for event in ctx.local.debouncer.events(keys).map(ctx.local.transform) {
            // note: serialize event and send it over tx using nb::block
            // ctx.local.tx.write_full_blocking(b"Key");
            handle_event::spawn(event).unwrap();
        }
        tick_keeb::spawn().unwrap();
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(ctx: handle_event::Context, event: Event) {
        ctx.shared.layout.event(event);
    }

    #[task(priority = 2, shared = [usb_dev, usb_class, layout])]
    fn tick_keeb(mut ctx: tick_keeb::Context) {
        let tick = ctx.shared.layout.tick();

        if ctx.shared.usb_dev.lock(|usb_dev| usb_dev.state()) != UsbDeviceState::Configured {
            return;
        }
        match tick {
            CustomEvent::Press(event) => match event {
                hillside::CustomActions::Reset => cortex_m::peripheral::SCB::sys_reset(),
                hillside::CustomActions::Bootloader => hal::rom_data::reset_to_usb_boot(0, 0),
            },
            _ => (),
        };

        // write HID report
        let report: key_code::KbHidReport = ctx.shared.layout.keycodes().collect();
        if !ctx
            .shared
            .usb_class
            .lock(|usb_class| usb_class.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        while let Ok(0) = ctx
            .shared
            .usb_class
            .lock(|usb_class| usb_class.write(report.as_bytes()))
        {}
    }

    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_rx(ctx: usb_rx::Context) {
        (ctx.shared.usb_dev, ctx.shared.usb_class).lock(|usb_dev, usb_class| {
            if usb_dev.poll(&mut [usb_class]) {
                usb_class.poll();
            }
        })
    }

    #[task(binds = UART0_IRQ, priority = 4, local = [rx])]
    fn uart_rx(ctx: uart_rx::Context) {
        // note: read uart into a shared buffer
        // ctx.local.rx.read_full_blocking(buffer)
    }
}
