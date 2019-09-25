use std::sync::mpsc::Sender;

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection, PortInfoError};

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

    fn connect_midi_out(&mut self) -> Result<(), failure::Error> {
        let output = MidiOutput::new("Neutron").unwrap();
        let out_port = get_neutron_port(&output);
        return match out_port {
            Ok(port_number) => {
                self.midi_out = output.connect(port_number, "neutron").ok();
                Ok(())
            }
            Err(error) => Err(error),
        };
    }

    pub fn register_midi_in_channel(
        &mut self,
        message_sender_channel: Sender<Vec<u8>>,
    ) -> Result<(), failure::Error> {
        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input);

        return match in_port {
            Ok(port_number) => {
                self.midi_in = input
                    .connect(
                        port_number,
                        "neutron",
                        move |_, msg, _| {
                            message_sender_channel.send(msg.to_vec());
                        },
                        (),
                    )
                    .map_err(|e| failure::err_msg(e.to_string()))
                    .ok();
                Ok(())
            }
            Err(error) => Err(error),
        };
    }

    pub fn send_message(&mut self, message: &[u8]) -> Result<(), failure::Error> {
        if self.midi_out.is_none() {
            self.connect_midi_out();
        }
        match &mut self.midi_out {
            Some(out) => out
                .send(message)
                .map_err(|e| failure::err_msg(e.to_string())),
            None => Err(failure::err_msg("No connection established.")),
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

fn get_neutron_port(midi_output: &dyn Neutron) -> Result<usize, failure::Error> {
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
