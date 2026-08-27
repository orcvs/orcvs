use midir::{MidiOutput, MidiOutputConnection};

use crate::midi::{MidiBackend, MidiConnection, MidiDestination, MidiError, MidiOutputAdapter};

pub type NativeMidiOutputAdapter = MidiOutputAdapter<MidirBackend>;

#[derive(Default)]
pub struct MidirBackend;

impl MidiBackend for MidirBackend {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, MidiError> {
        let output = MidiOutput::new("Orca").map_err(midi_error)?;
        output
            .ports()
            .into_iter()
            .map(|port| {
                let name = output.port_name(&port).map_err(midi_error)?;
                Ok(MidiDestination::new(port.id(), name))
            })
            .collect()
    }

    fn connect(&mut self, destination_id: &str) -> Result<Box<dyn MidiConnection>, MidiError> {
        let output = MidiOutput::new("Orca").map_err(midi_error)?;
        let port = output.find_port_by_id(destination_id).ok_or_else(|| {
            MidiError::new("the selected MIDI destination is no longer available")
        })?;
        let connection = output.connect(&port, "Orca output").map_err(midi_error)?;
        Ok(Box::new(MidirConnection(connection)))
    }
}

struct MidirConnection(MidiOutputConnection);

impl MidiConnection for MidirConnection {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.0.send(message).map_err(midi_error)
    }
}

fn midi_error(error: impl std::fmt::Display) -> MidiError {
    MidiError::new(error.to_string())
}
