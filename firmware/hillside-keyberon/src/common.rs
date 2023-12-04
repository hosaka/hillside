use rp2040_hal as hal;

use hal::gpio::{
    DynPinId, FunctionNull, FunctionSioInput, FunctionSioOutput, FunctionUart, Pin, PullDown,
    PullNone, PullUp,
};
use hal::usb::UsbBus;

use keyberon::{debounce::Debouncer, hid::HidClass, keyboard::Keyboard, matrix::Matrix};

pub type UsbClass = HidClass<'static, UsbBus, Keyboard<()>>;
pub type UsbDevice = usb_device::device::UsbDevice<'static, UsbBus>;

// generic USB keyboard
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
pub const VID: u16 = 0x16c0;
pub const PID: u16 = 0x27db;
pub const PRODUCT: &str = "hillside";
pub const MANUFACTURER: &str = "Hillside by https://github.com/mmccoyd";

pub type AnyPin<I> = Pin<I, FunctionNull, PullDown>;
pub type InputPin = Pin<DynPinId, FunctionSioInput, PullUp>;
pub type OutputPin = Pin<DynPinId, FunctionSioOutput, PullDown>;

type GpioUartTx = hal::gpio::bank0::Gpio0;
type GpioUartRx = hal::gpio::bank0::Gpio1;

type UartPins = (
    Pin<GpioUartTx, FunctionUart, PullNone>,
    Pin<GpioUartRx, FunctionUart, PullNone>,
);
pub type UartReader = hal::uart::Reader<hal::pac::UART0, UartPins>;
pub type UartWriter = hal::uart::Writer<hal::pac::UART0, UartPins>;

pub type PressedKeys<const COLS: usize, const ROWS: usize> = [[bool; COLS]; ROWS];
pub type PressedKeys6x4 = PressedKeys<6, 4>;

pub struct HillsideKeyboard<const COLS: usize, const ROWS: usize> {
    pub matrix: Matrix<InputPin, OutputPin, COLS, ROWS>,
    pub debouncer: Debouncer<PressedKeys<COLS, ROWS>>,
}

impl<const COLS: usize, const ROWS: usize> HillsideKeyboard<COLS, ROWS> {
    pub fn new(
        matrix: Matrix<InputPin, OutputPin, COLS, ROWS>,
        debouncer: Debouncer<PressedKeys<COLS, ROWS>>,
    ) -> Self {
        Self { matrix, debouncer }
    }

    // pub fn events(&mut self) {
    //     let keys = self.matrix.get().unwrap();
    //     return self.debouncer.events(keys).map(self.transform);
    // }
}
