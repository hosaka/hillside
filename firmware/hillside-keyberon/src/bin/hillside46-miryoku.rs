#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m::prelude::*;
use embedded_hal::digital::v2::InputPin;
use fugit::{ExtU32, RateExtU32};
use keyberon::debounce::Debouncer;
use keyberon::hid::HidClass;
use keyberon::key_code;
use keyberon::layout::{CustomEvent, Event, Layout};
use keyberon::matrix::Matrix;
use nb::block;
use rtic::app;
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;
use usb_device::device::UsbDeviceState;

use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal;
use hal::prelude::*;
use hal::timer::Alarm;
use hal::uart;
use hal::usb;

use hillside_keyberon::common;
use hillside_keyberon::keyboards::hillside46;
use hillside_keyberon::layouts;
use hillside_keyberon::mcu;

#[app(device = hal::pac, dispatchers = [PIO0_IRQ_0])]
mod app {

    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, usb::UsbBus>,
        usb_class: HidClass<'static, usb::UsbBus, keyberon::keyboard::Keyboard<()>>,
        #[lock_free]
        layout: Layout<12, 4, 7, layouts::common::CustomAction>,
    }

    #[local]
    struct Local {
        matrix: Matrix<common::InputPin, common::OutputPin, 6, 4>,
        debouncer: Debouncer<common::PressedKeys6x4>,
        watchdog: hal::watchdog::Watchdog,
        alarm: hal::timer::Alarm0,
        transform: fn(Event) -> Event,
        rx: common::UartReader,
        tx: common::UartWriter,
        buffer: [u8; 4],
    }

    #[init(local = [bus: Option<UsbBusAllocator<usb::UsbBus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        warn!("mcu setup");
        let mut resets = ctx.device.RESETS;
        let (mut watchdog, clocks) = mcu::init_clocks(
            ctx.device.WATCHDOG,
            bsp::XOSC_CRYSTAL_FREQ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut resets,
        );
        let (_timer, alarm) = mcu::init_timer(ctx.device.TIMER, &mut resets, &clocks);

        info!("usb setup");
        *ctx.local.bus = Some(UsbBusAllocator::new(usb::UsbBus::new(
            ctx.device.USBCTRL_REGS,
            ctx.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut resets,
        )));
        let usb_bus = ctx.local.bus.as_ref().unwrap();
        let (usb_dev, usb_class) = mcu::init_usb(
            usb_bus,
            common::VID,
            common::PID,
            common::MANUFACTURER,
            "hillside46",
        );

        let sio = hal::sio::Sio::new(ctx.device.SIO);
        let pins = bsp::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        info!("uart setup");
        let uart_pins = (pins.gpio0.reconfigure(), pins.gpio1.reconfigure());
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

        info!("keyboard setup");
        // gpio19 is vbus_detect on sea picro
        // note: col0 and row3 can signal left hand if bridge is soldered
        let vbus_pin = pins.gpio19.into_floating_input();
        let is_host = vbus_pin.is_high().unwrap();
        let transform: fn(Event) -> Event = if !is_host {
            |e| e
        } else {
            |e| e.transform(|i, j| (i, 11 - j))
        };

        let cols = hillside46::cols(
            pins.gpio27,
            pins.gpio26,
            pins.gpio22,
            pins.gpio20,
            // this is gpio23 in rp2040 and copi in sparkfun pro micro
            pins.b_power_save,
            pins.gpio21,
        );
        let rows = hillside46::rows(pins.gpio5, pins.gpio6, pins.gpio7, pins.gpio9);
        let matrix = Matrix::new(cols, rows).unwrap();
        let debouncer = Debouncer::new(
            common::PressedKeys6x4::default(),
            common::PressedKeys6x4::default(),
            5,
        );
        let layout = Layout::new(&layouts::miryoku::LAYERS);
        // let keyboard = hillside46::Keyboard { matrix, debouncer };

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
                alarm,
                transform,
                tx,
                rx,
                buffer: [0; 4],
            },
            init::Monotonics(),
        );
    }

    #[task(binds = TIMER_IRQ_0, priority = 1, local = [matrix, debouncer, watchdog, alarm, transform, tx])]
    fn tick(ctx: tick::Context) {
        let alarm = ctx.local.alarm;
        alarm.clear_interrupt();
        alarm.schedule(1_000.micros()).unwrap();

        ctx.local.watchdog.feed();

        let keys = ctx.local.matrix.get().unwrap();
        for event in ctx.local.debouncer.events(keys).map(ctx.local.transform) {
            // serialize an event and send it byte-by-byte over tx
            // for &byte in &serialize(event) {
            //     block!(ctx.local.tx.write(byte)).unwrap();
            // }
            // let buff = serialize(event);
            // ctx.local.tx.write_full_blocking(&buff);
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
                layouts::common::CustomAction::Reset => cortex_m::peripheral::SCB::sys_reset(),
                layouts::common::CustomAction::Bootloader => hal::rom_data::reset_to_usb_boot(0, 0),
            },
            _ => (),
        };

        // write HID report
        let report = ctx
            .shared
            .layout
            .keycodes()
            .collect::<key_code::KbHidReport>();
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

    #[task(binds = UART0_IRQ, priority = 4, local = [rx, buffer])]
    fn uart_rx(ctx: uart_rx::Context) {
        if let Ok(byte) = ctx.local.rx.read() {
            ctx.local.buffer.rotate_left(1);
            ctx.local.buffer[3] = byte;

            if ctx.local.buffer[3] == b'\n' {
                if let Ok(event) = deserialize(&ctx.local.buffer[..]) {
                    handle_event::spawn(event).unwrap();
                }
            }
        }
    }
}

fn deserialize(bytes: &[u8]) -> Result<Event, ()> {
    return match *bytes {
        [b'P', i, j, b'\n'] => Ok(Event::Press(i, j)),
        [b'R', i, j, b'\n'] => Ok(Event::Release(i, j)),
        _ => Err(()),
    };
}

fn serialize(event: Event) -> [u8; 4] {
    return match event {
        Event::Press(i, j) => [b'P', i, j, b'\n'],
        Event::Release(i, j) => [b'R', i, j, b'\n'],
    };
}
