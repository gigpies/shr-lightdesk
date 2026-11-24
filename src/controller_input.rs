//! Production pad acquisition: a strict learned pad profile plus the shared
//! read-only MIDI reader, delivered through `Surface::midi` and its role fences.
//! Nothing here writes to a device, guesses notes or grants authority.
use crate::{
    controller_surface::{MidiBinding, Surface},
    frontend::Worker,
};
use shr_instrument_panel::midi::{self, Event, Reader};
use std::{path::Path, sync::mpsc::Receiver};

pub const CONTRACT: &str = "SHR-LIGHTDESK-CONTROLLER";
pub struct Profile {
    pub endpoint: String,
    pub binding: MidiBinding,
    pub hardware_qualified: bool,
}
/// Strictly decode one learned pad profile; unknown or missing fields are refused.
pub fn parse_profile(bytes: &[u8]) -> Result<Profile, String> {
    if bytes.len() > 16 * 1024 {
        return Err("controller profile capacity".into());
    }
    let v: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let o = v.as_object().ok_or("controller profile object")?;
    const KEYS: [&str; 6] = [
        "contract",
        "version",
        "endpoint",
        "pad_channel",
        "pads",
        "hardware_qualified",
    ];
    if o.len() != KEYS.len() || !KEYS.iter().all(|k| o.contains_key(*k)) {
        return Err("controller profile fields".into());
    }
    if o["contract"] != CONTRACT || o["version"] != 1 {
        return Err("unsupported controller profile contract".into());
    }
    let byte = |v: &serde_json::Value| v.as_u64().filter(|n| *n <= 127).map(|n| n as u8);
    let endpoint = o["endpoint"].as_str().ok_or("endpoint")?.to_owned();
    let channel = byte(&o["pad_channel"]).ok_or("pad_channel")?;
    let pads = o["pads"]
        .as_array()
        .filter(|a| a.len() == 8)
        .ok_or("eight pads")?;
    let mut notes = [0u8; 8];
    for (n, p) in notes.iter_mut().zip(pads) {
        *n = byte(p).ok_or("pad note")?;
    }
    Ok(Profile {
        binding: MidiBinding::new(endpoint.clone(), channel, notes)?,
        endpoint,
        hardware_qualified: o["hardware_qualified"]
            .as_bool()
            .ok_or("hardware_qualified")?,
    })
}
pub fn load_profile(path: &Path) -> Result<Profile, String> {
    parse_profile(&std::fs::read(path).map_err(|e| format!("controller profile: {e}"))?)
}

pub struct Acquisition {
    profile: Profile,
    events: Receiver<Event>,
    pub last_error: Option<String>,
    _reader: Reader,
}
impl Acquisition {
    pub fn start(profile: Profile) -> Result<Self, String> {
        let id = profile.endpoint.clone();
        Self::spawn(profile, move || {
            midi::resolve_by_id(Path::new("/dev/snd"), &id)
        })
    }
    /// Explicit path (FIFO or device node) instead of `by-id` resolution.
    pub fn start_path(profile: Profile, path: std::path::PathBuf) -> Result<Self, String> {
        Self::spawn(profile, move || Ok(path.clone()))
    }
    fn spawn(
        profile: Profile,
        open: impl Fn() -> Result<std::path::PathBuf, String> + Send + 'static,
    ) -> Result<Self, String> {
        let (reader, events) = Reader::spawn(&profile.endpoint, open)?;
        Ok(Self {
            profile,
            events,
            last_error: None,
            _reader: reader,
        })
    }
    /// Drain queued input without blocking. Loss clears held pads; the role
    /// fence inside `Surface::midi` decides whether a message may act.
    pub fn poll(&mut self, surface: &mut Surface, worker: &Worker) -> usize {
        let mut delivered = 0;
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Lost { .. } => surface.lost(),
                Event::Message {
                    endpoint, message, ..
                } => match surface.midi(
                    &self.profile.binding,
                    &endpoint,
                    message.bytes(),
                    worker.generation(),
                    worker,
                ) {
                    Ok(()) => delivered += 1,
                    Err(e) => self.last_error = Some(e),
                },
            }
        }
        delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_pad_profile() {
        let ok = serde_json::json!({"contract": CONTRACT, "version": 1,
            "endpoint": "usb-Arturia_Arturia_MiniLab_mkII-00", "pad_channel": 9,
            "pads": [36,37,38,39,40,41,42,43], "hardware_qualified": false});
        assert!(parse_profile(&serde_json::to_vec(&ok).unwrap()).is_ok());
        for (k, v) in [
            ("pads", serde_json::json!([36, 36, 38, 39, 40, 41, 42, 43])),
            ("pad_channel", 16.into()),
            ("version", 2.into()),
            ("extra", true.into()),
        ] {
            let mut bad = ok.clone();
            bad[k] = v;
            assert!(
                parse_profile(&serde_json::to_vec(&bad).unwrap()).is_err(),
                "{k}"
            );
        }
    }
}
