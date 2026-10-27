//! Labelled controller-only traversal over the production Lux worker.
//! Logical pad/rotary gestures are not an assumed physical MIDI profile.
use crate::{
    frontend::{View, Worker},
    native_actions::{ConsoleEditor, Semantic},
    surface::{Page, actions::Action},
};
#[derive(Clone, Debug)]
pub enum Command {
    Action(Semantic),
    Scroll(i32),
    Reconnect(u64),
}
#[derive(Clone, Debug)]
pub struct Item {
    pub label: &'static str,
    pub command: Command,
}
#[derive(Clone, Copy, Debug)]
pub enum Gesture {
    CancelPress,
    Menu,
    Browse(i32),
    Character(i32),
    Pad { index: usize, pressed: bool },
}
pub struct Surface {
    pub open: bool,
    pub selected: usize,
    pub scroll: usize,
    character: usize,
    held: [bool; 8],
    blocked: [bool; 8],
    pending: Option<(usize, String, u64)>,
    epoch: u64,
    press_context: [Option<(String, u64)>; 8],
}
impl Surface {
    pub fn new(epoch: u64) -> Self {
        Self {
            open: false,
            selected: 0,
            scroll: 0,
            character: 0,
            held: [false; 8],
            blocked: [false; 8],
            pending: None,
            epoch,
            press_context: Default::default(),
        }
    }
    pub fn midi(
        &mut self,
        binding: &MidiBinding,
        endpoint: &str,
        bytes: &[u8],
        generation: u64,
        worker: &Worker,
    ) -> Result<(), String> {
        if worker.role.as_ref().is_some_and(|role| {
            role.state
                .lock()
                .unwrap()
                .grant
                .as_ref()
                .is_some_and(|g| g["binding"]["controller"].as_str() != Some(endpoint))
        }) {
            return Ok(());
        }
        let assigned = worker.role.as_ref().is_some_and(|role| {
            let state = role.state.lock().unwrap();
            state.live()
                && role.live.load(std::sync::atomic::Ordering::Acquire)
                && state
                    .grant
                    .as_ref()
                    .is_some_and(|g| g["binding"]["controller"].as_str() == Some(endpoint))
        });
        if !assigned {
            self.lost();
            return Err("controller endpoint has no live assigned role".into());
        }
        if generation != worker.generation() {
            self.lost();
            return Err("stale controller acquisition".into());
        }
        if let Some(event) = binding.decode(endpoint, bytes) {
            let view = worker.view.lock().unwrap().current();
            self.gesture(event, &view, worker)?;
        }
        Ok(())
    }
    pub fn lost(&mut self) {
        self.open = false;
        self.held = [false; 8];
        self.blocked = [true; 8];
        self.pending = None;
        self.press_context = Default::default();
    }
    pub fn character(&self) -> char {
        (32 + self.character as u8) as char
    }
    fn context(view: &View) -> String {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{}|{}",
            view.inventory
                .as_ref()
                .map(|i| (&i["snapshot"]["epoch"], &i["snapshot"]["revision"])),
            view.selected,
            view.review_context,
            view.editor,
            view.page,
            view.role_live
        )
    }
    pub fn items(&self, view: &View) -> Vec<Item> {
        let mut out = Vec::new();
        let mut add = |label, command| out.push(Item { label, command });
        if view.review.is_some() {
            add("Previous review page", Command::Scroll(-1));
            add("Next review page", Command::Scroll(1));
            add(
                "Confirm reviewed change",
                Command::Action(Semantic::Action(Action::Confirm)),
            );
            add("Cancel", Command::Action(Semantic::Action(Action::Cancel)));
            return out;
        }
        add("Reconnect read-only", Command::Reconnect(self.epoch));
        let mut action = |label, a| add(label, Command::Action(a));
        action("Request control", Semantic::Grant);
        for (label, page) in [
            ("Stage", Page::Stage),
            ("Programmer", Page::Programmer),
            ("Library", Page::Library),
            ("Playbacks", Page::Playbacks),
            ("Health", Page::Health),
        ] {
            action(label, Semantic::Action(Action::Page(page)));
        }
        action("Scenes", Semantic::Scenes);
        action("Previous fixture", Semantic::Action(Action::Navigate(-1)));
        action("Next fixture", Semantic::Action(Action::Navigate(1)));
        action(
            "Select focused fixture",
            Semantic::Action(Action::Select(vec![])),
        );
        action("Add / remove focused fixture", Semantic::ToggleFocused);
        action("Deselect", Semantic::Action(Action::Deselect));
        action("Previous fixture bank", Semantic::Bank(-1));
        action("Next fixture bank", Semantic::Bank(1));
        action(
            "Previous attribute",
            Semantic::Action(Action::Attribute(-1)),
        );
        action("Next attribute", Semantic::Action(Action::Attribute(1)));
        action("Edit value", Semantic::Edit);
        action("Review entered value", Semantic::Action(Action::Confirm));
        action("Cancel", Semantic::Action(Action::Cancel));
        for (label, palette, replace) in [
            ("Record cue", false, false),
            ("Record palette", true, false),
            ("Update cue", false, true),
            ("Update palette", true, true),
        ] {
            action(
                label,
                Semantic::Action(Action::Record {
                    palette,
                    replace,
                    id: 0,
                }),
            );
        }
        action("Choose cue and playback", Semantic::OpenGo);
        action("Clear to Hold", Semantic::Action(Action::ClearHold));
        action("Preview selected attribute release", Semantic::Preview);
        action(
            "Choose release attributes",
            Semantic::ConsoleEdit(ConsoleEditor::ReleaseMask),
        );
        action("Grand master", Semantic::ConsoleEdit(ConsoleEditor::Master));
        action("Review blackout / release", Semantic::Blackout);
        action("Review Manual / Assist", Semantic::Mode);
        action("Save checkpoint", Semantic::Checkpoint);
        action(
            "Import scene map",
            Semantic::ConsoleEdit(ConsoleEditor::SceneImport),
        );
        action(
            "Export scene map",
            Semantic::ConsoleEdit(ConsoleEditor::SceneExport),
        );
        action("Edit scene", Semantic::SceneEdit);
        action("Preview scene", Semantic::ScenePreview);
        action(
            "Scene grant",
            Semantic::ConsoleEdit(ConsoleEditor::SceneGrant),
        );
        action("Start calibration", Semantic::Calibrate(false));
        action("Finish calibration", Semantic::Calibrate(true));
        action("Review AUTO entry", Semantic::AutoEnter);
        action("Set AUTO grant", Semantic::AutoGrant);
        action("Review AUTO revoke", Semantic::AutoRevoke);
        out
    }
    /// Full pad edges dispatch once; stale menu/role/review contexts are rejected.
    pub fn gesture(&mut self, event: Gesture, view: &View, worker: &Worker) -> Result<(), String> {
        let items = self.items(view);
        self.selected = self.selected.min(items.len().saturating_sub(1));
        match event {
            Gesture::CancelPress => {
                self.pending = None;
                self.held = [false; 8];
            }
            Gesture::Menu => {
                self.selected = self.selected.min(items.len().saturating_sub(1));
                for i in 0..8 {
                    self.blocked[i] |= self.held[i];
                }
                self.open = !self.open;
                self.held = [false; 8];
                self.pending = None;
            }
            Gesture::Browse(n) => {
                self.open = true;
                self.selected = (self.selected as i64 + i64::from(n.clamp(-8, 8)))
                    .rem_euclid(items.len() as i64) as usize
            }
            Gesture::Character(n) => {
                self.character =
                    (self.character as i64 + i64::from(n.clamp(-8, 8))).rem_euclid(95) as usize
            }
            Gesture::Pad { index, pressed } if index < 8 => {
                if pressed {
                    if !self.blocked[index] && !self.held[index] {
                        self.held[index] = true;
                        self.press_context[index] =
                            Some((Self::context(view), worker.generation()));
                        if index == 2 {
                            self.pending =
                                Some((self.selected, Self::context(view), worker.generation()));
                        }
                    }
                } else {
                    self.blocked[index] = false;
                    if !std::mem::take(&mut self.held[index]) {
                        return Ok(());
                    }
                    let generation = worker.generation();
                    let context = self.press_context[index].take();
                    if context != Some((Self::context(view), worker.generation())) {
                        return Err("controller context changed; release and choose again".into());
                    }
                    match index {
                        0 => self.gesture(Gesture::Browse(-1), view, worker)?,
                        1 => self.gesture(Gesture::Browse(1), view, worker)?,
                        2 => {
                            let (at, context, generation) = self
                                .pending
                                .take()
                                .ok_or("controller press context missing")?;
                            if at != self.selected
                                || context != Self::context(view)
                                || generation != worker.generation()
                            {
                                return Err("controller context changed; choose again".into());
                            }
                            let command = &items.get(at).ok_or("menu changed")?.command;
                            self.open = false;
                            match command {
                                Command::Action(action) => {
                                    worker.controller_action_at(action.clone(), generation)?;
                                }
                                Command::Scroll(n) => {
                                    self.scroll = (self.scroll as i64 + i64::from(*n) * 20)
                                        .clamp(0, 4096)
                                        as usize
                                }
                                Command::Reconnect(epoch) => {
                                    worker.controller_reconnect(*epoch)?;
                                }
                            }
                        }
                        3 => {
                            self.open = false;
                            worker.controller_action_at(
                                Semantic::Action(Action::Cancel),
                                generation,
                            )?;
                        }
                        4 => self.gesture(Gesture::Character(-1), view, worker)?,
                        5 => self.gesture(Gesture::Character(1), view, worker)?,
                        6 => {
                            worker.controller_action_at(
                                Semantic::Action(Action::Text(self.character().to_string())),
                                generation,
                            )?;
                        }
                        7 => {
                            worker.controller_action_at(
                                Semantic::Action(Action::Backspace),
                                generation,
                            )?;
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub fn scene(&self, view: &View) -> crate::render::Scene {
        use crate::render::{Primitive, Scene};
        let items = self.items(view);
        let selected = self.selected.min(items.len().saturating_sub(1));
        let mut lines = vec!["CONTROLS / choose an action".to_owned()];
        for (i, item) in items.iter().enumerate().skip(selected / 24 * 24).take(24) {
            lines.push(format!(
                "{} {}",
                if i == selected { ">" } else { " " },
                item.label
            ));
        }
        lines.push(format!("Entry character: {:?}", self.character()));
        if let Some(editor) = &view.editor {
            lines.push(format!("Value: {}", editor.text()));
        }
        lines.push(view.notice.clone());
        let mut scene = Scene::default();
        scene.primitives.push(Primitive::Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
            fill: "#10151d".into(),
        });
        for (i, value) in lines.into_iter().enumerate() {
            scene.primitives.push(Primitive::Text {
                x: 24,
                y: 24 + i as u32 * 32,
                value: value.chars().take(156).collect(),
                color: "#e4e8e9",
            });
        }
        scene
    }
}

/// Explicit learned pad binding. No default preset, enumeration or MIDI output.
#[derive(Clone, Debug)]
pub struct MidiBinding {
    endpoint: String,
    channel: u8,
    notes: [u8; 8],
}
impl MidiBinding {
    pub fn new(endpoint: String, channel: u8, notes: [u8; 8]) -> Result<Self, String> {
        if endpoint.is_empty()
            || endpoint.len() > 128
            || channel > 15
            || notes
                .iter()
                .enumerate()
                .any(|(i, n)| *n > 127 || notes[..i].contains(n))
        {
            return Err("invalid or ambiguous controller pad binding".into());
        }
        Ok(Self {
            endpoint,
            channel,
            notes,
        })
    }
    pub fn decode(&self, endpoint: &str, bytes: &[u8]) -> Option<Gesture> {
        if endpoint != self.endpoint
            || bytes.len() != 3
            || bytes[1] > 127
            || bytes[2] > 127
            || bytes[0] & 15 != self.channel
            || !matches!(bytes[0] & 0xf0, 0x80 | 0x90)
        {
            return None;
        }
        let index = self.notes.iter().position(|n| *n == bytes[1])?;
        Some(Gesture::Pad {
            index,
            pressed: bytes[0] & 0xf0 == 0x90 && bytes[2] != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn worker() -> Worker {
        Worker::spawn(
            std::env::temp_dir().join(format!(
                "missing-controller-provider-{}",
                std::process::id()
            )),
            "11111111-1111-4111-8111-111111111111".into(),
            1,
        )
    }
    #[test]
    fn menu_traversal_entry_and_recovery_are_bounded() {
        let worker = worker();
        let mut surface = Surface::new(1);
        let view = View::default();
        let binding = MidiBinding::new(
            "assigned-lightdesk".into(),
            9,
            [36, 37, 38, 39, 40, 41, 42, 43],
        )
        .unwrap();
        assert!(binding.decode("other-desk", &[0x99, 37, 127]).is_none());
        assert!(
            binding
                .decode("assigned-lightdesk", &[0xb9, 37, 127])
                .is_none()
        );
        surface
            .gesture(
                binding
                    .decode("assigned-lightdesk", &[0x99, 37, 127])
                    .unwrap(),
                &view,
                &worker,
            )
            .unwrap();
        surface
            .gesture(
                binding
                    .decode("assigned-lightdesk", &[0x99, 37, 127])
                    .unwrap(),
                &view,
                &worker,
            )
            .unwrap();
        surface
            .gesture(
                binding
                    .decode("assigned-lightdesk", &[0x99, 37, 0])
                    .unwrap(),
                &view,
                &worker,
            )
            .unwrap();
        assert_eq!(surface.selected, 1);
        assert!(surface.open);
        surface
            .gesture(
                Gesture::Pad {
                    index: 1,
                    pressed: false,
                },
                &view,
                &worker,
            )
            .unwrap();
        assert_eq!(surface.selected, 1);
        assert!(surface.scene(&view).in_bounds());
        surface
            .gesture(
                Gesture::Pad {
                    index: 2,
                    pressed: true,
                },
                &view,
                &worker,
            )
            .unwrap();
        let mut changed = view.clone();
        changed.review_context = Some(3);
        assert!(
            surface
                .gesture(
                    Gesture::Pad {
                        index: 2,
                        pressed: false
                    },
                    &changed,
                    &worker
                )
                .is_err()
        );
        surface.lost();
        surface
            .gesture(
                Gesture::Pad {
                    index: 2,
                    pressed: false,
                },
                &view,
                &worker,
            )
            .unwrap();
        assert!(!surface.open);
    }
    #[test]
    fn menu_has_value_review_selection_and_show_controls_without_key_names() {
        let s = Surface::new(1);
        let view = View::default();
        let items = s.items(&view);
        for label in [
            "Request control",
            "Reconnect read-only",
            "Edit value",
            "Add / remove focused fixture",
            "Choose release attributes",
            "Grand master",
            "Scene grant",
            "Save checkpoint",
        ] {
            assert!(items.iter().any(|i| i.label == label), "{label}");
        }
        let review = View {
            review: Some(serde_json::json!({"action":"blackout","enabled":false})),
            review_context: Some(1),
            ..view
        };
        let items = s.items(&review);
        assert_eq!(items.len(), 4);
        assert_eq!(items[2].label, "Confirm reviewed change");
        assert!(MidiBinding::new("x".into(), 0, [1, 1, 2, 3, 4, 5, 6, 7]).is_err());
    }
}
