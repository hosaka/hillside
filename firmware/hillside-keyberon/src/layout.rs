use keyberon::action::{k, l, m, Action::*, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

// macro_rules! s {
//     ($k:indent) => {
//         m(&[LShift, $k].as_slice())
//     };
// }

type Action = keyberon::action::Action<()>;

const CUT: Action = m(&&[LShift, Delete].as_slice());
const COPY: Action = m(&&[LCtrl, Insert].as_slice());
const PASTE: Action = m(&&[LShift, Insert].as_slice());

const NUM_TAB: Action = HoldTap(&HoldTapAction {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::HoldOnOtherKeyPress,
    hold: l(1),
    tap: k(Tab),
});

// todo: finish the rest of layer holds
// const NAV_SPACE: Action
// const FUN_ESCAPE: Action
// const SYM_ENTER: Action
// const MED_BSPACE: Action

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<16, 4, 1, ()> = keyberon::layout::layout!{
    // extra
    {
        [ t Q W E R T n n                       n n Y U I O P t],
        [ t A S D F G n n                       n n H J K L '\'' t],
        [ t Z X C V B t n                       n t N M , . / t],
        [ n n n n t Escape Space {NUM_TAB} Enter BSpace Delete t n n n n],
    }
    // todo: finish the rest of the layers

    // base
    // {
    //     [ t t t t t t n n n n t t t t t t],
    //     [ t t t t t t n n n n t t t t t t],
    //     [ t t t t t t t n n t t t t t t t],
    //     [ n n n n t t t t t t t t n n n n],
    // }

    // tap
    // button
    // nav
    // media
    // num
    // sym
    // fun
};
