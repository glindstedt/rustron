use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take},
    combinator::{cut, map},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::protocol::GlobalSetting::{
    DisableMidiDips, KeyRangeMute, KeyRangeReset, LfoBlendMode, LfoDepth, LfoKeySync, LfoMidiSync,
    LfoOneShot, LfoResetOrder, LfoRetrigger, LfoShapeOrder, MidiChannel, Osc1Autoglide,
    Osc1BlendMode, Osc1Range, Osc1TunePotBypass, Osc2Autoglide, Osc2BlendMode, Osc2KeyTrack,
    Osc2Range, Osc2TunePotBypass, OscSync, ParaphonicMode, PolyChainMode, VcfKeyTracking,
    VcfModDepth,
};
use crate::protocol::NeutronMessage::{
    CalibrationModeCommand, GlobalSettingUpdate, RestoreGlobalSetting, SetGlobalSetting,
    SoftwareVersionRequest, SoftwareVersionResponse,
};
use crate::protocol::{
    AutoglideSemitones, BlendMode, Channel, DeviceId, GlobalSetting, KeyTrackMode, LfoIndex,
    LfoShape, NeutronMessage, OscRange, Percent, ToggleOption, COMMS_PROTOCOL_V1,
    NEUTRON_MESSAGE_HEADER, SYSEX_EOX,
};

fn toggle_option(input: &[u8]) -> IResult<&[u8], ToggleOption> {
    alt((
        map(tag(&[0x01]), |_| ToggleOption::On),
        map(tag(&[0x00]), |_| ToggleOption::Off),
    ))(input)
}

fn take1(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take(1usize)(input)
}

fn percent(input: &[u8]) -> IResult<&[u8], Percent> {
    map(take1, |p| Percent::from_byte(p[0]))(input)
}

fn blend_mode(input: &[u8]) -> IResult<&[u8], BlendMode> {
    alt((
        map(tag(&[0x01]), |_| BlendMode::Switch),
        map(tag(&[0x00]), |_| BlendMode::Blend),
    ))(input)
}

fn osc_range(input: &[u8]) -> IResult<&[u8], OscRange> {
    alt((
        map(tag(&[0x00]), |_| OscRange::ThirtyTwo),
        map(tag(&[0x01]), |_| OscRange::Sixteen),
        map(tag(&[0x02]), |_| OscRange::Eight),
        map(tag(&[0x03]), |_| OscRange::PlusMinusTen),
    ))(input)
}

fn autoglide_semitones(input: &[u8]) -> IResult<&[u8], AutoglideSemitones> {
    alt((
        alt((
            map(tag(&[0x00]), |_| AutoglideSemitones::MinusTwelve),
            map(tag(&[0x01]), |_| AutoglideSemitones::MinusEleven),
            map(tag(&[0x02]), |_| AutoglideSemitones::MinusTen),
            map(tag(&[0x03]), |_| AutoglideSemitones::MinusNine),
            map(tag(&[0x04]), |_| AutoglideSemitones::MinusEight),
            map(tag(&[0x05]), |_| AutoglideSemitones::MinusSeven),
            map(tag(&[0x06]), |_| AutoglideSemitones::MinusSix),
            map(tag(&[0x07]), |_| AutoglideSemitones::MinusFive),
            map(tag(&[0x08]), |_| AutoglideSemitones::MinusFour),
            map(tag(&[0x09]), |_| AutoglideSemitones::MinusThree),
            map(tag(&[0x0a]), |_| AutoglideSemitones::MinusTwo),
            map(tag(&[0x0b]), |_| AutoglideSemitones::MinusOne),
        )),
        alt((
            map(tag(&[0x0c]), |_| AutoglideSemitones::Zero),
            map(tag(&[0x0d]), |_| AutoglideSemitones::PlusOne),
            map(tag(&[0x0e]), |_| AutoglideSemitones::PlusTwo),
            map(tag(&[0x0f]), |_| AutoglideSemitones::PlusThree),
            map(tag(&[0x10]), |_| AutoglideSemitones::PlusFour),
            map(tag(&[0x11]), |_| AutoglideSemitones::PlusFive),
            map(tag(&[0x12]), |_| AutoglideSemitones::PlusSix),
            map(tag(&[0x13]), |_| AutoglideSemitones::PlusSeven),
            map(tag(&[0x14]), |_| AutoglideSemitones::PlusEight),
            map(tag(&[0x15]), |_| AutoglideSemitones::PlusNine),
            map(tag(&[0x16]), |_| AutoglideSemitones::PlusTen),
            map(tag(&[0x17]), |_| AutoglideSemitones::PlusEleven),
            map(tag(&[0x18]), |_| AutoglideSemitones::PlusTwelve),
        )),
    ))(input)
}

fn key_track_mode(input: &[u8]) -> IResult<&[u8], KeyTrackMode> {
    alt((
        map(tag(&[0x01]), |_| KeyTrackMode::Hold),
        map(tag(&[0x00]), |_| KeyTrackMode::Track),
    ))(input)
}

fn lfo_index(input: &[u8]) -> IResult<&[u8], LfoIndex> {
    alt((
        map(tag(&[0x00]), |_| LfoIndex::One),
        map(tag(&[0x01]), |_| LfoIndex::Two),
        map(tag(&[0x02]), |_| LfoIndex::Three),
        map(tag(&[0x03]), |_| LfoIndex::Four),
        map(tag(&[0x04]), |_| LfoIndex::Five),
    ))(input)
}

fn lfo_shape(input: &[u8]) -> IResult<&[u8], LfoShape> {
    alt((
        map(tag(&[0x00]), |_| LfoShape::Sine),
        map(tag(&[0x01]), |_| LfoShape::Triangle),
        map(tag(&[0x02]), |_| LfoShape::FallingSaw),
        map(tag(&[0x03]), |_| LfoShape::Square),
        map(tag(&[0x04]), |_| LfoShape::RisingSaw),
    ))(input)
}

fn global_setting(input: &[u8]) -> IResult<&[u8], GlobalSetting> {
    alt((
        alt((
            map(preceded(tag(&[0x0f]), toggle_option), |t| ParaphonicMode(t)),
            map(preceded(tag(&[0x0e]), toggle_option), |t| OscSync(t)),
            map(preceded(tag(&[0x20]), blend_mode), |b| Osc1BlendMode(b)),
            map(preceded(tag(&[0x21]), blend_mode), |b| Osc2BlendMode(b)),
            map(preceded(tag(&[0x22]), toggle_option), |t| {
                Osc1TunePotBypass(t)
            }),
            map(preceded(tag(&[0x23]), toggle_option), |t| {
                Osc2TunePotBypass(t)
            }),
            map(preceded(tag(&[0x26]), osc_range), |r| Osc1Range(r)),
            map(preceded(tag(&[0x27]), osc_range), |r| Osc2Range(r)),
            map(preceded(tag(&[0x2a]), key_track_mode), |m| Osc2KeyTrack(m)),
            map(preceded(tag(&[0x30]), blend_mode), |b| LfoBlendMode(b)),
            map(preceded(tag(&[0x37]), toggle_option), |t| LfoKeySync(t)),
            map(preceded(tag(&[0x31]), toggle_option), |t| LfoOneShot(t)),
            map(preceded(tag(&[0x3b]), toggle_option), |t| LfoRetrigger(t)),
            map(preceded(tag(&[0x35]), toggle_option), |t| LfoMidiSync(t)),
            map(preceded(tag(&[0x34]), percent), |p| LfoDepth(p)),
            map(tag(&[0x39, 0x00]), |_| LfoResetOrder),
            map(preceded(tag(&[0x11]), toggle_option), |t| VcfKeyTracking(t)),
            map(preceded(tag(&[0x14]), percent), |p| VcfModDepth(p)),
            map(preceded(tag(&[0x00]), channel), |c| MidiChannel(c)),
            map(preceded(tag(&[0x0a]), toggle_option), |t| {
                DisableMidiDips(t)
            }),
            map(preceded(tag(&[0x08]), toggle_option), |t| PolyChainMode(t)),
        )),
        alt((
            map(preceded(tag(&[0x24]), autoglide_semitones), |s| {
                Osc1Autoglide(s)
            }),
            map(preceded(tag(&[0x25]), autoglide_semitones), |s| {
                Osc2Autoglide(s)
            }),
            map(preceded(tag(&[0x0b]), toggle_option), |t| KeyRangeMute(t)),
            map(tag(&[0x06, 0x00]), |_| KeyRangeReset),
            map(
                preceded(tag(&[0x38]), pair(lfo_index, lfo_shape)),
                |(i, s)| LfoShapeOrder(i, s),
            ),
        )),
    ))(input)
}

fn channel(input: &[u8]) -> IResult<&[u8], Channel> {
    cut(alt((
        map(tag(&[0x00]), |_| Channel::One),
        map(tag(&[0x01]), |_| Channel::Two),
        map(tag(&[0x02]), |_| Channel::Three),
        map(tag(&[0x03]), |_| Channel::Four),
        map(tag(&[0x04]), |_| Channel::Five),
        map(tag(&[0x05]), |_| Channel::Six),
        map(tag(&[0x06]), |_| Channel::Seven),
        map(tag(&[0x07]), |_| Channel::Eight),
        map(tag(&[0x08]), |_| Channel::Nine),
        map(tag(&[0x09]), |_| Channel::Ten),
        map(tag(&[0x0a]), |_| Channel::Eleven),
        map(tag(&[0x0b]), |_| Channel::Twelve),
        map(tag(&[0x0c]), |_| Channel::Thirteen),
        map(tag(&[0x0d]), |_| Channel::Fourteen),
        map(tag(&[0x0e]), |_| Channel::Fifteen),
        map(tag(&[0x0f]), |_| Channel::Sixteen),
    )))(input)
}

fn device_id(input: &[u8]) -> IResult<&[u8], DeviceId> {
    alt((
        map(tag(&[0x7f]), |_| DeviceId::Multicast),
        map(channel, |c| DeviceId::Channel(c)),
    ))(input)
}

fn version(input: &[u8]) -> IResult<&[u8], String> {
    map(is_not([SYSEX_EOX]), |v| {
        String::from_utf8_lossy(v).into_owned()
    })(input)
}

pub fn neutron_message(input: &[u8]) -> IResult<&[u8], NeutronMessage> {
    delimited(
        tag(NEUTRON_MESSAGE_HEADER),
        alt((
            map(
                separated_pair(device_id, tag(&[0x0a]), global_setting),
                |(id, gs)| SetGlobalSetting(id, gs),
            ),
            map(terminated(device_id, tag(&[0x0b])), |id| {
                RestoreGlobalSetting(id)
            }),
            map(terminated(device_id, tag(&[0x73])), |id| {
                SoftwareVersionRequest(id)
            }),
            map(
                separated_pair(device_id, tag(&[0x74, COMMS_PROTOCOL_V1]), version),
                |(id, version)| SoftwareVersionResponse(id, version),
            ),
            map(
                separated_pair(device_id, tag(&[0x5a, COMMS_PROTOCOL_V1]), global_setting),
                |(id, gs)| GlobalSettingUpdate(id, gs),
            ),
        )),
        tag(&[SYSEX_EOX]),
    )(input)
}

#[cfg(test)]
mod test {
    use nom::error::ErrorKind;
    use nom::Err::Error;
    use nom::IResult;

    use crate::parser::{
        blend_mode, device_id, global_setting, key_track_mode, neutron_message, osc_range,
        toggle_option,
    };
    use crate::protocol::BlendMode::{Blend, Switch};
    use crate::protocol::GlobalSetting::{
        DisableMidiDips, KeyRangeMute, KeyRangeReset, LfoBlendMode, LfoDepth, LfoKeySync,
        LfoMidiSync, LfoOneShot, LfoResetOrder, LfoRetrigger, LfoShapeOrder, MidiChannel,
        Osc1Autoglide, Osc1BlendMode, Osc1Range, Osc1TunePotBypass, Osc2Autoglide, Osc2BlendMode,
        Osc2KeyTrack, Osc2Range, Osc2TunePotBypass, OscSync, ParaphonicMode, PolyChainMode,
        VcfKeyTracking, VcfModDepth,
    };
    use crate::protocol::KeyTrackMode::Track;
    use crate::protocol::NeutronMessage::{
        CalibrationModeCommand, GlobalSettingUpdate, RestoreGlobalSetting, SetGlobalSetting,
        SoftwareVersionRequest, SoftwareVersionResponse,
    };
    use crate::protocol::OscRange::{PlusMinusTen, ThirtyTwo};
    use crate::protocol::ToggleOption::{Off, On};
    use crate::protocol::{
        AutoglideSemitones, BlendMode, ByteBuilder, Channel, DeviceId, GlobalSetting, KeyTrackMode,
        LfoIndex, LfoShape, OscRange, Percent, ToggleOption, BEHRINGER_MANUFACTURER,
        NEUTRON_DEVICE, SYSEX_EOX, SYSEX_MESSAGE_START,
    };

    #[test]
    fn test_toggle_option() {
        assert_eq!(
            toggle_option(&[ToggleOption::On.as_byte()]),
            Ok((&[][..], ToggleOption::On))
        );
        assert_eq!(
            toggle_option(&[ToggleOption::Off.as_byte()]),
            Ok((&[][..], ToggleOption::Off))
        );
    }

    #[test]
    fn test_blend_mode() {
        assert_eq!(
            blend_mode(&[BlendMode::Switch.as_byte()]),
            Ok((&[][..], BlendMode::Switch))
        );
        assert_eq!(
            blend_mode(&[BlendMode::Blend.as_byte()]),
            Ok((&[][..], BlendMode::Blend))
        );
    }

    #[test]
    fn test_osc_range() {
        assert_eq!(
            osc_range(&[OscRange::ThirtyTwo.as_byte()]),
            Ok((&[][..], OscRange::ThirtyTwo))
        );
        assert_eq!(
            osc_range(&[OscRange::Sixteen.as_byte()]),
            Ok((&[][..], OscRange::Sixteen))
        );
        assert_eq!(
            osc_range(&[OscRange::Eight.as_byte()]),
            Ok((&[][..], OscRange::Eight))
        );
        assert_eq!(
            osc_range(&[OscRange::PlusMinusTen.as_byte()]),
            Ok((&[][..], OscRange::PlusMinusTen))
        );
    }

    #[test]
    fn test_key_track_mode() {
        assert_eq!(
            key_track_mode(&[KeyTrackMode::Hold.as_byte()]),
            Ok((&[][..], KeyTrackMode::Hold))
        );
        assert_eq!(
            key_track_mode(&[KeyTrackMode::Track.as_byte()]),
            Ok((&[][..], KeyTrackMode::Track))
        );
    }

    // Test helper
    fn to_vec(gs: GlobalSetting) -> Vec<u8> {
        let mut buf = Vec::new();
        gs.append_to(&mut buf);
        buf
    }

    #[test]
    fn test_global_setting() {
        assert_eq!(
            global_setting(to_vec(ParaphonicMode(On)).as_slice()),
            Ok((&[][..], ParaphonicMode(On)))
        );
        assert_eq!(
            global_setting(to_vec(OscSync(Off)).as_slice()),
            Ok((&[][..], OscSync(Off)))
        );
        assert_eq!(
            global_setting(to_vec(Osc1BlendMode(Blend)).as_slice()),
            Ok((&[][..], Osc1BlendMode(Blend)))
        );
        assert_eq!(
            global_setting(to_vec(Osc2BlendMode(Switch)).as_slice()),
            Ok((&[][..], Osc2BlendMode(Switch)))
        );
        assert_eq!(
            global_setting(to_vec(Osc1TunePotBypass(On)).as_slice()),
            Ok((&[][..], Osc1TunePotBypass(On)))
        );
        assert_eq!(
            global_setting(to_vec(Osc2TunePotBypass(On)).as_slice()),
            Ok((&[][..], Osc2TunePotBypass(On)))
        );
        assert_eq!(
            global_setting(to_vec(Osc1Range(ThirtyTwo)).as_slice()),
            Ok((&[][..], Osc1Range(ThirtyTwo)))
        );
        assert_eq!(
            global_setting(to_vec(Osc2Range(PlusMinusTen)).as_slice()),
            Ok((&[][..], Osc2Range(PlusMinusTen)))
        );
        assert_eq!(
            global_setting(to_vec(Osc2KeyTrack(Track)).as_slice()),
            Ok((&[][..], Osc2KeyTrack(Track)))
        );
        assert_eq!(
            global_setting(to_vec(Osc1Autoglide(AutoglideSemitones::MinusThree)).as_slice()),
            Ok((&[][..], Osc1Autoglide(AutoglideSemitones::MinusThree)))
        );
        assert_eq!(
            global_setting(to_vec(Osc2Autoglide(AutoglideSemitones::PlusEight)).as_slice()),
            Ok((&[][..], Osc2Autoglide(AutoglideSemitones::PlusEight)))
        );
        assert_eq!(
            global_setting(to_vec(LfoBlendMode(Blend)).as_slice()),
            Ok((&[][..], LfoBlendMode(Blend)))
        );
        assert_eq!(
            global_setting(to_vec(LfoKeySync(On)).as_slice()),
            Ok((&[][..], LfoKeySync(On)))
        );
        assert_eq!(
            global_setting(to_vec(LfoOneShot(On)).as_slice()),
            Ok((&[][..], LfoOneShot(On)))
        );
        assert_eq!(
            global_setting(to_vec(LfoRetrigger(On)).as_slice()),
            Ok((&[][..], LfoRetrigger(On)))
        );
        assert_eq!(
            global_setting(to_vec(LfoMidiSync(On)).as_slice()),
            Ok((&[][..], LfoMidiSync(On)))
        );
        assert_eq!(
            global_setting(to_vec(LfoDepth(Percent::from_percentage(50))).as_slice()),
            Ok((&[][..], LfoDepth(Percent::from_byte(31))))
        );
        assert_eq!(
            global_setting(to_vec(LfoShapeOrder(LfoIndex::Two, LfoShape::RisingSaw)).as_slice()),
            Ok((&[][..], LfoShapeOrder(LfoIndex::Two, LfoShape::RisingSaw)))
        );
        assert_eq!(
            global_setting(to_vec(LfoResetOrder).as_slice()),
            Ok((&[][..], LfoResetOrder))
        );
        assert_eq!(
            global_setting(to_vec(VcfKeyTracking(On)).as_slice()),
            Ok((&[][..], VcfKeyTracking(On)))
        );
        assert_eq!(
            global_setting(to_vec(VcfModDepth(Percent::from_percentage(50))).as_slice()),
            Ok((&[][..], VcfModDepth(Percent::from_byte(31))))
        );
        assert_eq!(
            global_setting(to_vec(MidiChannel(Channel::Thirteen)).as_slice()),
            Ok((&[][..], MidiChannel(Channel::Thirteen)))
        );
        assert_eq!(
            global_setting(to_vec(DisableMidiDips(On)).as_slice()),
            Ok((&[][..], DisableMidiDips(On)))
        );
        assert_eq!(
            global_setting(to_vec(PolyChainMode(On)).as_slice()),
            Ok((&[][..], PolyChainMode(On)))
        );
        assert_eq!(
            global_setting(to_vec(KeyRangeMute(On)).as_slice()),
            Ok((&[][..], KeyRangeMute(On)))
        );
        assert_eq!(
            global_setting(to_vec(KeyRangeReset).as_slice()),
            Ok((&[][..], KeyRangeReset))
        );
    }

    #[test]
    fn test_device_id() {
        assert_eq!(
            device_id(&[0x00]),
            Ok((&[][..], DeviceId::Channel(Channel::One)))
        );
        assert_eq!(
            device_id(&[0x0f]),
            Ok((&[][..], DeviceId::Channel(Channel::Sixteen)))
        );
        assert_eq!(device_id(&[0x7f]), Ok((&[][..], DeviceId::Multicast)));
        match device_id(&[0x10]) {
            Ok(_) => panic!("Invalid DeviceId should fail"),
            _ => (),
        }
    }

    #[test]
    fn test_neutron_message() {
        assert_eq!(
            neutron_message(
                SetGlobalSetting(DeviceId::Multicast, ParaphonicMode(On))
                    .as_bytes()
                    .as_slice()
            ),
            Ok((
                &[][..],
                SetGlobalSetting(DeviceId::Multicast, ParaphonicMode(On))
            ))
        );
        assert_eq!(
            neutron_message(
                RestoreGlobalSetting(DeviceId::Channel(Channel::One))
                    .as_bytes()
                    .as_slice()
            ),
            Ok((
                &[][..],
                RestoreGlobalSetting(DeviceId::Channel(Channel::One))
            ))
        );
        // TODO
        // assert_eq!(
        //     neutron_message(CalibrationModeCommand(DeviceId::Multicast).as_bytes().as_slice()),
        //     Ok((&[][..], CalibrationModeCommand(DeviceId::Multicast)))
        // );
        assert_eq!(
            neutron_message(
                SoftwareVersionRequest(DeviceId::Multicast)
                    .as_bytes()
                    .as_slice()
            ),
            Ok((&[][..], SoftwareVersionRequest(DeviceId::Multicast)))
        );
        assert_eq!(
            neutron_message(
                SoftwareVersionResponse(DeviceId::Multicast, String::from("1.2.3"))
                    .as_bytes()
                    .as_slice()
            ),
            Ok((
                &[][..],
                SoftwareVersionResponse(DeviceId::Multicast, String::from("1.2.3"))
            ))
        );
        assert_eq!(
            neutron_message(
                GlobalSettingUpdate(DeviceId::Multicast, ParaphonicMode(On))
                    .as_bytes()
                    .as_slice()
            ),
            Ok((
                &[][..],
                GlobalSettingUpdate(DeviceId::Multicast, ParaphonicMode(On))
            ))
        );
    }

    #[test]
    fn test_command() {
        let turn_on_paraphonic_raw: [u8; 10] = [
            SYSEX_MESSAGE_START,
            BEHRINGER_MANUFACTURER[0],
            BEHRINGER_MANUFACTURER[1],
            BEHRINGER_MANUFACTURER[2],
            NEUTRON_DEVICE,
            0x7f,
            0x0a,
            0x0f, // paraphonic mode
            0x01, // value
            SYSEX_EOX,
        ];
        let msg_turn_on_paraphonic =
            SetGlobalSetting(DeviceId::Multicast, ParaphonicMode(On)).as_bytes();
        assert_eq!(turn_on_paraphonic_raw, msg_turn_on_paraphonic.as_slice());
        //format_command(NeutronToggleCommand::ParaphonicMode(ToggleOption::On));

        let ack_turn_on_paraphonic_raw: [u8; 11] = [
            SYSEX_MESSAGE_START,
            BEHRINGER_MANUFACTURER[0],
            BEHRINGER_MANUFACTURER[1],
            BEHRINGER_MANUFACTURER[2],
            NEUTRON_DEVICE,
            0x00,
            0x5a,
            0x01,
            0x0f, // paraphonic mode
            0x01, // value
            SYSEX_EOX,
        ];
        let ack_turn_on_paraphonic =
            GlobalSettingUpdate(DeviceId::Channel(Channel::One), ParaphonicMode(On)).as_bytes();
        assert_eq!(
            ack_turn_on_paraphonic_raw,
            ack_turn_on_paraphonic.as_slice()
        )
    }
}
