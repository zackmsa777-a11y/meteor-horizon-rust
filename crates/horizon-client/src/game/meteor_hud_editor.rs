// SPDX-License-Identifier: GPL-3.0-only
// Native ordered HUD instance editor and actual TextHud settings.
use super::{Game, inside};
use horizon_client::hud::{HudBatch, rgba};
use serde_json::{Value, json};
use winit::{event::MouseButton, keyboard::KeyCode};
#[derive(Default)]
pub(in crate::game) struct Editor {
    pub selected: Option<usize>,
    pub panel: bool,
    pub scroll: f32,
    pub max_scroll: f32,
    pub edit: Option<(String, String, bool)>,
    pub hits: Vec<(String, [f32; 4], Value)>,
}
impl Game<'_> {
    #[cfg(feature = "meteor")]
    pub(in crate::game) fn meteor_hud_editor_status(&self) -> Value {
        #[cfg(feature="meteor")]
        let output=self.meteor_hud.text_cache.iter().map(|(id,r)|(id.clone(),json!({"text":r.output.iter().map(|s|s.text.as_str()).collect::<String>(),"visible":r.visible}))).collect::<serde_json::Map<_,_>>();
        #[cfg(not(feature = "meteor"))]
        let output = serde_json::Map::<String, Value>::new();
        json!({"viewport":self.meteor_hud.viewport,"layout_viewport":self.meteor_hud.layout_viewport,"scale":self.meteor_hud.scale,"outputs":output,"selected":self.meteor_hud.controls.selected,"panel":self.meteor_hud.controls.panel,"editing":self.meteor_hud.controls.edit.as_ref().map(|(p,t,_)|json!({"path":p,"text":t})),"error":self.meteor_hud.error,"targets":self.meteor_hud.controls.hits.iter().map(|(key,rect,_)|(key.clone(),json!(rect))).collect::<serde_json::Map<_,_>>(),"elements":self.meteor_hud.bounds.iter().map(|(i,r)|json!({"index":i,"rect":r})).collect::<Vec<_>>()})
    }
    fn meteor_hud_edit_value(&mut self, path: &str, value: Value) {
        let mut candidate = self.meteor_hud.config.clone();
        if let Some(slot) = candidate.pointer_mut(path) {
            *slot = value;
        } else {
            return;
        }
        match self.meteor_hud_save_config(candidate.clone()) {
            Ok(()) => {
                self.meteor_hud.config = candidate;
                self.meteor_hud.error = None;
                self.meteor_hud.controls.edit = None;
            }
            Err(e) => self.meteor_hud.error = Some(e),
        }
    }
    pub(in crate::game) fn meteor_hud_control_key(
        &mut self,
        key: KeyCode,
        input: Option<&str>,
    ) -> bool {
        if !self.meteor_hud.editor {
            return false;
        }
        if let Some((path, mut text, mut replace)) = self.meteor_hud.controls.edit.clone() {
            let ctrl = self.physical_keys.contains(&KeyCode::ControlLeft)
                || self.physical_keys.contains(&KeyCode::ControlRight);
            match key {
                KeyCode::Escape => {
                    self.meteor_hud.controls.edit = None;
                    self.meteor_hud.error = None;
                    return true;
                }
                KeyCode::KeyA if ctrl => replace = true,
                KeyCode::Backspace => {
                    if replace {
                        text.clear();
                        replace = false
                    } else {
                        text.pop();
                    }
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    let old = self
                        .meteor_hud
                        .config
                        .pointer(&path)
                        .cloned()
                        .unwrap_or(Value::Null);
                    let candidate = if old.is_string() {
                        Ok(Value::String(text.clone()))
                    } else {
                        serde_json::from_str::<Value>(&text).map_err(|e| e.to_string())
                    };
                    match candidate {
                        Ok(v) => self.meteor_hud_edit_value(&path, v),
                        Err(e) => self.meteor_hud.error = Some(e),
                    };
                    return true;
                }
                _ => {
                    if !ctrl {
                        if let Some(input) = input {
                            let filtered: String =
                                input.chars().filter(|c| !c.is_control()).collect();
                            if !filtered.is_empty() {
                                if replace {
                                    text.clear();
                                    replace = false
                                }
                                if text.len() + filtered.len() <= 65536 {
                                    text.push_str(&filtered);
                                    self.meteor_hud.error = None
                                }
                            }
                        }
                    }
                }
            }
            self.meteor_hud.controls.edit = Some((path, text, replace));
            return true;
        }
        if key == KeyCode::Escape {
            if self.meteor_hud.controls.panel {
                self.meteor_hud.controls.panel = false
            } else {
                self.meteor_hud_editor_close()
            }
        }
        true
    }
    pub(in crate::game) fn meteor_hud_control_wheel(&mut self, steps: f32) -> bool {
        if !self.meteor_hud.editor {
            return false;
        }
        self.meteor_hud.controls.scroll = (self.meteor_hud.controls.scroll - steps * 54.)
            .clamp(0., self.meteor_hud.controls.max_scroll);
        true
    }
    pub(in crate::game) fn meteor_hud_control_mouse(
        &mut self,
        button: MouseButton,
        pressed: bool,
    ) -> bool {
        if !pressed || button != MouseButton::Left {
            return self.meteor_hud.controls.panel;
        }
        let hit = self
            .meteor_hud
            .controls
            .hits
            .iter()
            .find(|(_, r, _)| inside(*r, self.cursor))
            .cloned();
        let Some((key, _, value)) = hit else {
            return self.meteor_hud.controls.panel;
        };
        if !key.starts_with('/') {
            self.meteor_hud.controls.edit = None;
        }
        match key.as_str() {
            "add-text" => {
                #[cfg(feature = "meteor")]
                {
                    let elements = self.meteor_hud.config["elements"].as_array_mut().unwrap();
                    let mut n = 1;
                    while elements.iter().any(|e| e["id"] == format!("text-{n}")) {
                        n += 1
                    }
                    elements.push(json!({"id":format!("text-{n}"),"kind":"text","enabled":true,"anchor":[0,0],"offset":[0,0],"source_box":{"x_anchor":"Left","y_anchor":"Top","x":16,"y":80},"auto_anchors":true,"text":meteor_client::hud_text::TextSettings::default()}));
                    self.meteor_hud.controls.selected = Some(elements.len() - 1);
                    self.meteor_hud.controls.panel = true;
                    self.meteor_hud.controls.scroll = 0.;
                    self.meteor_hud_commit();
                }
            }
            "edit" => {
                self.meteor_hud.controls.panel = !self.meteor_hud.controls.panel;
                self.meteor_hud.controls.scroll = 0.;
            }
            "duplicate" => {
                if let Some(i) = self.meteor_hud.controls.selected {
                    let elements = self.meteor_hud.config["elements"].as_array_mut().unwrap();
                    if let Some(mut element) = elements.get(i).cloned() {
                        let base = element["id"].as_str().unwrap_or("element").to_owned();
                        let mut n = 1;
                        while elements
                            .iter()
                            .any(|e| e["id"] == format!("{base}-copy{n}"))
                        {
                            n += 1
                        }
                        element["id"] = json!(format!("{base}-copy{n}"));
                        if element["kind"].as_str().is_none_or(|s| s.is_empty()) {
                            element["kind"] = json!(base)
                        }
                        elements.push(element);
                        self.meteor_hud.controls.selected = Some(elements.len() - 1);
                        self.meteor_hud_commit();
                    }
                }
            }
            "delete" => {
                if let Some(i) = self.meteor_hud.controls.selected.take() {
                    if let Some(elements) = self.meteor_hud.config["elements"].as_array_mut() {
                        if i < elements.len() {
                            elements.remove(i);
                        }
                    }
                    self.meteor_hud.controls.panel = false;
                    self.meteor_hud_commit();
                }
            }
            _ => {
                if value.is_boolean() {
                    self.meteor_hud_edit_value(&key, json!(!value.as_bool().unwrap()));
                } else if key.ends_with("/shown") {
                    let next = match value.as_str() {
                        Some("Always") => "WhenTrue",
                        Some("WhenTrue") => "WhenFalse",
                        _ => "Always",
                    };
                    self.meteor_hud_edit_value(&key, json!(next));
                } else if key.ends_with("/x_anchor") || key.ends_with("/y_anchor") {
                    let options = if key.ends_with("/x_anchor") {
                        ["Left", "Center", "Right"]
                    } else {
                        ["Top", "Center", "Bottom"]
                    };
                    let _i = options
                        .iter()
                        .position(|s| Some(*s) == value.as_str())
                        .unwrap_or(0);
                    #[cfg(feature = "meteor")]
                    if let Some(index) = self.meteor_hud.controls.selected {
                        let path = format!("/elements/{index}/source_box");
                        if let Ok(mut box_) = serde_json::from_value::<meteor_client::hud_box::HudBox>(
                            self.meteor_hud
                                .config
                                .pointer(&path)
                                .cloned()
                                .unwrap_or(Value::Null),
                        ) {
                            let size = self
                                .meteor_hud
                                .bounds
                                .iter()
                                .find(|(n, _)| *n == index)
                                .map_or([0, 0], |(_, r)| {
                                    [
                                        (r[2] * self.meteor_hud.scale).round() as i32,
                                        (r[3] * self.meteor_hud.scale).round() as i32,
                                    ]
                                });
                            if key.ends_with("/x_anchor") {
                                box_.set_x_anchor(
                                    serde_json::from_value(json!(options[(_i + 1) % 3])).unwrap(),
                                    self.meteor_hud.layout_viewport,
                                    size,
                                )
                            } else {
                                box_.set_y_anchor(
                                    serde_json::from_value(json!(options[(_i + 1) % 3])).unwrap(),
                                    self.meteor_hud.layout_viewport,
                                    size,
                                )
                            }
                            self.meteor_hud_edit_value(&path, serde_json::to_value(box_).unwrap());
                        }
                    }
                } else {
                    self.meteor_hud.controls.edit = Some((
                        key,
                        value
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| value.to_string()),
                        true,
                    ));
                }
            }
        }
        true
    }
    pub(in crate::game) fn meteor_hud_draw_controls(&mut self, batch: &mut HudBatch) {
        if !self.meteor_hud.editor {
            return;
        }
        let atlas = &self.gui.atlas;
        let w = batch.width;
        let h = batch.height;
        self.meteor_hud.controls.hits.clear();
        for (key, label, x, width) in [
            ("add-text", "+ Text", 205., 57.),
            ("edit", "Settings", 265., 60.),
            ("duplicate", "Copy", 328., 45.),
            ("delete", "Delete", 376., 55.),
        ] {
            let rect = [x, 3., width, 23.];
            batch.fill(atlas, x, 3., width, 23., rgba(60, 40, 75, 255));
            batch.text(atlas, label, x + 3., 11., rgba(255, 255, 255, 255), false);
            self.meteor_hud
                .controls
                .hits
                .push((key.into(), rect, Value::Null));
        }
        if !self.meteor_hud.controls.panel {
            return;
        }
        let width = (w - 24.).min(390.);
        let x = (w - width) / 2.;
        let top = 34.;
        let bottom = h - 23.;
        batch.fill(atlas, x, top, width, bottom - top, rgba(22, 20, 26, 250));
        let mut rows = vec![];
        if let Some(i) = self.meteor_hud.controls.selected {
            if let Some(element) = self.meteor_hud.config["elements"].get(i) {
                rows.push((
                    format!("/elements/{i}/enabled"),
                    "Enabled".to_owned(),
                    element["enabled"].clone(),
                ));
                rows.push((
                    format!("/elements/{i}/auto_anchors"),
                    "Automatic anchors".to_owned(),
                    element["auto_anchors"].clone(),
                ));
                for group in ["source_box", "text"] {
                    if let Some(fields) = element[group].as_object() {
                        for (key, value) in fields {
                            if let Some(color) = value.as_object() {
                                for (component, v) in color {
                                    rows.push((
                                        format!("/elements/{i}/{group}/{key}/{component}"),
                                        format!("{key} {component}"),
                                        v.clone(),
                                    ));
                                }
                            } else {
                                rows.push((
                                    format!("/elements/{i}/{group}/{key}"),
                                    key.replace('_', " "),
                                    value.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        for key in ["text_scale", "hide_in_menus", "border", "snapping_range"] {
            rows.push((
                format!("/{key}"),
                format!("HUD {}", key.replace('_', " ")),
                self.meteor_hud.config[key].clone(),
            ));
        }
        if let Some(colors) = self.meteor_hud.config["text_colors"].as_array() {
            for (i, color) in colors.iter().enumerate() {
                if let Some(fields) = color.as_object() {
                    for (channel, value) in fields {
                        rows.push((
                            format!("/text_colors/{i}/{channel}"),
                            format!("HUD color {i} {channel}"),
                            value.clone(),
                        ));
                    }
                }
            }
        }
        for (i, (path, label, value)) in rows.iter().enumerate() {
            let y = top + 4. + i as f32 * 18. - self.meteor_hud.controls.scroll;
            if y < top || y + 18. > bottom {
                continue;
            }
            let display = if let Some((p, t, _)) = &self.meteor_hud.controls.edit
                && p == path
            {
                format!("{t}_")
            } else {
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string())
            };
            let mut line = format!("{label}: {display}");
            while HudBatch::text_width(atlas, &line) > width - 10. && !line.is_empty() {
                line.pop();
            }
            batch.fill(atlas, x + 3., y, width - 6., 17., rgba(48, 43, 55, 255));
            batch.text(
                atlas,
                &line,
                x + 6.,
                y + 5.,
                rgba(235, 235, 235, 255),
                false,
            );
            self.meteor_hud.controls.hits.push((
                path.clone(),
                [x + 3., y, width - 6., 17.],
                value.clone(),
            ));
        }
        self.meteor_hud.controls.max_scroll =
            (rows.len() as f32 * 18. + 8. - (bottom - top)).max(0.);
    }
}
