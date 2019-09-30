use std::fmt::{Debug, Display};
use strum_macros::EnumIter;

pub const SYSEX_MESSAGE_START: u8 = 0xf0;
pub const SYSEX_EOX: u8 = 0xf7;
pub const BEHRINGER_MANUFACTURER: [u8; 3] = [0x00, 0x20, 0x32];
pub const NEUTRON_DEVICE: u8 = 0x28;
pub const NEUTRON_MESSAGE_HEADER: [u8; 5] = [
    SYSEX_MESSAGE_START,
    BEHRINGER_MANUFACTURER[0],
    BEHRINGER_MANUFACTURER[1],
    BEHRINGER_MANUFACTURER[2],
    NEUTRON_DEVICE,
];
pub const MAYBE_STATIC: [u8; 3] = [0x28, 0x7f, 0x0a];

pub const COMMS_PROTOCOL_V1: u8 = 0x01;

pub fn format_behringer_packet(bytes: &[u8]) -> String {
    let device = bytes[4];
    let mut buffer = String::new();
    if device == NEUTRON_DEVICE {
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

pub trait ByteBuilder {
    fn append_to(&self, buffer: &mut Vec<u8>);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ToggleOption {
    On,
    Off,
}

impl ToggleOption {
    pub fn as_byte(&self) -> u8 {
        match self {
            ToggleOption::On => 0x01,
            ToggleOption::Off => 0x00,
        }
    }
}

/// A percentage value representation for the Neutron. The value will be capped to 63 (0x3f), as
/// that is the value that corresponds to 100% in the Neutron. This means that the resolution
/// is 64 different values, with 0 = 0% and 63 = 100%.
///
/// # Example
///
/// ```rust
/// use rustron_lib::protocol::Percent;
///
/// let p1 = Percent::from_byte(255);
/// assert_eq!(p1.as_byte(), 63);
/// assert_eq!(p1.as_percentage(), 100 as f32);
///
/// let p2 = Percent::from_percentage(50);
/// assert_eq!(p2.as_byte(), 31);
/// assert_eq!(p2.as_percentage(), 49.20635);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Percent {
    value: u8,
}

impl Percent {
    pub fn from_byte(value: u8) -> Self {
        return Percent {
            value: value.min(63),
        };
    }

    pub fn from_percentage(value: u8) -> Self {
        return Percent {
            value: ((value.min(100) as f32 / 100f32) * 63f32) as u8,
        };
    }

    pub fn as_byte(&self) -> u8 {
        return self.value;
    }

    pub fn as_percentage(&self) -> f32 {
        return self.value as f32 / 63f32 * 100f32;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AutoglideSemitones {
    MinusTwelve,
    MinusEleven,
    MinusTen,
    MinusNine,
    MinusEight,
    MinusSeven,
    MinusSix,
    MinusFive,
    MinusFour,
    MinusThree,
    MinusTwo,
    MinusOne,
    Zero,
    PlusOne,
    PlusTwo,
    PlusThree,
    PlusFour,
    PlusFive,
    PlusSix,
    PlusSeven,
    PlusEight,
    PlusNine,
    PlusTen,
    PlusEleven,
    PlusTwelve,
}

impl AutoglideSemitones {
    pub fn as_byte(&self) -> u8 {
        match self {
            AutoglideSemitones::MinusTwelve => 0x00,
            AutoglideSemitones::MinusEleven => 0x01,
            AutoglideSemitones::MinusTen => 0x02,
            AutoglideSemitones::MinusNine => 0x03,
            AutoglideSemitones::MinusEight => 0x04,
            AutoglideSemitones::MinusSeven => 0x05,
            AutoglideSemitones::MinusSix => 0x06,
            AutoglideSemitones::MinusFive => 0x07,
            AutoglideSemitones::MinusFour => 0x08,
            AutoglideSemitones::MinusThree => 0x09,
            AutoglideSemitones::MinusTwo => 0x0a,
            AutoglideSemitones::MinusOne => 0x0b,
            AutoglideSemitones::Zero => 0x0c,
            AutoglideSemitones::PlusOne => 0x0d,
            AutoglideSemitones::PlusTwo => 0x0e,
            AutoglideSemitones::PlusThree => 0x0f,
            AutoglideSemitones::PlusFour => 0x10,
            AutoglideSemitones::PlusFive => 0x11,
            AutoglideSemitones::PlusSix => 0x12,
            AutoglideSemitones::PlusSeven => 0x13,
            AutoglideSemitones::PlusEight => 0x14,
            AutoglideSemitones::PlusNine => 0x15,
            AutoglideSemitones::PlusTen => 0x16,
            AutoglideSemitones::PlusEleven => 0x17,
            AutoglideSemitones::PlusTwelve => 0x18,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlendMode {
    Switch,
    Blend,
}

impl BlendMode {
    pub fn as_byte(&self) -> u8 {
        match self {
            BlendMode::Switch => 0x01,
            BlendMode::Blend => 0x00,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OscRange {
    // Oscillator Pipe Lengths
    ThirtyTwo,
    Sixteen,
    Eight,
    // +/- 10 Octaves
    PlusMinusTen,
}

impl OscRange {
    pub fn as_byte(&self) -> u8 {
        match self {
            OscRange::ThirtyTwo => 0x00,
            OscRange::Sixteen => 0x01,
            OscRange::Eight => 0x02,
            OscRange::PlusMinusTen => 0x03,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyTrackMode {
    Track,
    Hold,
}

impl KeyTrackMode {
    pub fn as_byte(&self) -> u8 {
        match self {
            KeyTrackMode::Track => 0x00,
            KeyTrackMode::Hold => 0x01,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LfoIndex {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl LfoIndex {
    pub fn as_byte(&self) -> u8 {
        match self {
            LfoIndex::One => 0x00,
            LfoIndex::Two => 0x01,
            LfoIndex::Three => 0x02,
            LfoIndex::Four => 0x03,
            LfoIndex::Five => 0x04,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LfoShape {
    Sine,
    Triangle,
    FallingSaw,
    Square,
    RisingSaw,
}

impl LfoShape {
    pub fn as_byte(&self) -> u8 {
        match self {
            LfoShape::Sine => 0x00,
            LfoShape::Triangle => 0x01,
            LfoShape::FallingSaw => 0x02,
            LfoShape::Square => 0x03,
            LfoShape::RisingSaw => 0x04,
        }
    }
}

/// Lfo phase offset in degrees
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LfoPhaseOffset {
    Zero,
    FourtyFive,
    Ninety,
    HundredThirtyFive,
    HundredEighty,
    TwoHundredTwentyFive,
    TwoHundredSeventy,
    ThreeHundredFifteen,
}

impl LfoPhaseOffset {
    pub fn as_byte(&self) -> u8 {
        match self {
            LfoPhaseOffset::Zero => 0x00,
            LfoPhaseOffset::FourtyFive => 0x01,
            LfoPhaseOffset::Ninety => 0x02,
            LfoPhaseOffset::HundredThirtyFive => 0x03,
            LfoPhaseOffset::HundredEighty => 0x04,
            LfoPhaseOffset::TwoHundredTwentyFive => 0x05,
            LfoPhaseOffset::TwoHundredSeventy => 0x06,
            LfoPhaseOffset::ThreeHundredFifteen => 0x07,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ModSource {
    Off,
    AfterTouch,
    ModWheel,
    Velocity,
}

impl ModSource {
    pub fn as_byte(&self) -> u8 {
        match self {
            ModSource::Off => 0x00,
            ModSource::AfterTouch => 0x01,
            ModSource::ModWheel => 0x02,
            ModSource::Velocity => 0x03,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq)]
pub enum AssignOutOption {
    Osc1,
    Osc2,
    Velocity,
    ModWheel,
    AfterTouch,
}

impl AssignOutOption {
    pub fn as_byte(&self) -> u8 {
        match self {
            AssignOutOption::Osc1 => 0x00,
            AssignOutOption::Osc2 => 0x01,
            AssignOutOption::Velocity => 0x02,
            AssignOutOption::ModWheel => 0x03,
            AssignOutOption::AfterTouch => 0x04,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GlobalSetting {
    ParaphonicMode(ToggleOption),
    OscSync(ToggleOption),
    Osc1BlendMode(BlendMode),
    Osc2BlendMode(BlendMode),
    Osc1TunePotBypass(ToggleOption),
    Osc2TunePotBypass(ToggleOption),
    Osc1Range(OscRange),
    Osc2Range(OscRange),
    Osc2KeyTrack(KeyTrackMode),
    Osc1Autoglide(AutoglideSemitones),
    Osc2Autoglide(AutoglideSemitones),
    LfoBlendMode(BlendMode),
    LfoKeySync(ToggleOption),
    LfoOneShot(ToggleOption),
    LfoRetrigger(ToggleOption),
    LfoMidiSync(ToggleOption),
    LfoDepth(Percent),
    LfoShapeOrder(LfoIndex, LfoShape),
    LfoShapePhase(LfoIndex, LfoPhaseOffset),
    LfoResetOrder,
    VcfKeyTracking(ToggleOption),
    VcfModDepth(Percent),
    VcfModSource(ModSource),
    MidiChannel(Channel),
    DisableMidiDips(ToggleOption),
    PolyChainMode(ToggleOption),
    KeyRangeMute(ToggleOption),
    KeyRangeReset,
    AssignOut(AssignOutOption),
}

impl ByteBuilder for GlobalSetting {
    fn append_to(&self, buffer: &mut Vec<u8>) {
        match self {
            GlobalSetting::ParaphonicMode(t) => {
                buffer.push(0x0f);
                buffer.push(t.as_byte());
            }
            GlobalSetting::OscSync(t) => {
                buffer.push(0x0e);
                buffer.push(t.as_byte());
            }
            GlobalSetting::Osc1BlendMode(b) => {
                buffer.push(0x20);
                buffer.push(b.as_byte());
            }
            GlobalSetting::Osc2BlendMode(b) => {
                buffer.push(0x21);
                buffer.push(b.as_byte());
            }
            GlobalSetting::Osc1TunePotBypass(t) => {
                buffer.push(0x22);
                buffer.push(t.as_byte());
            }
            GlobalSetting::Osc2TunePotBypass(t) => {
                buffer.push(0x23);
                buffer.push(t.as_byte());
            }
            GlobalSetting::Osc1Range(r) => {
                buffer.push(0x26);
                buffer.push(r.as_byte());
            }
            GlobalSetting::Osc2Range(r) => {
                buffer.push(0x27);
                buffer.push(r.as_byte());
            }
            GlobalSetting::Osc2KeyTrack(k) => {
                buffer.push(0x2a);
                buffer.push(k.as_byte());
            }
            GlobalSetting::LfoBlendMode(b) => {
                buffer.push(0x30);
                buffer.push(b.as_byte());
            }
            GlobalSetting::LfoKeySync(t) => {
                buffer.push(0x37);
                buffer.push(t.as_byte());
            }
            GlobalSetting::LfoOneShot(t) => {
                buffer.push(0x31);
                buffer.push(t.as_byte());
            }
            GlobalSetting::LfoRetrigger(t) => {
                buffer.push(0x3b);
                buffer.push(t.as_byte());
            }
            GlobalSetting::LfoMidiSync(t) => {
                buffer.push(0x35);
                buffer.push(t.as_byte());
            }
            GlobalSetting::LfoResetOrder => {
                buffer.push(0x39);
                buffer.push(0x00);
            }
            GlobalSetting::VcfKeyTracking(t) => {
                buffer.push(0x11);
                buffer.push(t.as_byte());
            }
            GlobalSetting::MidiChannel(c) => {
                buffer.push(0x00);
                buffer.push(c.as_byte());
            }
            GlobalSetting::DisableMidiDips(t) => {
                buffer.push(0x0a);
                buffer.push(t.as_byte());
            }
            GlobalSetting::PolyChainMode(t) => {
                buffer.push(0x08);
                buffer.push(t.as_byte());
            }
            GlobalSetting::KeyRangeMute(t) => {
                buffer.push(0x0b);
                buffer.push(t.as_byte());
            }
            GlobalSetting::KeyRangeReset => {
                buffer.push(0x06);
                buffer.push(0x00);
            }
            GlobalSetting::LfoDepth(p) => {
                buffer.push(0x34);
                buffer.push(p.as_byte());
            }
            GlobalSetting::VcfModDepth(p) => {
                buffer.push(0x14);
                buffer.push(p.as_byte());
            }
            GlobalSetting::LfoShapeOrder(i, s) => {
                buffer.push(0x38);
                buffer.push(i.as_byte());
                buffer.push(s.as_byte());
            }
            GlobalSetting::Osc1Autoglide(s) => {
                buffer.push(0x24);
                buffer.push(s.as_byte());
            }
            GlobalSetting::Osc2Autoglide(s) => {
                buffer.push(0x25);
                buffer.push(s.as_byte());
            }
            GlobalSetting::LfoShapePhase(i, o) => {
                buffer.push(0x3a);
                buffer.push(i.as_byte());
                buffer.push(o.as_byte());
            }
            GlobalSetting::VcfModSource(m) => {
                buffer.push(0x12);
                buffer.push(m.as_byte());
            }
            GlobalSetting::AssignOut(o) => {
                buffer.push(0x04);
                buffer.push(o.as_byte());
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Channel {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
}

impl Channel {
    fn as_byte(&self) -> u8 {
        match self {
            Channel::One => 0x00,
            Channel::Two => 0x01,
            Channel::Three => 0x02,
            Channel::Four => 0x03,
            Channel::Five => 0x04,
            Channel::Six => 0x05,
            Channel::Seven => 0x06,
            Channel::Eight => 0x07,
            Channel::Nine => 0x08,
            Channel::Ten => 0x09,
            Channel::Eleven => 0x0a,
            Channel::Twelve => 0x0b,
            Channel::Thirteen => 0x0c,
            Channel::Fourteen => 0x0d,
            Channel::Fifteen => 0x0d,
            Channel::Sixteen => 0x0f,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeviceId {
    Channel(Channel),
    Multicast,
}

impl DeviceId {
    fn as_byte(&self) -> u8 {
        match &self {
            DeviceId::Channel(c) => c.as_byte(),
            DeviceId::Multicast => 0x7f,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NeutronMessage {
    SetGlobalSetting(DeviceId, GlobalSetting),
    RestoreGlobalSetting(DeviceId),
    CalibrationModeCommand(DeviceId),
    SoftwareVersionRequest(DeviceId),
    SoftwareVersionResponse(DeviceId, String),
    GlobalSettingUpdate(DeviceId, GlobalSetting),
}

impl Display for NeutronMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl NeutronMessage {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(SYSEX_MESSAGE_START);
        bytes.extend_from_slice(&BEHRINGER_MANUFACTURER);
        bytes.push(NEUTRON_DEVICE);
        match self {
            NeutronMessage::SetGlobalSetting(id, c) => {
                bytes.push(id.as_byte());
                bytes.push(0x0a);
                c.append_to(&mut bytes);
            }
            NeutronMessage::RestoreGlobalSetting(id) => {
                bytes.push(id.as_byte());
                bytes.push(0x0b)
            }
            NeutronMessage::CalibrationModeCommand(id) => {
                bytes.push(id.as_byte());
                bytes.push(0x10);
                // TODO
            }
            NeutronMessage::SoftwareVersionRequest(id) => {
                bytes.push(id.as_byte());
                bytes.push(0x73)
            }
            NeutronMessage::SoftwareVersionResponse(id, v) => {
                bytes.push(id.as_byte());
                bytes.push(0x74);
                bytes.push(COMMS_PROTOCOL_V1);
                bytes.extend_from_slice(v.as_bytes()); // TODO verify this
            }
            NeutronMessage::GlobalSettingUpdate(id, c) => {
                bytes.push(id.as_byte());
                bytes.push(0x5a);
                bytes.push(COMMS_PROTOCOL_V1);
                c.append_to(&mut bytes);
            }
        }
        bytes.push(SYSEX_EOX);
        bytes
    }
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

pub fn vcf_mode() -> Vec<u8> {
    // TODO param
    // 0x00 = 1 (1 High 2 Band)
    // 0x01 = 2 (1 Band 2 Low)
    // 0x02 = 3 (1 Low  2 High)
    wrap_message(vec![0x10, 0x00])
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
    wrapped_message.push(0x05); // TODO this is not in the documentation
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
