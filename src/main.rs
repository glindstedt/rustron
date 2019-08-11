use std::io;
use std::sync::mpsc::channel;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, Text, Widget};
use tui::Terminal;

use crate::events::{Event, Events};

mod events;
mod midi;
mod protocol;

pub struct State {
    // TODO will grow indefinitely, does it matter?
    midi_in_messages: Vec<midi::MidiPacket>,
}

impl State {
    pub fn new() -> State {
        State {
            midi_in_messages: Vec::new().into(),
        }
    }

    pub fn push(&mut self, packet: midi::MidiPacket) {
        self.midi_in_messages.push(packet);
    }
}

pub struct App {
    connection: midi::MidiConnection,
    state: State,
    command_history: Vec<midi::MidiPacket>,
}

impl App {
    pub fn new(state: State) -> Result<App, failure::Error> {
        Ok(App {
            connection: midi::MidiConnection::new().into(),
            state,
            command_history: Vec::new().into(),
        })
    }

    pub fn command(&mut self, message: &[u8]) -> Result<(), failure::Error> {
        self.command_history.push(midi::MidiPacket::new(message));
        self.connection.send_message(message)
    }
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let key_events = Events::new();

    let (midi_in_sender, midi_in_receiver) = channel();
    let state = State::new();

    let app = &mut App::new(state)?;
    app.connection.register_midi_in_channel(midi_in_sender)?;

    loop {
        match midi_in_receiver.try_recv() {
            Ok(msg) => app.state.push(msg.into()),
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

            {
                // Left half
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[0]);
                let buffer_height = chunks[1].height;
                let message_count = app.command_history.len();
                let start_index = if message_count < buffer_height as usize {
                    0
                } else {
                    message_count - buffer_height as usize
                };
                let command_history = app.command_history[start_index..]
                    .iter()
                    .map(|event| Text::raw(event.to_string()));
                List::new(command_history)
                    .block(
                        Block::default()
                            .title("Command History")
                            .borders(Borders::ALL),
                    )
                    .render(&mut frame, chunks[1]);
            }

            // Primitive scrolling logic
            let buffer_height = chunks[1].height;
            let message_count = app.state.midi_in_messages.len();
            let start_index = if message_count < buffer_height as usize {
                0
            } else {
                message_count - buffer_height as usize
            };
            let midi_messages = app.state.midi_in_messages[start_index..]
                .iter()
                .map(|event| Text::raw(event.to_string()));
            List::new(midi_messages)
                .block(
                    Block::default()
                        .title("MIDI Sysex Input")
                        .borders(Borders::ALL),
                )
                .render(&mut frame, chunks[1]);
        })?;

        match key_events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => break,
                Key::Char('s') => app.command(protocol::maybe_request_state().as_slice())?,
                Key::Char('P') => app.command(protocol::turn_on_paraphonic_mode().as_slice())?,
                Key::Char('p') => app.command(protocol::turn_off_paraphonic_mode().as_slice())?,
                Key::Char('Y') => app.command(protocol::osc_sync_on().as_slice())?,
                Key::Char('y') => app.command(protocol::osc_sync_off().as_slice())?,
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
