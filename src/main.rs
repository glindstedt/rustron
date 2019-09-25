use std::io;
use std::sync::mpsc;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};
use tui::Terminal;

use crate::events::{Event, Events};
use rustron_lib::parser::neutron_message;
use rustron_lib::protocol;
use rustron_lib::protocol::{
    BlendMode::{Blend, Switch},
    DeviceId::Multicast,
    GlobalSetting,
    GlobalSetting::{
        LfoBlendMode, LfoKeySync, LfoMidiSync, LfoOneShot, LfoResetOrder, LfoRetrigger,
        Osc1BlendMode, Osc1Range, Osc1TunePotBypass, Osc2BlendMode, Osc2KeyTrack, Osc2Range,
        Osc2TunePotBypass, OscSync, ParaphonicMode, VcfKeyTracking,
    },
    KeyTrackMode::{Hold, Track},
    NeutronMessage::SetGlobalSetting,
    OscRange::{Eight, PlusMinusTen, Sixteen, ThirtyTwo},
    ToggleOption::{Off, On},
};

mod events;
mod midi;

pub struct State {
    // TODO will grow indefinitely, does it matter?
    midi_in_messages: Vec<Vec<u8>>,
}

impl State {
    pub fn new() -> State {
        State {
            midi_in_messages: Vec::new().into(),
        }
    }

    pub fn push(&mut self, packet: Vec<u8>) {
        self.midi_in_messages.push(packet);
    }
}

pub struct App {
    connection: midi::MidiConnection,
    state: State,
    command_history: Vec<String>,
}

impl App {
    pub fn new(state: State) -> Result<App, failure::Error> {
        Ok(App {
            connection: midi::MidiConnection::new().into(),
            state,
            command_history: Vec::new().into(),
        })
    }

    pub fn command(&mut self, message: &[u8]) {
        match neutron_message(message) {
            Ok((_, msg)) => {
                self.command_history.push(msg.to_string());
            }
            Err(_) => self.command_history.push(hex::encode(message)),
        }
        if let Err(error) = self.connection.send_message(message) {
            self.command_history.push(format!("{}", error))
        };
    }
}

pub const MENU_MAPPINGS: [(&str, GlobalSetting); 35] = [
    ("Paraphonic mode On", ParaphonicMode(On)),
    ("Paraphonic mode Off", ParaphonicMode(Off)),
    ("OSC Sync On", OscSync(On)),
    ("OSC Sync Off", OscSync(Off)),
    ("OSC 1 blend mode Switch", Osc1BlendMode(Switch)),
    ("OSC 1 blend mode Blend", Osc1BlendMode(Blend)),
    ("OSC 1 tune pot Bypass", Osc1TunePotBypass(On)),
    ("OSC 1 tune pot Enable", Osc1TunePotBypass(Off)),
    ("OSC 1 range 32", Osc1Range(ThirtyTwo)),
    ("OSC 1 range 16", Osc1Range(Sixteen)),
    ("OSC 1 range 8", Osc1Range(Eight)),
    ("OSC 1 range +/- 10 Oct", Osc1Range(PlusMinusTen)),
    ("OSC 2 blend mode Switch", Osc2BlendMode(Switch)),
    ("OSC 2 blend mode Blend", Osc2BlendMode(Blend)),
    ("OSC 2 tune pot Bypass", Osc2TunePotBypass(On)),
    ("OSC 2 tune pot Enable", Osc2TunePotBypass(Off)),
    ("OSC 2 range 32", Osc2Range(ThirtyTwo)),
    ("OSC 2 range 16", Osc2Range(Sixteen)),
    ("OSC 2 range 8", Osc2Range(Eight)),
    ("OSC 2 range +/- 10 Oct", Osc2Range(PlusMinusTen)),
    ("OSC 2 key track Hold", Osc2KeyTrack(Hold)),
    ("OSC 2 key track Track", Osc2KeyTrack(Track)),
    ("LFO blend mode Switch", LfoBlendMode(Switch)),
    ("LFO blend mode Blend", LfoBlendMode(Blend)),
    ("LFO key sync On", LfoKeySync(On)),
    ("LFO key sync Off", LfoKeySync(Off)),
    ("LFO one-shot On", LfoOneShot(On)),
    ("LFO one-shot Off", LfoOneShot(Off)),
    ("LFO retrigger On", LfoRetrigger(On)),
    ("LFO retrigger Off", LfoRetrigger(Off)),
    ("LFO midi sync On", LfoMidiSync(On)),
    ("LFO midi sync Off", LfoMidiSync(Off)),
    ("LFO reset order", LfoResetOrder),
    ("VCF key tracking On", VcfKeyTracking(On)),
    ("VCF key tracking Off", VcfKeyTracking(Off)),
];

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let key_events = Events::new();

    let (midi_in_sender, midi_in_receiver) = mpsc::channel();
    let state = State::new();

    let app = &mut App::new(state)?;
    if let Err(error) = app.connection.register_midi_in_channel(midi_in_sender) {
        app.command_history.push(format!("{}", error))
    };

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
                .map(|event| match neutron_message(event.as_slice()) {
                    Ok((_, msg)) => Text::raw(msg.to_string()),
                    Err(_) => Text::raw(hex::encode(event)),
                });
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
                Key::Char('s') => app.command(protocol::maybe_request_state().as_slice()),
                Key::Char('P') => app.command(
                    SetGlobalSetting(Multicast, ParaphonicMode(On))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('p') => app.command(
                    SetGlobalSetting(Multicast, ParaphonicMode(Off))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('Y') => app.command(
                    SetGlobalSetting(Multicast, OscSync(On))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('y') => app.command(
                    SetGlobalSetting(Multicast, OscSync(Off))
                        .as_bytes()
                        .as_slice(),
                ),

                // Menu stuff
                Key::Char('\n') => app.command(
                    SetGlobalSetting(Multicast, MENU_MAPPINGS[menu_selection].1)
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Down => {
                    menu_selection = (menu_selection + 1) % menu_items.len();
                }
                Key::Up => {
                    if menu_selection == 0 {
                        menu_selection = menu_items.len() - 1;
                    } else {
                        menu_selection = menu_selection - 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
