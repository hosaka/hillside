use rp2040_hal as hal;

use hal::gpio::{
    DynPinId, FunctionNull, FunctionSioInput, FunctionSioOutput, Pin, PullDown, PullUp,
};
use hal::usb::UsbBus;

use keyberon::{hid::HidClass, keyboard::Keyboard};

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
