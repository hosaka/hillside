use keyberon::action::{k, l, m, Action, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::{self, *};

// pub mod segments;
// pub use segments::*;

#[derive(Clone, Copy)]
pub enum CustomAction {
    Reset,
    Bootloader,
}

pub const fn hold_layer_tap_key(
    layer: usize,
    key: KeyCode,
) -> HoldTapAction<CustomAction, KeyCode> {
    return HoldTapAction {
        timeout: 200,
        tap_hold_interval: 0,
        config: HoldTapConfig::Default,
        hold: l(layer),
        tap: k(key),
    };
}

pub const CUT: Action<CustomAction> = m(&&[LCtrl, X].as_slice());
pub const COPY: Action<CustomAction> = m(&&[LCtrl, C].as_slice());
pub const PASTE: Action<CustomAction> = m(&&[LCtrl, V].as_slice());
pub const REDO: Action<CustomAction> = m(&&[LCtrl, Y].as_slice());
pub const UNDO: Action<CustomAction> = m(&&[LCtrl, Z].as_slice());

#[allow(dead_code)]
pub const RESET: Action<CustomAction> = Action::Custom(CustomAction::Reset);
#[allow(dead_code)]
pub const BOOTLOADER: Action<CustomAction> = Action::Custom(CustomAction::Bootloader);
