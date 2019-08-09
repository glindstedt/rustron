use std::io;
use std::thread::sleep;
use std::time::Duration;
use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, Text, Widget};
use tui::Terminal;

use crate::events::{Event, Events};
use crate::protocol::wrap_message;
use midir::{
    ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection, PortInfoError,
};

mod events;
mod protocol;

pub struct Connection {
    midi_out: Option<MidiOutputConnection>,
    midi_in: Option<MidiInputConnection<()>>,
}

pub fn handle_midi_message(timestamp: u64, message: &[u8]) {
    print!("{}", hex::encode(message))
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
                |ts, msg, _| handle_midi_message(ts, msg),
                (),
            )
            .map_err(|_| failure::err_msg("Could not connect MIDI in to Neutron"))
            .ok();

        Ok(())
    }
}

pub struct App<'a> {
    //    midi_out: MidiOutputConnection,
    midi_in_messages: Vec<&'a str>,
}

impl<'a> App<'a> {
    pub fn new() -> Result<App<'a>, failure::Error> {
        Ok(App {
            midi_in_messages: Vec::new().into(),
        })
    }
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut conn = Connection::new();
    conn.open()?;
    conn.midi_out
        .unwrap()
        .send(&protocol::maybe_request_state())?;
    let app = &mut App::new()?;
    app.midi_in_messages.push("Hello World!");

    // communicate();
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

//pub fn communicate() -> Result<&'static str, Box<dyn Error>> {
//    let output = MidiOutput::new("Neutron").unwrap();
//
//    let out_port = get_neutron_port(&output)?;
//
//    let input = MidiInput::new("Neutron").unwrap();
//
//    let in_port = get_neutron_port(&input)?;
//
//    let mut conn_out = output.connect(out_port, "neutron")?;
//
//    let mut result = conn_out.send(&turn_on_paraphonic_mode());
//    result.map_err(|e| println!("{}", e));
//    sleep(Duration::from_millis(5000));
//    result = conn_out.send(&turn_off_paraphonic_mode());
//    result.map_err(|e| println!("{}", e));
//
//    Ok("foo")
//}

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
