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

use crate::events::Event;
use crate::midi;

mod state {
    use rustron_lib::protocol::GlobalSetting;
    use rustron_lib::protocol::NeutronMessage;

    #[derive(Default)]
    pub struct GlobalSettingsState {
        // TODO device_id stuff
        device_id: u8,
        paraphonic_mode: bool,
        osc_sync: bool,
    }

    #[derive(Default)]
    pub struct NeutronState {
        global_settings: GlobalSettingsState,
    }

    impl NeutronState {
        pub fn new() -> NeutronState {
            // TODO device_id
            Default::default()
        }

        fn global_setting_update(&mut self, global_setting: GlobalSetting) {
            match global_setting {
                GlobalSetting::ParaphonicMode(t) => self.global_settings.paraphonic_mode = t.into(),
                GlobalSetting::OscSync(_) => {}
                GlobalSetting::Osc1BlendMode(_) => {}
                GlobalSetting::Osc2BlendMode(_) => {}
                GlobalSetting::Osc1TunePotBypass(_) => {}
                GlobalSetting::Osc2TunePotBypass(_) => {}
                GlobalSetting::Osc1Range(_) => {}
                GlobalSetting::Osc2Range(_) => {}
                GlobalSetting::Osc2KeyTrack(_) => {}
                GlobalSetting::Osc1Autoglide(_) => {}
                GlobalSetting::Osc2Autoglide(_) => {}
                GlobalSetting::LfoBlendMode(_) => {}
                GlobalSetting::LfoKeySync(_) => {}
                GlobalSetting::LfoOneShot(_) => {}
                GlobalSetting::LfoRetrigger(_) => {}
                GlobalSetting::LfoMidiSync(_) => {}
                GlobalSetting::LfoDepth(_) => {}
                GlobalSetting::LfoShapeOrder(_, _) => {}
                GlobalSetting::LfoShapePhase(_, _) => {}
                GlobalSetting::LfoResetOrder => {}
                GlobalSetting::VcfKeyTracking(_) => {}
                GlobalSetting::VcfModDepth(_) => {}
                GlobalSetting::VcfModSource(_) => {}
                GlobalSetting::MidiChannel(_) => {}
                GlobalSetting::DisableMidiDips(_) => {}
                GlobalSetting::PolyChainMode(_) => {}
                GlobalSetting::KeyRangeMute(_) => {}
                GlobalSetting::KeyRangeReset => {}
                GlobalSetting::AssignOut(_) => {}
                GlobalSetting::EnvRetriggerMode(_) => {}
            }
        }

        pub fn update(&mut self, message: NeutronMessage) {
            match message {
                NeutronMessage::SetGlobalSetting(_, global_setting) => {
                    // Messages sent to the Neutron
                    self.global_setting_update(global_setting)
                }
                NeutronMessage::GlobalSettingUpdate(_, global_setting) => {
                    // Messages sent from the Neutron
                    self.global_setting_update(global_setting)
                }
                NeutronMessage::RestoreGlobalSetting(_) => {}
                NeutronMessage::CalibrationModeCommand(_) => {}
                NeutronMessage::SoftwareVersionRequest(_) => {}
                NeutronMessage::SoftwareVersionResponse(_, _) => {}
            }
        }
    }

    pub struct ListState<T> {
        pub items: Vec<T>,
        pub selection: usize,
    }

    impl<T> ListState<T> {
        pub fn new(items: Vec<T>) -> ListState<T> {
            ListState {
                items,
                selection: 0,
            }
        }

        pub fn select_next(&mut self) {
            self.selection = (self.selection + 1) % self.items.len();
        }

        pub fn select_previous(&mut self) {
            if self.selection == 0 {
                self.selection = self.items.len() - 1;
            } else {
                self.selection -= 1
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::app::state::NeutronState;
        use rustron_lib::protocol::Channel::One;
        use rustron_lib::protocol::DeviceId::Channel;
        use rustron_lib::protocol::GlobalSetting::ParaphonicMode;
        use rustron_lib::protocol::NeutronMessage::{GlobalSettingUpdate, SetGlobalSetting};
        use rustron_lib::protocol::ToggleOption::{Off, On};

        #[test]
        fn paraphonic_mode_is_updated() {
            let mut ns = NeutronState::new();
            assert!(!ns.global_settings.paraphonic_mode);
            ns.update(SetGlobalSetting(Channel(One), ParaphonicMode(On)));
            assert!(ns.global_settings.paraphonic_mode);
            ns.update(GlobalSettingUpdate(Channel(One), ParaphonicMode(Off)));
            assert!(!ns.global_settings.paraphonic_mode);
        }
    }
}

pub struct App {
    pub connection: midi::MidiConnection,
    pub neutron_state: state::NeutronState,
    pub command_history: Vec<String>,
    // TODO will grow indefinitely, does it matter?
    pub midi_in_messages: Vec<Vec<u8>>,
    pub basic_menu: state::ListState<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            connection: midi::MidiConnection::new(),
            neutron_state: state::NeutronState::new(),
            command_history: Vec::new(),
            midi_in_messages: Vec::new(),
            basic_menu: state::ListState::new(
                MENU_MAPPINGS
                    .iter()
                    .map(|(name, _)| name.to_string())
                    .collect(),
            ),
            should_quit: false,
        }
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
        if let Event::Input(key) = event {
            match key {
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
            }
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

#[cfg(test)]
mod test {

    use crate::app::App;

    #[test]
    fn test() {
        //TODO
        let app = App::new();
    }
}
