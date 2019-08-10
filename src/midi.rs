use std::fmt::Display;
use std::sync::mpsc::Sender;

use midir::{
    ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection, PortInfoError,
};

use crate::protocol;

pub struct MidiPacket {
    timestamp: u64,
    message: Vec<u8>,
}

impl MidiPacket {
    pub fn new(timestamp: u64, message: &[u8]) -> MidiPacket {
        MidiPacket {
            timestamp,
            message: message.to_vec(),
        }
    }
    pub fn timestamp(&self) -> u64 { self.timestamp }
    pub fn set_timestamp(&mut self, timestamp: u64) { self.timestamp = timestamp }
    pub fn message(&self) -> &[u8] { self.message.as_slice() }
}

impl Display for MidiPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format into 8 byte batches
        let mut packet_string = String::new();
        if self.message.starts_with(&protocol::PACKET_HEADER)
            && self.message.ends_with([protocol::SYSEX_EOX].as_ref()) {
            packet_string.push_str("B[ ");
            packet_string.push_str(hex::encode(&self.message[protocol::PACKET_HEADER.len()..]).as_str());
            packet_string.push_str(" ]");
        } else {
            packet_string.push_str(hex::encode(&self.message).as_str());
        }
        write!(f, "{:.3} - {}", self.timestamp as f64 / 100_000.0, packet_string)
    }
}


pub struct Connection {
    // TODO what about closing connections?
    midi_out: Option<MidiOutputConnection>,
    midi_in: Option<MidiInputConnection<()>>,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            midi_out: None,
            midi_in: None,
        }
    }

    fn connect_midi_out(&mut self) -> Result<(), failure::Error> {
        let output = MidiOutput::new("Neutron").unwrap();
        let out_port = get_neutron_port(&output)?;
        self.midi_out = output
            .connect(out_port, "neutron")
            .ok();
        Ok(())
    }

    pub fn register_midi_in_channel(
        &mut self,
        message_sender_channel: Sender<MidiPacket>,
    ) -> Result<(), failure::Error> {
        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input)?;

        self.midi_in = input
            .connect(
                in_port,
                "neutron",
                move |ts, msg, _| { message_sender_channel.send(MidiPacket { timestamp: ts, message: msg.to_vec() }); },
                (),
            )
            .map_err(|e| failure::err_msg(e.to_string()))
            .ok();

        Ok(())
    }

    pub fn send_message(&mut self, message: Vec<u8>) -> Result<(), failure::Error> {
        if self.midi_out.is_none() {
            self.connect_midi_out()?;
        }
        match &mut self.midi_out {
            Some(out) => out.send(&message)
                .map_err(|e| failure::err_msg(e.to_string())),
            None => Err(failure::err_msg("No connection established.")),
        }
    }
}

// ========================== OTHER STUFF ======================
pub trait Neutron {
    fn port_count(&self) -> usize;
    fn port_name(&self, port_number: usize) -> Result<String, PortInfoError>;
}

impl Neutron for MidiOutput {
    fn port_count(&self) -> usize {
        self.port_count()
    }

    fn port_name(&self, port_number: usize) -> Result<String, PortInfoError> {
        self.port_name(port_number)
    }
}

impl Neutron for MidiInput {
    fn port_count(&self) -> usize {
        self.port_count()
    }
    fn port_name(&self, port_number: usize) -> Result<String, PortInfoError> {
        self.port_name(port_number)
    }
}

pub fn get_neutron_port(midi_output: &dyn Neutron) -> Result<usize, failure::Error> {
    let mut out_port: Option<usize> = None;
    for i in 0..midi_output.port_count() {
        match midi_output.port_name(i).unwrap().starts_with("Neutron") {
            true => {
                out_port = Some(i);
                break;
            }
            _ => (),
        }
    }
    match out_port {
        Some(i) => Ok(i),
        None => Err(failure::err_msg("Could not find Neutron.")),
    }
}
