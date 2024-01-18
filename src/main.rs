extern crate hidapi;

use hidapi::HidApi;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the HID API
    let api = HidApi::new().expect("Failed to initialize HID API");

    // Find the target HID device by vendor and product IDs. Can be found on MacOS System Report.
    let vendor_id = 0x1222; // Replace with your device's vendor ID
    let product_id = 0xfaca; // Replace with your device's product ID
    let device_info = api
        .device_list()
        .find(|info| info.vendor_id() == vendor_id && info.product_id() == product_id)
        .expect("Target HID device not found");

    // Open the HID device
    let device = api
        .open_path(device_info.path())
        .expect("Failed to open HID device");

    // Initialize MIDI
    let midi_out = MidiOutput::new("Tipro Sender")?;
    let ports = midi_out.ports();

    println!("Available MIDI output ports:");
    for (i, p) in ports.iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }

    let midi_port: &MidiOutputPort = match ports.len() {
        0 => return Err("No output ports found.".into()),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&ports[0]).unwrap()
            );
            &ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("Invalid MIDI output port selected")?
        }
    };

    // let mut connection = midi_port.connect("MIDI Sender")?;
    let mut connection = midi_out.connect(midi_port, "MIDI Sender")?;

    // Main loop to read HID events
    loop {
        let mut buffer = [0u8; 64]; // Adjust the buffer size based on your device's report size

        // Read HID input report
        device.read(&mut buffer).expect("Failed to read HID report");
        let key = buffer[1];

        if key > 0u8 {
            println!("Received HID data: {:?}", key);

            let note = key % 127;
            let note_on_4ths_layout = subtract_eleven_on_multiple_of_sixteen(note) + 36;
            println!("Note on layout: {:?}", note_on_4ths_layout);

            if key > 127 {
                send_note_on(&mut connection, note_on_4ths_layout, 64)?;
            } else {
                send_note_off(&mut connection, note_on_4ths_layout)?;
            }
        }
    }
}

fn subtract_eleven_on_multiple_of_sixteen(num: u8) -> u8 {
    let offset = (num - 4) / 16;
    return num - 11 * offset;
}

// Function to send MIDI note-on message
fn send_note_on(
    connection: &mut MidiOutputConnection,
    note: u8,
    velocity: u8,
) -> Result<(), Box<dyn Error>> {
    const NOTE_ON: u8 = 0x90; // MIDI note-on status byte (channel 1)
    connection.send(&[NOTE_ON, note, velocity])?;
    Ok(())
}

// Function to send MIDI note-off message
fn send_note_off(connection: &mut MidiOutputConnection, note: u8) -> Result<(), Box<dyn Error>> {
    const NOTE_OFF: u8 = 0x80; // MIDI note-off status byte (channel 1)
    const RELEASE_VELOCITY: u8 = 0; // Velocity 0 for note-off
    connection.send(&[NOTE_OFF, note, RELEASE_VELOCITY])?;
    Ok(())
}
