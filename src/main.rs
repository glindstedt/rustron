use std::io;
use std::sync::mpsc;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};
use tui::{Frame, Terminal};

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

use crate::events::{Event, Events};

mod events;
mod midi;

#[derive(Default)]
pub struct NeutronState {
    // TODO
    paraphonic_mode: bool,
    osc_sync: bool,
}

impl NeutronState {
    pub fn new() -> NeutronState {
        Default::default()
    }
}

pub struct ListState<T> {
    items: Vec<T>,
    selection: usize,
}

impl<T> ListState<T> {
    fn new(items: Vec<T>) -> ListState<T> {
        ListState {
            items,
            selection: 0,
        }
    }

    fn select_next(&mut self) {
        self.selection = (self.selection + 1) % self.items.len();
    }

    fn select_previous(&mut self) {
        if self.selection == 0 {
            self.selection = self.items.len() - 1;
        } else {
            self.selection -= 1
        }
    }
}

pub struct App {
    connection: midi::MidiConnection,
    neutron_state: NeutronState,
    command_history: Vec<String>,
    // TODO will grow indefinitely, does it matter?
    midi_in_messages: Vec<Vec<u8>>,
    basic_menu: ListState<String>,
    should_quit: bool,
}

impl App {
    pub fn new() -> Result<App, failure::Error> {
        Ok(App {
            connection: midi::MidiConnection::new().into(),
            neutron_state: NeutronState::new(),
            command_history: Vec::new().into(),
            midi_in_messages: Vec::new().into(),
            basic_menu: ListState::new(
                MENU_MAPPINGS
                    .iter()
                    .map(|(name, _)| name.to_string())
                    .collect(),
            ),
            should_quit: false,
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

    pub fn handle_event(&mut self, event: Event<Key>) {
        match event {
            Event::Input(key) => match key {
                Key::Char('q') => self.should_quit = true,
                Key::Char('s') => self.command(protocol::maybe_request_state().as_slice()),
                Key::Char('P') => self.command(
                    SetGlobalSetting(Multicast, ParaphonicMode(On))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('p') => self.command(
                    SetGlobalSetting(Multicast, ParaphonicMode(Off))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('Y') => self.command(
                    SetGlobalSetting(Multicast, OscSync(On))
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Char('y') => self.command(
                    SetGlobalSetting(Multicast, OscSync(Off))
                        .as_bytes()
                        .as_slice(),
                ),

                // Menu stuff
                Key::Char('\n') => self.command(
                    SetGlobalSetting(Multicast, MENU_MAPPINGS[self.basic_menu.selection].1)
                        .as_bytes()
                        .as_slice(),
                ),
                Key::Down => {
                    self.basic_menu.select_next();
                }
                Key::Up => {
                    self.basic_menu.select_previous();
                }
                _ => {}
            },
            _ => {}
        }
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

// Used for primitive scrolling logic
fn bottom_slice<T>(array: &[T], max_size: usize) -> &[T] {
    let array_size = array.len();
    let start_index = if array_size < max_size {
        0
    } else {
        array_size - max_size
    };
    &array[start_index..]
}

fn render_command_history<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let command_history = bottom_slice(app.command_history.as_slice(), rectangle.height as usize)
        .iter()
        .map(|event| Text::raw(event.to_string()));
    List::new(command_history)
        .block(
            Block::default()
                .title("Command History")
                .borders(Borders::ALL),
        )
        .render(frame, rectangle);
}

fn render_options_menu<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rectangle);

    // Old menu
    SelectableList::default()
        .block(Block::default())
        .items(&app.basic_menu.items)
        .select(Some(app.basic_menu.selection))
        .highlight_symbol(">>")
        .render(frame, chunks[0]);

    // Prototype new menu
    //TODO
}

fn render_midi_stream<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let midi_messages = bottom_slice(app.midi_in_messages.as_slice(), rectangle.height as usize)
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
        .render(frame, rectangle);
}

fn main() -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let key_events = Events::new();

    let (midi_in_sender, midi_in_receiver) = mpsc::channel();

    let app = &mut App::new()?;
    if let Err(error) = app.connection.register_midi_in_channel(midi_in_sender) {
        app.command_history.push(format!("{}", error))
    };

    while !app.should_quit {
        match midi_in_receiver.try_recv() {
            Ok(msg) => app.midi_in_messages.push(msg.into()),
            Err(_) => {}
        }
        terminal.draw(|mut frame| {
            let size = frame.size();
            let vertical_split = Layout::default()
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
                    .split(vertical_split[0]);

                render_options_menu(&mut frame, chunks[0], app);
                render_command_history(&mut frame, chunks[1], app);
            }

            render_midi_stream(&mut frame, vertical_split[1], app);
        })?;

        app.handle_event(key_events.next()?);
    }
    Ok(())
}
