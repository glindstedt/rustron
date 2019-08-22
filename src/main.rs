use std::io;
use std::sync::mpsc::channel;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, Text, SelectableList, Widget};
use tui::Terminal;

use crate::events::{Event, Events};
use crate::protocol::Toggle;

mod events;
mod midi;
mod protocol;
mod parser;

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

pub const MENU_MAPPINGS: [(&str, fn() -> Vec<u8>); 35] = [
    ("Paraphonic mode On", || protocol::toggle_paraphonic_mode(Toggle::On)),
    ("Paraphonic mode Off", || protocol::toggle_paraphonic_mode(Toggle::Off)),
    ("OSC Sync On", || protocol::toggle_osc_sync(Toggle::On)),
    ("OSC Sync Off", || protocol::toggle_osc_sync(Toggle::Off)),

    ("OSC 1 blend mode Switch", || protocol::toggle_osc_1_blend_mode(Toggle::On)),
    ("OSC 1 blend mode Blend", || protocol::toggle_osc_1_blend_mode(Toggle::Off)),
    ("OSC 1 tune pot Bypass", || protocol::toggle_osc_1_tune_pot(Toggle::On)),
    ("OSC 1 tune pot Enable", || protocol::toggle_osc_1_tune_pot(Toggle::Off)),
    ("OSC 1 range 32", protocol::osc_1_range_32),
    ("OSC 1 range 16", protocol::osc_1_range_16),
    ("OSC 1 range 8", protocol::osc_1_range_8),
    ("OSC 1 range +/- 10 Oct", protocol::osc_1_range_pm_10_oct),

    ("OSC 2 blend mode Switch", || protocol::toggle_osc_2_blend_mode(Toggle::On)),
    ("OSC 2 blend mode Blend", || protocol::toggle_osc_2_blend_mode(Toggle::Off)),
    ("OSC 2 tune pot Bypass", || protocol::toggle_osc_2_tune_pot(Toggle::On)),
    ("OSC 2 tune pot Enable", || protocol::toggle_osc_2_tune_pot(Toggle::Off)),
    ("OSC 2 range 32", protocol::osc_2_range_32),
    ("OSC 2 range 16", protocol::osc_2_range_16),
    ("OSC 2 range 8", protocol::osc_2_range_8),
    ("OSC 2 range +/- 10 Oct", protocol::osc_2_range_pm_10_oct),

    ("OSC 2 key track Hold", || protocol::toggle_osc_2_key_track_hold(Toggle::On)),
    ("OSC 2 key track Track", || protocol::toggle_osc_2_key_track_hold(Toggle::Off)),

    ("LFO blend mode Switch", || protocol::toggle_lfo_blend_mode(Toggle::On)),
    ("LFO blend mode Blend", || protocol::toggle_lfo_blend_mode(Toggle::Off)),
    ("LFO key sync On", || protocol::toggle_lfo_key_sync(Toggle::On)),
    ("LFO key sync Off", || protocol::toggle_lfo_key_sync(Toggle::Off)),
    ("LFO one-shot On", || protocol::toggle_lfo_one_shot(Toggle::On)),
    ("LFO one-shot Off", || protocol::toggle_lfo_one_shot(Toggle::Off)),
    ("LFO retrigger On", || protocol::toggle_lfo_retrigger(Toggle::On)),
    ("LFO retrigger Off", || protocol::toggle_lfo_retrigger(Toggle::Off)),
    ("LFO midi sync On", || protocol::toggle_lfo_midi_sync(Toggle::On)),
    ("LFO midi sync Off", || protocol::toggle_lfo_midi_sync(Toggle::Off)),
    ("LFO reset order", protocol::lfo_reset_order),

    ("VCF key tracking On", || protocol::toggle_vcf_key_tracking(Toggle::On)),
    ("VCF key tracking Off", || protocol::toggle_vcf_key_tracking(Toggle::Off)),
];

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

    // let menu_items = ["Hello world!", "Foo Bar"];
    let menu_items: Vec<String> = MENU_MAPPINGS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    let mut menu_selection: usize = 0;

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

                SelectableList::default()
                    .block(Block::default())
                    .items(&menu_items)
                    .select(Some(menu_selection))
                    .highlight_symbol(">>")
                    .render(&mut frame, chunks[0]);

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
                Key::Char('P') => app.command(protocol::toggle_paraphonic_mode(Toggle::On).as_slice())?,
                Key::Char('p') => app.command(protocol::toggle_paraphonic_mode(Toggle::Off).as_slice())?,
                Key::Char('Y') => app.command(protocol::toggle_osc_sync(Toggle::On).as_slice())?,
                Key::Char('y') => app.command(protocol::toggle_osc_sync(Toggle::Off).as_slice())?,

                // Menu stuff
                Key::Char('\n') => app.command(MENU_MAPPINGS[menu_selection].1().as_slice())?,
                Key::Down => {
                    menu_selection = (menu_selection + 1) % menu_items.len();
                },
                Key::Up => {
                    if menu_selection == 0 {
                        menu_selection = menu_items.len() - 1;
                    } else {
                        menu_selection = menu_selection - 1;
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
