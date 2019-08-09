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
use std::fmt::Display;

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
        message_sender_channel: Sender<MidiPacket>,
    ) -> Result<(), failure::Error> {
        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input)?;

        self.midi_in = input
            .connect(
                in_port,
                "neutron",
                move |ts, msg, _| { message_sender_channel.send(MidiPacket { timestamp: ts, message: msg.to_vec()}); },
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

pub struct MidiPacket {
    timestamp: u64,
    message: Vec<u8>,
}

impl Display for MidiPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3} - {}", self.timestamp as f64 / 100_000.0, hex::encode(self.message.clone()))
    }
}

pub struct State {
    // MIDI connection returns timestamps in microseconds beginning "sometime" in the past
    beginning_of_time: u64,
    // TODO will grow indefinitely, does it matter?
    midi_in_messages: Vec<MidiPacket>,
}

impl State {
    pub fn new() -> State {
        State {
            beginning_of_time: 0,
            midi_in_messages: Vec::new().into(),
        }
    }

    pub fn push(&mut self, mut packet: MidiPacket) {
        // Adjust timestamps to the first packets timestamp
        if self.beginning_of_time == 0 {
            self.beginning_of_time = packet.timestamp;
        }
        packet.timestamp -= self.beginning_of_time;
        self.midi_in_messages.push(packet);
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
    terminal.hide_cursor();
    terminal.clear();

    let key_events = Events::new();

    let (midi_in_sender, midi_in_receiver) = channel();
    let state = State::new();

    let app = &mut App::new(state)?;
    app.connection.register_midi_in_callback(midi_in_sender);

    loop {
        match midi_in_receiver.try_recv() {
            Ok(msg) => { app.state.push(msg.into()) }
            Err(_) => {}
        }
        terminal.draw(|mut frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);
            Block::default()
                .title("Rustron")
                .borders(Borders::ALL)
                .render(&mut frame, size);

            // Primitive scrolling logic
            let buffer_height = chunks[1].height;
            let message_count = app.state.midi_in_messages.len();
            let start_index = if message_count < buffer_height as usize { 0 } else { message_count - buffer_height as usize };
            let midi_messages = app.state.midi_in_messages[start_index..]
                .iter().map(|event| Text::raw(event.to_string()));
            List::new(midi_messages)
                .block(
                    Block::default()
                        .title("MIDI Sysex Input")
                        .borders(Borders::ALL),
                )
                .render(&mut frame, chunks[1]);
        });

        match key_events.next()? {
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
