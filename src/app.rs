use termion::event::Key;

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
use crate::midi;

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
    pub items: Vec<T>,
    pub selection: usize,
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
    pub connection: midi::MidiConnection,
    pub neutron_state: NeutronState,
    pub command_history: Vec<String>,
    // TODO will grow indefinitely, does it matter?
    pub midi_in_messages: Vec<Vec<u8>>,
    pub basic_menu: ListState<String>,
    pub should_quit: bool,
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
