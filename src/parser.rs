use nom::{
    bytes::complete::{tag, take_till1, take_while1},
    Err,
    error::ErrorKind,
    IResult,
    named,
    sequence::{delimited, preceded},
};

use crate::protocol::{BEHRINGER_MANUFACTURER, SYSEX_EOX, SYSEX_MESSAGE_START, PROBABLY_NEUTRON_DEVICE};

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
        tag(&[PROBABLY_NEUTRON_DEVICE]),
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

    use crate::parser::{behringer_packet, parse, sysex, neutron_packet};
    use crate::protocol::{BEHRINGER_MANUFACTURER, SYSEX_EOX, SYSEX_MESSAGE_START, PROBABLY_NEUTRON_DEVICE};

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
        let pkg: [u8; 2] = [PROBABLY_NEUTRON_DEVICE, 0x01];
        assert_eq!(neutron_packet(&pkg), Ok((&[][..], &[0x01][..])));
    }

    #[test]
    fn happy_path() {
        let pkg: [u8; 7] = [
            SYSEX_MESSAGE_START,
            BEHRINGER_MANUFACTURER[0],
            BEHRINGER_MANUFACTURER[1],
            BEHRINGER_MANUFACTURER[2],
            PROBABLY_NEUTRON_DEVICE,
            0x01,
            SYSEX_EOX
        ];
        assert_eq!(parse(&pkg), Ok((&[][..], &[0x01][..])));
    }
}
