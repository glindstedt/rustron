use nom::{
    bytes::complete::{tag, take_till1, take_while1},
    Err,
    error::ErrorKind,
    IResult,
    named,
    sequence::{delimited, preceded},
};

use crate::protocol::{BEHRINGER_MANUFACTURER, NEUTRON_DEVICE, NeutronMessage, SYSEX_EOX, SYSEX_MESSAGE_START, ToggleOption};
use crate::protocol::DeviceId::Multicast;
use crate::protocol::NeutronMessage::RestoreGlobalSetting;

fn sysex(input: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(
        tag(&[SYSEX_MESSAGE_START]),
        take_till1(|b| b == SYSEX_EOX),
        tag(&[SYSEX_EOX]),
    )(input)
}

fn behringer_packet(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(
        tag(&BEHRINGER_MANUFACTURER),
        take_while1(|_| true),
    )(input)
}

fn neutron_packet(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(
        tag(&[NEUTRON_DEVICE]),
        take_while1(|_| true),
    )(input)
}

fn parse(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let s = sysex(input)?;
    let b = behringer_packet(s.1)?;
    neutron_packet(b.1)
}

#[cfg(test)]
mod test {
    use nom::error::ErrorKind;
    use nom::IResult;

    use crate::parser::{behringer_packet, neutron_packet, parse, sysex};
    use crate::protocol::{BEHRINGER_MANUFACTURER, GlobalSetting, NEUTRON_DEVICE, SYSEX_EOX, SYSEX_MESSAGE_START, ToggleOption};
    use crate::protocol::Channel::One;
    use crate::protocol::DeviceId::{Channel, Multicast};
    use crate::protocol::GlobalSetting::ParaphonicMode;
    use crate::protocol::NeutronMessage::{GlobalSettingUpdate, SetGlobalSetting};
    use crate::protocol::ToggleOption::On;

    #[test]
    fn sysex_happy_path() {
        let pkg: [u8; 3] = [SYSEX_MESSAGE_START, 0x01, SYSEX_EOX];
        assert_eq!(sysex(&pkg), Ok((&[][..], &[0x01][..])));
    }

    #[test]
    fn no_eox_fails() {
        let pkg: [u8; 3] = [SYSEX_MESSAGE_START, 0x01, 0x00];
        assert_eq!(sysex(&pkg), Err(nom::Err::Error((&[][..], ErrorKind::Tag))));
    }

    #[test]
    fn no_message_start_fails() {
        let pkg: [u8; 3] = [0x00, 0x01, SYSEX_EOX];
        assert_eq!(sysex(&pkg), Err(nom::Err::Error((&pkg[..], ErrorKind::Tag))));
    }

    #[test]
    fn behringer_happy_path() {
        let pkg: [u8; 4] = [BEHRINGER_MANUFACTURER[0], BEHRINGER_MANUFACTURER[1], BEHRINGER_MANUFACTURER[2], 0x01];
        assert_eq!(behringer_packet(&pkg), Ok((&[][..], &[0x01][..])));
    }

    #[test]
    fn neutron_happy_path() {
        let pkg: [u8; 2] = [NEUTRON_DEVICE, 0x01];
        assert_eq!(neutron_packet(&pkg), Ok((&[][..], &[0x01][..])));
    }

    #[test]
    fn happy_path() {
        let pkg: [u8; 7] = [
            SYSEX_MESSAGE_START,
            BEHRINGER_MANUFACTURER[0],
            BEHRINGER_MANUFACTURER[1],
            BEHRINGER_MANUFACTURER[2],
            NEUTRON_DEVICE,
            0x01,
            SYSEX_EOX
        ];
        // TODO
//        assert_eq!(parse(&pkg), Ok((&[][..], &[0x01][..])));
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
            SYSEX_EOX
        ];
        let msg_turn_on_paraphonic = SetGlobalSetting(Multicast, ParaphonicMode(On)).as_bytes();
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
            SYSEX_EOX
        ];
        let ack_turn_on_paraphonic = GlobalSettingUpdate(Channel(One), ParaphonicMode(On)).as_bytes();
        assert_eq!(ack_turn_on_paraphonic_raw, ack_turn_on_paraphonic.as_slice())
    }
}
