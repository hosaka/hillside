use keyberon::action::{k, l, m, Action, Action::*, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::{self, *};

#[derive(Clone, Copy)]
pub enum CustomAction {
    Reset,
    Bootloader,
}

enum Layers {
    Base,
    Extra,
    Symbol,
    Number,
    Function,
    Navigation,
    Media,
}

const fn hold_layer_tap_key(layer: usize, key: KeyCode) -> HoldTapAction<CustomAction, KeyCode> {
    return HoldTapAction {
        timeout: 200,
        tap_hold_interval: 0,
        config: HoldTapConfig::Default,
        hold: l(layer),
        tap: k(key),
    };
}

impl Layers {
    const fn num(&self) -> usize {
        return match self {
            Layers::Base => 0,
            Layers::Extra => 1,
            Layers::Symbol => 2,
            Layers::Number => 3,
            Layers::Function => 4,
            Layers::Navigation => 5,
            Layers::Media => 6,
        };
    }

    const fn hold_tap(&self, key: KeyCode) -> HoldTapAction<CustomAction, KeyCode> {
        return hold_layer_tap_key(self.num(), key);
    }
}

const CUT: Action<CustomAction> = m(&&[LCtrl, X].as_slice());
const COPY: Action<CustomAction> = m(&&[LCtrl, C].as_slice());
const PASTE: Action<CustomAction> = m(&&[LCtrl, V].as_slice());
const REDO: Action<CustomAction> = m(&&[LCtrl, Y].as_slice());
const UNDO: Action<CustomAction> = m(&&[LCtrl, Z].as_slice());

// generic USB keyboard
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
// const VID: u16 = 0x16c0;
// const PID: u16 = 0x27db;
// const PRODUCT: &str = "hillside";
// const MANUFACTURER: &str = "Hillside by https://github.com/mmccoyd";

#[allow(dead_code)]
const RESET: Action<CustomAction> = Action::Custom(CustomAction::Reset);
#[allow(dead_code)]
const BOOTLOADER: Action<CustomAction> = Action::Custom(CustomAction::Bootloader);

const ENTER_SYM: Action<CustomAction> = HoldTap(&Layers::Symbol.hold_tap(Enter));
const TAB_NUM: Action<CustomAction> = HoldTap(&Layers::Number.hold_tap(Tab));
const ESCAPE_FUN: Action<CustomAction> = HoldTap(&Layers::Function.hold_tap(Escape));
const SPACE_NUM: Action<CustomAction> = HoldTap(&Layers::Navigation.hold_tap(Space));
const DELETE_MED: Action<CustomAction> = HoldTap(&Layers::Media.hold_tap(Delete));

// macro_rules! s {
//     ($k:indent) => {
//         m(&[LShift, $k].as_slice())
//     };
// }

// todo: figure out the lang and caps word keys
pub static LAYERS: keyberon::layout::Layers<12, 4, 7, CustomAction> = keyberon::layout::layout! {
    // base
    {
        [ t Q W E R T                                     Y      U I O P    t],
        [ t A S D F G                                     H      J K L Quote t],
        [ t Z X C V B                                     N      M Comma Dot Slash    t],
        [ n t  {ESCAPE_FUN} {SPACE_NUM} {TAB_NUM} Lang1 CapsLock {ENTER_SYM} BSpace {DELETE_MED} t n],
    }

    // extra
    {
        [ t Q W F P B J L U Y Quote t],
        [ t A R S T G  M N E I O t],
        [ t Z X C D B K H Comma Dot Slash t],
        [ n t {ESCAPE_FUN} {SPACE_NUM} {TAB_NUM} t CapsLock {ENTER_SYM} BSpace {DELETE_MED} t n],
    }

    // sym
    {
        [ t '{' t * & '}' t t t t t t],
        [ t :   ^ % $ +   t RShift RCtrl RAlt RGui t],
        [ t ~   # @ ! |   t t t t t t],
        [ n t  '(' ')' '_' t t t t t t n],
    }

    // num
    {
        [ t t t t t t LBracket  7 8 9 RBracket   t],
        [ t LGui LAlt LCtrl LShift t Equal    4 5 6 SColon     t],
        [ t t t t t t  Bslash 1 2 3 Grave t],
        [ n t t t t t  t Minus 0 Dot    t t ],
    }

    // tap
    // {
    //     [ t t t t t t n n n n t t t t t t],
    //     [ t t t t t t n n n n t t t t t t],
    //     [ t t t t t t t n n t t t t t t t],
    //     [ n n n n t t t t t t t t n n n n],
    // }

    // button

    // fun
    {
        [ t t t t t t PScreen    F7 F8 F9 F12 t],
        [ t LGui LAlt LCtrl LShift t ScrollLock F4 F5 F6 F11 t],
        [ t t t t t t Pause      F1 F2 F3 F10 t],
        [ n t t t t t t Tab Space Menu        t  n],
    }

    // nav
    {
        [ t t t t t t {REDO} {PASTE} {COPY} {CUT} {UNDO} t],
        [ t LGui LAlt LCtrl LShift t Left Down Up Right CapsLock t],
        [ t t t t t t Home PgDown PgUp End Insert t],
        [ n t t t t t t t t t t n ],
    }

    // media
    {
        [ t t t t t t t t t t t t],
        [ t MediaPreviousSong MediaVolDown MediaVolUp MediaNextSong t t RShift RCtrl RAlt RGui t],
        [ t t t t t t t t t t t t],
        [ n t MediaMute MediaPlayPause MediaStop t t t t t t n],
    }

};
