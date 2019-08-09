use std::io;
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;
use std::time::Duration;

use midir::{
    ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection, PortInfoError,
};
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;
use tui::widgets::{Block, Borders, List, Text, Widget};

use crate::events::{Event, Events};
use crate::protocol::wrap_message;

mod events;
mod protocol;

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

    pub fn register_midi_in_callback(
        &mut self,
        message_sender_channel: Sender<String>,
    ) -> Result<(), failure::Error> {
        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input)?;

        self.midi_in = input
            .connect(
                in_port,
                "neutron",
                move |ts, msg, _| { message_sender_channel.send(hex::encode(msg)); },
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

pub struct State {
    midi_in_messages: Vec<String>,
}

impl State {
    pub fn new() -> State {
        State {
            midi_in_messages: Vec::new().into(),
        }
    }
}

pub struct App {
    connection: Connection,
    state: State,
}

impl App {
    pub fn new(state: State) -> Result<App, failure::Error> {
        Ok(App {
            connection: Connection::new().into(),
            state,
        })
    }
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let (midi_in_sender, midi_in_receiver) = channel();
    let state = State::new();

    let app = &mut App::new(state)?;
    app.state.midi_in_messages.push(String::from("Hello World!"));
    app.connection.register_midi_in_callback(midi_in_sender);

    loop {
        match midi_in_receiver.try_recv() {
            Ok(msg) => { app.state.midi_in_messages.push(msg.into()) }
            Err(_) => {}
        }
        terminal.draw(|mut frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);
            Block::default()
                .title("Some Title")
                .borders(Borders::ALL)
                .render(&mut frame, size);

            let events = app.state.midi_in_messages
                .iter().map(|event| Text::raw(event));
            List::new(events)
                .block(
                    Block::default()
                        .title("MIDI Sysex Input")
                        .borders(Borders::ALL),
                )
                .render(&mut frame, chunks[1]);
        });

        match events.next()? {
            Event::Input(key) => {
                match key {
                    Key::Char('q') => break,
                    Key::Char('s') => app.connection.send_message(protocol::maybe_request_state())?,
                    Key::Char('P') => app.connection.send_message(protocol::turn_on_paraphonic_mode())?,
                    Key::Char('p') => app.connection.send_message(protocol::turn_off_paraphonic_mode())?,
                    _ => {},
                }
            }
            _ => {}
        }
    }
    Ok(())
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
                println!("FOUND IT: {}", i);
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
