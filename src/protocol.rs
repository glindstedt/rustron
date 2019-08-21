use crate::midi::SysExPacket;

pub const SYSEX_MESSAGE_START: u8 = 0xf0;
pub const SYSEX_EOX: u8 = 0xf7;
pub const BEHRINGER_MANUFACTURER: [u8; 3] = [0x00, 0x20, 0x32];
pub const PROBABLY_NEUTRON_DEVICE: u8 = 0x28;
pub const PROBABLY_COMMAND_SEQUENCE: [u8; 2] = [0x7f, 0x0a];
pub const MAYBE_STATIC: [u8; 3] = [0x28, 0x7f, 0x0a];

pub fn is_behringer_packet(bytes: &[u8]) -> bool {
    bytes.is_sysex() && bytes.sysex_manufacturer() == BEHRINGER_MANUFACTURER
}

pub fn format_behringer_packet(bytes: &[u8]) -> String {
    let device = bytes[4];
    let mut buffer = String::new();
    if device == PROBABLY_NEUTRON_DEVICE {
        buffer.push_str("N ");
        buffer.push_str(hex::encode(&bytes[5..]).as_str());
    } else {
        buffer.push_str(hex::encode([bytes[4]].as_ref()).as_str());
        buffer.push_str(" ");
        buffer.push_str(hex::encode(&bytes[5..]).as_str());
    }
    format!("B[ {} ]", buffer)
}

pub fn wrap_message(message: Vec<u8>) -> Vec<u8> {
    let mut wrapped_message = vec![
        SYSEX_MESSAGE_START,
        BEHRINGER_MANUFACTURER[0],
        BEHRINGER_MANUFACTURER[1],
        BEHRINGER_MANUFACTURER[2],
        MAYBE_STATIC[0],
        MAYBE_STATIC[1],
        MAYBE_STATIC[2],
    ];
    wrapped_message.extend(message);
    wrapped_message.push(SYSEX_EOX);
    wrapped_message
}

pub enum Toggle {
    On,
    Off,
}

fn toggle_value(t: Toggle) -> u8 {
    match t {
        Toggle::On => 0x01,
        Toggle::Off => 0x00,
    }
}

pub fn toggle_paraphonic_mode(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x0f, toggle_value(t)])
}

pub fn toggle_osc_sync(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x0e, toggle_value(t)])
}

// OSC 1

pub fn toggle_osc_1_blend_mode(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x20, toggle_value(t)])
}

pub fn osc_1_range_32() -> Vec<u8> {
    wrap_message(vec![0x26, 0x00])
}

pub fn osc_1_range_16() -> Vec<u8> {
    wrap_message(vec![0x26, 0x01])
}

pub fn osc_1_range_8() -> Vec<u8> {
    wrap_message(vec![0x26, 0x02])
}

pub fn osc_1_range_pm_10_oct() -> Vec<u8> {
    wrap_message(vec![0x26, 0x03])
}

// OSC 2
pub fn toggle_osc_2_blend_mode(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x21, toggle_value(t)])
}

pub fn osc_2_range_32() -> Vec<u8> {
    wrap_message(vec![0x27, 0x00])
}

pub fn osc_2_range_16() -> Vec<u8> {
    wrap_message(vec![0x27, 0x01])
}

pub fn osc_2_range_8() -> Vec<u8> {
    wrap_message(vec![0x27, 0x02])
}

pub fn osc_2_range_pm_10_oct() -> Vec<u8> {
    wrap_message(vec![0x27, 0x03])
}

// LFO
pub fn toggle_lfo_blend_mode(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x30, toggle_value(t)])
}

pub fn toggle_lfo_key_sync(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x37, toggle_value(t)])
}

pub fn toggle_lfo_one_shot(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x31, toggle_value(t)])
}

// VCF
pub fn toggle_vcf_key_tracking(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x11, toggle_value(t)])
}


// ======================= UNVERIFIED =======================

pub fn osc_key_split() -> Vec<u8> {
    // TODO parameter
    // 0x00 = Disabled
    // 0x18 = C0
    // 0x19 = C#0/Db0
    // 0x1a = D0
    // 0x1b = D#0/Eb0
    // 0x1c = E0
    // 0x1d = F0
    // 0x1e = F#0/Gb0
    // 0x1f = G0
    // 0x20 = G#0/Ab0
    // 0x21 = A0
    // 0x22 = A#0/Bb0
    // 0x23 = B0
    // ...  = C1
    // ...
    // 0x56 = D5
    wrap_message(vec![0x28, 0x00])
}

pub fn toggle_osc_1_tune_pot(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x22, toggle_value(t)])
}

pub fn osc_1_autoglide() -> Vec<u8> {
    // TODO parameter
    // 0x00 <-> 0x18 for a range of 25 (-12 to +12)
    wrap_message(vec![0x24, 0x00])
}

pub fn toggle_osc_2_tune_pot(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x23, toggle_value(t)])
}

pub fn osc_2_autoglide() -> Vec<u8> {
    // TODO parameter
    // 0x00 <-> 0x18 for a range of 25 (-12 to +12)
    wrap_message(vec![0x25, 0x00])
}

pub fn toggle_osc_2_key_track_hold(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x2a, toggle_value(t)])
}

pub fn toggle_lfo_retrigger(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x3b, toggle_value(t)])
}

pub fn toggle_lfo_midi_sync(t: Toggle) -> Vec<u8> {
    wrap_message(vec![0x35, toggle_value(t)])
}

pub fn lfo_key_tracking() -> Vec<u8> {
    // TODO parameter
    // 0x00 = Disabled
    // 0x0c = C-1
    // ...
    // 0x17 = B-1
    // ...
    // 0x6c = C7
    wrap_message(vec![0x32, 0x00])
}

pub fn lfo_depth() -> Vec<u8> {
    // TODO parameter
    // 0x00 = 0%
    // ...
    // 0x3f = 100%
    wrap_message(vec![0x34, 0x00])
}

pub fn lfo_reset_order() -> Vec<u8> {
    wrap_message(vec![0x39, 0x00])
}

pub fn lfo_shape_order() -> Vec<u8> {
    // TODO param
    // For some reason the app sends updates for all shapes when one shape is saved
    // Positions: 0x00 - 0x04
    // Shapes:
    // 0x00 = ∿
    // 0x01 = /\
    // 0x02 = |\
    // 0x03 = _П_
    // 0x04 = /|
    wrap_message(vec![
        0x38, 0x00, // Position
        0x00, // Shape
    ])
}

pub fn lfo_phase_offset() -> Vec<u8> {
    // TODO param
    // For some reason the app sends updates for all shapes when one shape is saved
    // Positions: 0x00 - 0x04
    // Offsets:
    // 0x00 = 0°
    // 0x01 = 45°
    // 0x02 = 90°
    // 0x03 = 135°
    // 0x04 = 180°
    // 0x05 = 225°
    // 0x06 = 270°
    // 0x07 = 315°
    wrap_message(vec![
        0x38, 0x00, // Position
        0x00, // Offset
    ])
}

pub fn vcf_mod_depth() -> Vec<u8> {
    // TODO param
    // 0x00 = 0%
    // 0x3f = 100%
    wrap_message(vec![0x14, 0x00])
}

pub fn vcf_mod_source() -> Vec<u8> {
    // TODO param
    // 0x00 = OFF
    // 0x01 = After Touch
    // 0x02 = Mod Wheel
    // 0x03 = Velocity
    wrap_message(vec![0x12, 0x00])
}

pub fn vcf_mode() -> Vec<u8> {
    // TODO param
    // 0x00 = 1 (1 High 2 Band)
    // 0x01 = 2 (1 Band 2 Low)
    // 0x02 = 3 (1 Low  2 High)
    wrap_message(vec![0x10, 0x00])
}

// Options
pub fn midi_channel() -> Vec<u8> {
    // TODO param
    // 0x00 = channel 1
    // ...
    // 0x0f = channel 16
    wrap_message(vec![0x00, 0x00])
}

pub fn disable_midi_dips() -> Vec<u8> {
    wrap_message(vec![0x0a, 0x01])
}

pub fn enable_midi_dips() -> Vec<u8> {
    wrap_message(vec![0x0a, 0x00])
}

pub fn env_retrigger_staccato() -> Vec<u8> {
    wrap_message(vec![0x05, 0x00])
}

pub fn env_retrigger_legato() -> Vec<u8> {
    wrap_message(vec![0x05, 0x01])
}

pub fn note_priority() -> Vec<u8> {
    // TODO param
    // 0x00 = Low
    // 0x01 = High
    // 0x02 = Last
    wrap_message(vec![0x01, 0x00])
}

pub fn pitch_bend_range() -> Vec<u8> {
    // TODO param
    // 0x00 = 0
    // ...
    // 0x18 = 24
    wrap_message(vec![0x03, 0x00])
}

pub fn assignable_out() -> Vec<u8> {
    // TODO param
    // 0x00 = OSC 1
    // 0x01 = OSC 2
    // 0x02 = Velocity
    // 0x03 = Mod Wheel
    // 0x00 = After Touch
    wrap_message(vec![0x04, 0x00])
}

pub fn poly_chain_mode_on() -> Vec<u8> {
    wrap_message(vec![0x08, 0x01])
}

pub fn poly_chain_mode_off() -> Vec<u8> {
    wrap_message(vec![0x08, 0x00])
}

pub fn key_range_mute() -> Vec<u8> {
    wrap_message(vec![0x0b, 0x01])
}

pub fn key_range_unmute() -> Vec<u8> {
    wrap_message(vec![0x0b, 0x00])
}

pub fn key_range_min() -> Vec<u8> {
    // TODO param
    // 0x18 = C0
    // ...
    // 0x57 = D#5/Eb5
    wrap_message(vec![0x0c, 0x18])
}

pub fn key_range_max() -> Vec<u8> {
    // TODO param
    // Values decreasing
    // 0x60 = C6
    // ...
    // 0x21 = A0
    wrap_message(vec![0x0d, 0x60])
}

pub fn key_range_reset() -> Vec<u8> {
    wrap_message(vec![0x06, 0x00])
}

pub fn restore_default_settings() -> Vec<u8> {
    // 0x0a not included when restoring settings
    // TODO App keeps sending 0x05 about once per second, also without the 0x0a, what does it mean?
    let mut wrapped_message = vec![
        SYSEX_MESSAGE_START,
        BEHRINGER_MANUFACTURER[0],
        BEHRINGER_MANUFACTURER[1],
        BEHRINGER_MANUFACTURER[2],
        MAYBE_STATIC[0],
        MAYBE_STATIC[1],
    ];
    wrapped_message.push(0x0b);
    wrapped_message.push(SYSEX_EOX);
    wrapped_message
}

// INPUT DOCUMENTATION

// Sent periodically (about once every second) by the neutron app, the neutron responds with one
// long message of 33 bytes that seems to be the configuration state, followed by 24 messages of
// 25 bytes with varying data. I assume this data is related to the tuners or possibly some clock
pub fn maybe_request_state() -> Vec<u8> {
    let mut wrapped_message = vec![
        SYSEX_MESSAGE_START,
        BEHRINGER_MANUFACTURER[0],
        BEHRINGER_MANUFACTURER[1],
        BEHRINGER_MANUFACTURER[2],
        MAYBE_STATIC[0],
        MAYBE_STATIC[1],
    ];
    wrapped_message.push(0x05);
    wrapped_message.push(SYSEX_EOX);
    wrapped_message
}

// F0 00 20 32 28 00 06 01  6B 02 00 00 02 31 08 59  46 00 00 00 00 00 00 00  7F 0F 00 00 00 00 00 01  F7

// TEST
// OSC SYNC OFF, PARAPHONIC MODE OFF
// F0 00 20 32 28 00 06 01  6B 02 00 00 02 31 08 58  46 00 00 00 00 00 00 00  7F 0F 00 00 00 00 00 01  F7
// OSC SYNC ON              |
// F0 00 20 32 28 00 06 01  7B 02 00 00 02 31 08 58  46 00 00 00 00 00 00 00  7F 0F 00 00 00 00 00 01  F7
// PARAPHONIC MODE ON                             |
// F0 00 20 32 28 00 06 01  7B 02 00 00 02 31 08 59  46 00 00 00 00 00 00 00  7F 0F 00 00 00 00 00 01  F7


// Maybe firmware version?
// Only sent once when first connecting to the neutron
pub fn maybe_request_state2() -> Vec<u8> {
    let mut wrapped_message = vec![
        SYSEX_MESSAGE_START,
        BEHRINGER_MANUFACTURER[0],
        BEHRINGER_MANUFACTURER[1],
        BEHRINGER_MANUFACTURER[2],
        MAYBE_STATIC[0],
        MAYBE_STATIC[1],
    ];
    wrapped_message.push(0x73);
    wrapped_message.push(SYSEX_EOX);
    wrapped_message
}
// Sample response: F0 00 20 32 28 00 74 01  32 2E 30 2E 32 F7

// Possibly tuner values plus other stuff:
// Header:
// 28 00 72 01
//
// Payload:
// 8 hex values (16 digits)
// 8 hex values (16 digits)
//
// within the payloads, the first hex changes often,
// the second sometimes, and the last one sometimes

// unknown stuff
// possibly settings state?
// 28 00 06 01
// 00 01 00 00 02 31 08 58
// 46 00 00 00 00 00 00 00
// 7f 2f 00 00 00 00 00 01
//
// Seems like this is an answer to the app sending a message with '28 7f 05'

// Probably confirmation that OSC 1 Blend mode was set to SWITCH (28 7f 0a 20 01)
// 28 00 5a 01 20 01
// Probably confirmation that OSC 1 Blend mode was set to BLEND (28 7f 0a 20 00)
// 28 00 5a 01 20 00
