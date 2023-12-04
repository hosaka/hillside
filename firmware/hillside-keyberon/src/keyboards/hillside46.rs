use rp2040_hal as hal;

use hal::gpio::bank0;

use crate::common::{AnyPin, HillsideKeyboard, InputPin, OutputPin};

pub const COLS: usize = 6;
pub const ROWS: usize = 4;

pub fn cols(
    gp27: AnyPin<bank0::Gpio27>,
    gp26: AnyPin<bank0::Gpio26>,
    gp22: AnyPin<bank0::Gpio22>,
    gp20: AnyPin<bank0::Gpio20>,
    gp23: AnyPin<bank0::Gpio23>,
    gp21: AnyPin<bank0::Gpio21>,
) -> [InputPin; COLS] {
    return [
        gp27.into_pull_up_input().into_dyn_pin(),
        gp26.into_pull_up_input().into_dyn_pin(),
        gp22.into_pull_up_input().into_dyn_pin(),
        gp20.into_pull_up_input().into_dyn_pin(),
        gp23.into_pull_up_input().into_dyn_pin(),
        gp21.into_pull_up_input().into_dyn_pin(),
    ];
}

pub fn rows(
    gp5: AnyPin<bank0::Gpio5>,
    gp6: AnyPin<bank0::Gpio6>,
    gp7: AnyPin<bank0::Gpio7>,
    gp9: AnyPin<bank0::Gpio9>,
) -> [OutputPin; ROWS] {
    return [
        gp5.into_push_pull_output().into_dyn_pin(),
        gp6.into_push_pull_output().into_dyn_pin(),
        gp7.into_push_pull_output().into_dyn_pin(),
        gp9.into_push_pull_output().into_dyn_pin(),
    ];
}

pub type Keyboard = HillsideKeyboard<COLS, ROWS>;
