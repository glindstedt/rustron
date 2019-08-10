use std::io;
use std::sync::mpsc::channel;
use std::thread::sleep;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;
use tui::widgets::{Block, Borders, List, Text, Widget};

use crate::events::{Event, Events};

mod events;
mod protocol;
mod midi;

pub struct State {
    // TODO this thimestamp value seems to be crap, maybe just throw away...
    // MIDI connection returns timestamps in microseconds beginning "sometime" in the past
    beginning_of_time: u64,
    // TODO will grow indefinitely, does it matter?
    midi_in_messages: Vec<midi::MidiPacket>,
}

impl State {
    pub fn new() -> State {
        State {
            beginning_of_time: 0,
            midi_in_messages: Vec::new().into(),
        }
    }

    pub fn push(&mut self, mut packet: midi::MidiPacket) {
        // Adjust timestamps to the first packets timestamp
        if self.beginning_of_time == 0 {
            self.beginning_of_time = packet.timestamp();
        }
        packet.set_timestamp(packet.timestamp() - self.beginning_of_time);
        self.midi_in_messages.push(packet);
    }
}

pub struct App {
    connection: midi::Connection,
    state: State,
}

impl App {
    pub fn new(state: State) -> Result<App, failure::Error> {
        Ok(App {
            connection: midi::Connection::new().into(),
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
    app.connection.register_midi_in_channel(midi_in_sender);

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
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}
