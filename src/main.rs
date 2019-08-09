use std::io;
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

    pub fn open(&mut self) -> Result<(), failure::Error> {
        let output = MidiOutput::new("Neutron").unwrap();
        let out_port = get_neutron_port(&output)?;
        self.midi_out = output
            .connect(out_port, "neutron")
            .map_err(|_| failure::err_msg("Could not connect MIDI out to Neutron"))
            .ok();

        let input = MidiInput::new("Neutron").unwrap();
        let in_port = get_neutron_port(&input)?;

        self.midi_in = input
            .connect(
                in_port,
                "neutron",
                |ts, msg, _| print!("{}", hex::encode(msg)),
                (),
            )
            .map_err(|_| failure::err_msg("Could not connect MIDI in to Neutron"))
            .ok();

        Ok(())
    }

    pub fn send_message(&mut self, message: Vec<u8>) -> Result<(), failure::Error> {
        match &mut self.midi_out {
            Some(out) => out.send(&message)
                .map_err(|e| failure::err_msg("")),
            None => Err(failure::err_msg("No connection established.")),
        }
    }
}

pub struct App<'a> {
    connection: Connection,
    midi_in_messages: Vec<&'a str>,
}

impl<'a> App<'a> {
    pub fn new() -> Result<App<'a>, failure::Error> {
        Ok(App {
            connection: Connection::new().into(),
            midi_in_messages: Vec::new().into(),
        })
    }

    pub fn connect(&mut self) -> Result<(), failure::Error> {
        if self.connection.midi_out.is_none() && self.connection.midi_in.is_none() {
            self.connection.open()?
        }
        Ok(())
    }
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let app = &mut App::new()?;
    app.midi_in_messages.push("Hello World!");
    app.connect();
    app.connection.send_message(protocol::maybe_request_state())?;

    loop {
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

            let events = app.midi_in_messages.iter().map(|&event| Text::raw(event));
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
                if key == Key::Char('q') {
                    break;
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
