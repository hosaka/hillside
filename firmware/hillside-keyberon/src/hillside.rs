use keyberon::action::{k, l, m, Action, Action::*, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

#[derive(Clone, Copy)]
pub enum CustomActions {
    Reset,
    Bootloader,
}

const CUT: Action<CustomActions> = m(&&[LCtrl, X].as_slice());
const COPY: Action<CustomActions> = m(&&[LCtrl, C].as_slice());
const PASTE: Action<CustomActions> = m(&&[LCtrl, V].as_slice());
const REDO: Action<CustomActions> = m(&&[LCtrl, Y].as_slice());
const UNDO: Action<CustomActions> = m(&&[LCtrl, Z].as_slice());

// generic USB keyboard
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
// const VID: u16 = 0x16c0;
// const PID: u16 = 0x27db;
// const PRODUCT: &str = "hillside";
// const MANUFACTURER: &str = "Hillside by https://github.com/mmccoyd";

#[allow(dead_code)]
const RESET: Action<CustomActions> = Action::Custom(CustomActions::Reset);
#[allow(dead_code)]
const BOOTLOADER: Action<CustomActions> = Action::Custom(CustomActions::Bootloader);

const SYM_ENTER: Action<CustomActions> = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(1),
    tap: k(Enter),
});

const NUM_TAB: Action<CustomActions> = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(2),
    tap: k(Tab),
});

const FUN_ESCAPE: Action<CustomActions> = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(3),
    tap: k(Escape),
});
const NAV_SPACE: Action<CustomActions> = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(4),
    tap: k(Space),
});
const MED_DELETE: Action<CustomActions> = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(5),
    tap: k(Delete),
});

// macro_rules! s {
//     ($k:indent) => {
//         m(&[LShift, $k].as_slice())
//     };
// }

// todo: figure out the lang and caps word keys
pub static LAYERS: keyberon::layout::Layers<16, 4, 7, CustomActions> = keyberon::layout::layout! {
    // base
    {
        [ t Q W E R T      n     n         n           n      Y      U I O P    t],
        [ t A S D F G      n     n         n           n      H      J K L Quote t],
        [ t Z X C V B      t     n         n           t      N      M Comma Dot Slash    t],
        [ n n n n t {FUN_ESCAPE} {NAV_SPACE} {NUM_TAB} {SYM_ENTER} BSpace {MED_DELETE} t n n n    n],
    }

    // extra
    {
        [ t Q W F P B n n n n J L U Y Quote t],
        [ t A R S T G n n n n M N E I O t],
        [ t Z X C D B t n n t K H Comma Dot Slash t],
        [ n n n n t {FUN_ESCAPE} {NAV_SPACE} {NUM_TAB} {SYM_ENTER} BSpace {MED_DELETE} t n n n    n],
    }

    // sym
    {
        [ t '{' t * & '}' n   n   n n t t t t t t],
        [ t :   ^ % $ +   n   n   n n t RShift RCtrl RAlt t t],
        [ t ~   # @ ! |   t   n   n t t t t t t t],
        [ n n   n n t '(' ')' '_' t t t t n n n n],
    }

    // tap

    // button

    // nav
    {
        [ t t t t t t n n n n {REDO} {PASTE} {COPY} {CUT} {UNDO} t],
        [ t t LAlt LCtrl LShift t n n n n Left Down Up Right CapsLock t],
        [ t t t t t t t n n t Home PgDown PgUp End Insert t],
        [ n n n n t t t t t t t t n n n n],
    }

    // media
    {
        [ t t t t t t n n n n t t t t t t],
        [ t MediaPreviousSong MediaVolDown MediaVolUp MediaNextSong t n n n n t RShift RCtrl RAlt t t],
        [ t t t t t t t n n t t t t t t t],
        [ n n n n t MediaMute MediaPlayPause MediaStop t t t t n n n n],
    }

    // num
    {
        [ t t t t t t n n n n LBracket  7 8 9 RBracket   t],
        [ t t LAlt LCtrl LShift t n n n n Equal    4 5 6 SColon     t],
        [ t t t t t t t n n t Bslash 1 2 3 Grave t],
        [ n n n n t t t t Minus 0 Dot    t n n n     n],
    }
    // fun
    {
        [ t t t t t t n n n   n     PScreen    F7 F8 F9 F12 t],
        [ t t LAlt LCtrl LShift t n n n   n     ScrollLock F4 F5 F6 F11 t],
        [ t t t t t t t n n   t     Pause      F1 F2 F3 F10 t],
        [ n n n n t t t t Tab Space Menu       t  n  n  n   n],
    }
};
