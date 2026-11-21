//! Raw pads to the standard layout (`og_input::padmap`): a browser's own
//! order through og's lines, a Linux driver's through SDL_GameControllerDB
//! (its hat as the d-pad, a firmware version no line names), Windows.Gaming
//! .Input's fixed order, the standard layout as it is, and each pad's family.
//!
//! Ways it could fail: a trigger axis resting at -1 read as half pressed; the
//! hat or the hat-as-axes d-pad dropped; a GUID differing only in version
//! unmatched (every pad's firmware differs); XInput's y left up; the
//! database's lines not read at all (each pad reads its raw order, which
//! these orders differ from); a family lost for a vendor og does not know.

use og_input::padmap::{self, PadLayout, RawPad, parse_line};
use og_input::{Device, GAMEPAD_BUTTONS, gamepad_button};

fn held(mask: u32) -> Vec<&'static str> {
    GAMEPAD_BUTTONS
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & 1 << i != 0)
        .map(|(_, n)| *n)
        .collect()
}

fn raw(layout: PadLayout, guid: [u8; 16], buttons: &[f32], axes: &[f32], hat: u8) -> RawPad {
    let mut r = RawPad {
        connected: true,
        layout,
        guid,
        hat,
        ..RawPad::default()
    };
    r.buttons[..buttons.len()].copy_from_slice(buttons);
    r.axes[..axes.len()].copy_from_slice(axes);
    r
}

#[test]
fn a_browsers_own_order_maps_through_ogs_lines() {
    // An Xbox 360 pad on Firefox for Linux: A, LT fully in (axis 2), RT at
    // rest (axis 5 at -1), Start is raw button 7, d-pad up is axis 7 at -1.
    let r = raw(
        PadLayout::Web,
        RawPad::usb_guid(0x045e, 0x028e),
        &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        &[0.25, -0.5, 1.0, 0.75, 0.1, -1.0, 0.0, -1.0],
        0,
    );
    let s = padmap::map(&r);
    assert_eq!(held(s.buttons), ["a", "leftTrigger", "start", "up"]);
    assert_eq!(s.axes, [0.25, -0.5, 0.75, 0.1]);
    assert_eq!(s.triggers, [1.0, 0.0]);
    assert_eq!(s.device, Device::Xbox);
}

#[test]
fn a_linux_drivers_order_maps_through_the_database() {
    // An Xbox 360 pad on evdev, at a firmware version no line names: X is
    // raw 2, Start raw 7, the d-pad is hat 0 (right and down), LT on a2.
    let mut guid = RawPad::usb_guid(0x045e, 0x028e);
    guid[12] = 0x77;
    let r = raw(
        PadLayout::Linux,
        guid,
        &[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        &[0.0, 1.0, -1.0, 0.0, 0.0, -1.0],
        2 | 4,
    );
    let m = padmap::mapping(&r).expect("the database names an Xbox 360 pad on Linux");
    assert!(m.name.to_lowercase().contains("360"), "{}", m.name);
    let s = padmap::map(&r);
    assert_eq!(held(s.buttons), ["x", "start", "down", "right"]);
    assert_eq!(s.axes, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(s.triggers, [0.0, 0.0], "both at rest");
}

#[test]
fn windows_gaming_inputs_order_is_read_with_y_down() {
    // West, South, …: raw 1 is A; left y pushed up reads up (y down: -1).
    let r = raw(
        PadLayout::XInput,
        [0x77; 16],
        &[0.0, 1.0],
        &[0.0, 1.0, 0.0, -1.0, 1.0, 0.0],
        0,
    );
    let s = padmap::map(&r);
    assert_eq!(held(s.buttons), ["a", "rightTrigger"]);
    assert_eq!(s.axes[1], -1.0);
    assert_eq!(s.triggers, [0.0, 1.0]);
    assert_eq!(s.device, Device::Xbox);
}

#[test]
fn the_standard_layout_reads_as_it_is_and_a_family_comes_from_vendor_or_name() {
    let b = |n: &str| 1u32 << gamepad_button(n).unwrap();
    let r = RawPad {
        guid: RawPad::usb_guid(0x054c, 0x09cc),
        ..RawPad::standard(3, b("b") | b("home"), [0.5, 0.0, 0.0, -1.0], [0.25, 0.0])
    };
    let s = padmap::map(&r);
    assert_eq!((s.pad, held(s.buttons)), (3, vec!["b", "home"]));
    assert_eq!((s.axes, s.triggers), ([0.5, 0.0, 0.0, -1.0], [0.25, 0.0]));
    assert_eq!(s.device, Device::PlayStation);
    // No vendor og knows: the database's name tells (an 8BitDo in X mode on Linux).
    let line =
        parse_line("03000000c82d00000000000000000000,8BitDo Xbox Controller,a:b0,platform:Web,")
            .unwrap();
    assert_eq!(
        padmap::family(&RawPad::default(), Some(&line.2)),
        Device::Xbox
    );
    assert_eq!(padmap::family(&RawPad::default(), None), Device::Gamepad);
    // Gone: it holds nothing.
    let gone = padmap::map(&RawPad {
        connected: false,
        ..r
    });
    assert_eq!((gone.connected, gone.buttons), (false, 0));
}

#[test]
fn every_line_of_ogs_and_nearly_every_database_line_reads() {
    let db = include_str!("../../../third_party/SDL_GameControllerDB/gamecontrollerdb.txt");
    let lines: Vec<&str> = db
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let unread: Vec<_> = lines.iter().filter_map(|l| parse_line(l).err()).collect();
    // Only SDL's own `xinput` line is unread: og's XInput line stands for it.
    assert_eq!(unread, ["'xinput' is not a 32-digit GUID"; 2]);
    // og's lines read whole.
    let ours = padmap::mapping(&raw(
        PadLayout::Web,
        RawPad::usb_guid(0x2dc8, 0x6101),
        &[],
        &[],
        0,
    ));
    assert_eq!(
        ours.map(|m| m.name.as_str()),
        Some("8BitDo Xbox Controller")
    );
}
