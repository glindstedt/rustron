use std::error;
use std::sync::mpsc::Sender;

use midir::{
    MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection, PortInfoError, SendError,
};

pub struct MidiConnection {
    // TODO what about closing connections?
    midi_out: Option<MidiOutputConnection>,
    midi_in: Option<MidiInputConnection<()>>,
}

impl MidiConnection {
    pub fn new() -> MidiConnection {
        MidiConnection {
            midi_out: None,
            midi_in: None,
        }
    }

    fn connect_midi_out(&mut self) -> Result<(), Box<dyn error::Error>> {
        let output = MidiOutput::new("Neutron").unwrap();
        let out_port = get_neutron_port(&output);
        return out_port.and_then(|port_number| {
            self.midi_out = output.connect(port_number, "neutron").ok();
            Ok(())
        });
    }

    pub fn register_midi_in_channel(
        &mut self,
        message_sender_channel: Sender<Vec<u8>>,
    ) -> Result<(), Box<dyn error::Error>> {
        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input);

        return in_port.and_then(|port_number| {
            self.midi_in = input
                .connect(
                    port_number,
                    "neutron",
                    move |_, msg, _| {
                        // TODO panic on Err for now
                        message_sender_channel.send(msg.to_vec()).unwrap();
                    },
                    (),
                )
                .ok();
            Ok(())
        });
    }

    pub fn send_message(&mut self, message: &[u8]) -> Result<(), SendError> {
        if self.midi_out.is_none() {
            self.connect_midi_out();
        }
        match &mut self.midi_out {
            Some(out) => out.send(message),
            None => Err(SendError::Other("No connection established.")),
        }
    }
}

// ========================== OTHER STUFF ======================
trait Neutron {
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

fn get_neutron_port(midi_output: &dyn Neutron) -> Result<usize, Box<dyn error::Error>> {
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
        None => Err(Box::from("Could not find Neutron.")),
    }
}
