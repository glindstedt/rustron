use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use libusb;
use midir::{ConnectError, MidiInput, MidiOutput, PortInfoError};

fn main() {
    println!("Hello, world!");
    let mut context = libusb::Context::new().unwrap();

    for mut device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        device_desc.product_string_index().map(|idx| println!("{}", idx));


        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                 device.bus_number(),
                 device.address(),
                 device_desc.vendor_id(),
                 device_desc.product_id());
    }
    communicate();
}

pub fn communicate() -> Result<&'static str, Box<dyn Error>> {
    let output = MidiOutput::new("Neutron").unwrap();

    let out_port = get_neutron_port(&output)?;

    let input = MidiInput::new("Neutron").unwrap();

    let in_port = get_neutron_port(&input)?;

    let mut conn_out = output.connect(out_port, "neutron")?;

    let mut result = conn_out.send(&turn_on_paraphonic_mode());
    result.map_err(|e| println!("{}", e));
    sleep(Duration::from_millis(5000));
    result = conn_out.send(&turn_off_paraphonic_mode());
    result.map_err(|e| println!("{}", e));

    Ok("foo")
}

const sysex_message_start: u8 = 0xf0;
const behringer_manufacturer: [u8; 3] = [0x00, 0x20, 0x32];


pub fn turn_on_paraphonic_mode() -> Vec<u8> {
        vec![
            sysex_message_start,
            behringer_manufacturer[0],
            behringer_manufacturer[1],
            behringer_manufacturer[2],
            0x28,
            0x7f,
            0x0a,
            0x0f,
            0x01,
            0xf7
        ]
}

pub fn turn_off_paraphonic_mode() -> Vec<u8> {
    vec![
        sysex_message_start,
        behringer_manufacturer[0],
        behringer_manufacturer[1],
        behringer_manufacturer[2],
        0x28,
        0x7f,
        0x0a,
        0x0f,
        0x00,
        0xf7
    ]
}

// pub fn set_command(counter: u8, command_type: &str, value: &str) -> Vec<u8> {
pub fn set_para_on() -> Vec<u8> {
    vec![
        0xf0,
        0x00,
        0x20,
        0x32,
        0x28,
        0x00,
        0x5a,
        0x01,
        0x0f,
        0x01,
        0xf7
    ]
}

pub fn set_para_off() -> Vec<u8> {
    vec![
        0xf0,
        0x00,
        0x20,
        0x32,
        0x28,
        0x00,
        0x5a,
        0x01,
        0x0f,
        0x00,
        0xf7
    ]
}

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

pub fn get_neutron_port(midi_output: &dyn Neutron) -> Result<usize, &str> {
    let mut out_port: Option<usize> = None;
    for i in 0..midi_output.port_count() {
        match midi_output.port_name(i).unwrap().starts_with("Neutron") {
            true => {
                println!("FOUND IT: {}", i);
                out_port = Some(i);
                break;
            }
            _ => ()
        }
    }
    match out_port {
        Some(i) => Ok(i),
        None => Err("Could not find Neutron."),
    }
}
