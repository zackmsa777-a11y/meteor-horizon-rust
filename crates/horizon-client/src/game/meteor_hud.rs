// SPDX-License-Identifier: GPL-3.0-only
// Native subset of Meteor Hud, HudEditorScreen and text presets.
#[path = "meteor_hud_editor.rs"]
mod editor;
use super::Game;
use horizon_client::hud::{HudBatch, rgba};
use serde_json::{Value, json};
use winit::event::MouseButton;

#[derive(Default)]
pub(super) struct State {
    pub(super) editor: bool,
    pub(super) controls: editor::Editor,
    pub(super) config: Value,
    #[cfg(feature = "meteor")]
    text_cache: std::collections::HashMap<String, meteor_client::hud_text::Runtime>,
    bounds: Vec<(usize, [f32; 4])>,
    drag: Option<(usize, (f32, f32), [f64; 2])>,
    source_drag: Option<Value>,
    #[cfg(feature = "meteor")]
    snapper: meteor_client::hud_snapper::Snapper,
    scale: f32,
    viewport: [i32; 2],
    error: Option<String>,
}
fn inside(r: [f32; 4], p: (f32, f32)) -> bool {
    p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3]
}
impl Game<'_> {
    pub(super) fn meteor_hud_editor_open(&mut self) {
        self.meteor_hud.config = self.meteor_status()["config"]["hud"].clone();
        self.meteor_hud.editor = true;
        self.meteor_hud.controls = Default::default();
        self.meteor_hud.drag = None;
        self.meteor_hud.source_drag = None;
        self.meteor_hud.error = None;
    }
    pub(super) fn meteor_hud_editor_close(&mut self) {
        self.meteor_hud.drag = None;
        self.meteor_hud.source_drag = None;
        self.meteor_hud.editor = false;
    }
    fn meteor_hud_commit(&mut self) {
        self.meteor_hud.error = self
            .meteor_hud_save_config(self.meteor_hud.config.clone())
            .err();
    }
    pub(super) fn meteor_hud_mouse(&mut self, button: MouseButton, pressed: bool) -> bool {
        if !self.meteor_hud.editor {
            return false;
        }
        if (self.meteor_hud.controls.panel
            || (pressed
                && self
                    .meteor_hud
                    .controls
                    .hits
                    .iter()
                    .any(|(_, r, _)| inside(*r, self.cursor))))
            && self.meteor_hud_control_mouse(button, pressed)
        {
            return true;
        }
        if !pressed {
            if button == MouseButton::Left
                && let Some((i, start, offset)) = self.meteor_hud.drag.take()
            {
                self.meteor_hud_drag_move(i, start, offset);
                self.meteor_hud_commit();
            }
            return true;
        }
        if self.cursor.1 < 29. && button == MouseButton::Left {
            if self.cursor.0 < 80. {
                self.meteor_hud_editor_close();
            } else if self.cursor.0 < 200. {
                self.meteor_hud.config["active"] =
                    json!(!self.meteor_hud.config["active"].as_bool().unwrap_or(false));
                self.meteor_hud_commit();
            }
            return true;
        }
        let hit = self
            .meteor_hud
            .bounds
            .iter()
            .rev()
            .find(|(_, r)| inside(*r, self.cursor))
            .map(|(i, _)| *i);
        if let Some(i) = hit {
            self.meteor_hud.controls.selected = Some(i);
            let element = &mut self.meteor_hud.config["elements"][i];
            if button == MouseButton::Right {
                element["enabled"] = json!(!element["enabled"].as_bool().unwrap_or(true));
                self.meteor_hud_commit();
            } else if button == MouseButton::Left {
                self.meteor_hud.source_drag =
                    element.get("source_box").filter(|b| b.is_object()).cloned();
                self.meteor_hud.drag = Some((
                    i,
                    self.cursor,
                    [
                        element["offset"][0].as_f64().unwrap_or(0.),
                        element["offset"][1].as_f64().unwrap_or(0.),
                    ],
                ));
            }
        }
        true
    }
    pub(super) fn draw_meteor_hud(&mut self, batch: &mut HudBatch) {
        let editor = self.meteor_hud.editor && self.meteor_gui.open;
        let status = self.meteor_status();
        if !status["enabled"].as_bool().unwrap_or(false) || self.player.is_none() {
            return;
        }
        if !editor
            && (!status["config"]["hud"]["active"].as_bool().unwrap_or(false)
                || self.ui.hide_gui
                || (status["config"]["hud"]["hide_in_menus"]
                    .as_bool()
                    .unwrap_or(false)
                    && self.screen != super::Screen::Playing)
                || self.debug
                || self.meteor_hide_hud())
        {
            return;
        }
        if editor && let Some((i, start, offset)) = self.meteor_hud.drag {
            self.meteor_hud_drag_move(i, start, offset);
        }
        #[allow(unused_mut)]
        let mut config = if editor {
            self.meteor_hud.config.clone()
        } else {
            status["config"]["hud"].clone()
        };
        #[cfg(feature="meteor")]
        if editor {
            let rgb=self.meteor.rainbow.rgb;
            let update=|color:&mut Value|{if color["rainbow"].as_bool().unwrap_or(false){for (key,value) in ["r","g","b"].into_iter().zip(rgb){color[key]=json!(value);}}};
            if let Some(colors)=config["text_colors"].as_array_mut(){for color in colors{update(color)}}
            if let Some(elements)=config["elements"].as_array_mut(){for element in elements{if element["text"].is_object(){update(&mut element["text"]["background_color"]);}}}
        }
        let Some(elements) = config["elements"].as_array() else {
            return;
        };
        #[cfg(feature = "meteor")]
        self.meteor_hud_update_text(false);
        let facts = self.meteor_hud_facts();
        let atlas = &self.gui.atlas;
        self.meteor_hud.scale = batch.scale;
        self.meteor_hud.viewport = self.renderer.as_ref().map_or(
            [
                (batch.width * batch.scale) as i32,
                (batch.height * batch.scale) as i32,
            ],
            |r| [r.config.width as i32, r.config.height as i32],
        );
        let w = batch.width as f32;
        let h = batch.height as f32;
        self.meteor_hud.bounds.clear();
        if editor {
            batch.fill(atlas, 0., 0., w, h, rgba(0, 0, 0, 65));
        }
        for (index, element) in elements.iter().enumerate() {
            let enabled = element["enabled"].as_bool().unwrap_or(true);
            if !enabled && !editor {
                continue;
            }
            let instance_id = element["id"].as_str().unwrap_or("");
            let id = element["kind"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or(instance_id);
            #[cfg(feature = "meteor")]
            if id == "text" {
                if let Some(runtime) = self.meteor_hud.text_cache.get(instance_id) {
                    if let Some(rect) = draw_source_text(
                        batch,
                        atlas,
                        element,
                        &config,
                        runtime,
                        editor,
                        enabled,
                        self.meteor_hud.viewport,
                    ) {
                        if editor {
                            self.meteor_hud.bounds.push((index, rect));
                        }
                    }
                }
                continue;
            }
            let mut lines = match id {
                "watermark" => vec!["Meteor Native".to_string()],
                "fps" => vec![format!("FPS: {:.0}", self.fps)],
                "position" => {
                    let p = self.player.as_ref().unwrap().position;
                    vec![format!(
                        "Pos: {:.0}, {:.0}, {:.0}",
                        p.x.floor(),
                        p.y.floor(),
                        p.z.floor()
                    )]
                }
                "active-modules" => status["config"]["modules"]["modules"]
                    .as_array()
                    .map(|modules| {
                        modules
                            .iter()
                            .filter(|m| m["active"].as_bool().unwrap_or(false))
                            .filter_map(|m| m["title"].as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
                _ => match facts.get(id) {
                    Some(line) => vec![line.clone()],
                    None => continue,
                },
            };
            if id == "active-modules" {
                lines.sort_by(|a, b| {
                    HudBatch::text_width(atlas, b)
                        .total_cmp(&HudBatch::text_width(atlas, a))
                        .then_with(|| a.cmp(b))
                });
                if lines.is_empty() && editor {
                    lines.push("Active Modules (none)".into());
                }
            }
            if lines.is_empty() {
                continue;
            }
            let width = lines
                .iter()
                .map(|line| HudBatch::text_width(atlas, line))
                .fold(0., f32::max)
                + 4.;
            let height = lines.len() as f32 * 12.;
            let ax = element["anchor"][0].as_f64().unwrap_or(0.) as f32;
            let ay = element["anchor"][1].as_f64().unwrap_or(0.) as f32;
            let x = (ax * (w - width) + element["offset"][0].as_f64().unwrap_or(0.) as f32)
                .clamp(0., (w - width).max(0.));
            let top = if editor { 29. } else { 0. };
            let bottom = if editor { 20. } else { 0. };
            let y = (top
                + ay * (h - top - bottom - height)
                + element["offset"][1].as_f64().unwrap_or(0.) as f32)
                .clamp(top, (h - bottom - height).max(top));
            if editor {
                batch.fill(
                    atlas,
                    x - 1.,
                    y - 1.,
                    width + 2.,
                    height + 2.,
                    if enabled {
                        rgba(145, 61, 226, 100)
                    } else {
                        rgba(100, 100, 100, 90)
                    },
                );
                self.meteor_hud.bounds.push((index, [x, y, width, height]));
            }
            for (row, line) in lines.iter().enumerate() {
                let tx = if ax > 0.5 {
                    x + width - 2. - HudBatch::text_width(atlas, line)
                } else {
                    x + 2.
                };
                batch.text(
                    atlas,
                    line,
                    tx,
                    y + 2. + row as f32 * 12.,
                    if !enabled {
                        rgba(150, 150, 150, 255)
                    } else if id == "active-modules" {
                        rgba(190, 130, 255, 255)
                    } else {
                        rgba(255, 255, 255, 255)
                    },
                    true,
                );
            }
        }
        if editor {
            batch.fill(atlas, 0., 0., w, 29., rgba(20, 20, 20, 245));
            batch.text(atlas, "< Modules", 7., 10., rgba(255, 255, 255, 255), false);
            batch.text(
                atlas,
                if config["active"].as_bool().unwrap_or(false) {
                    "HUD: Enabled"
                } else {
                    "HUD: Disabled"
                },
                85.,
                10.,
                rgba(190, 130, 255, 255),
                false,
            );
            let help =
                self.meteor_hud
                    .error
                    .as_deref()
                    .unwrap_or(if self.meteor_hud.controls.panel {
                        "Vanilla font | Enter: Save | Esc: Back | Scroll for settings"
                    } else {
                        "Drag to move | Right-click to toggle | Esc: Modules"
                    });
            batch.fill(atlas, 0., h - 20., w, 20., rgba(20, 20, 20, 245));
            batch.text(atlas, help, 6., h - 13., rgba(220, 220, 220, 255), false);
            self.meteor_hud_draw_controls(batch);
        }
    }
}

impl Game<'_> {
    fn meteor_hud_drag_move(&mut self, i: usize, start: (f32, f32), offset: [f64; 2]) {
        #[cfg_attr(not(feature = "meteor"), allow(unused_mut))]
        if let Some(mut source) = self.meteor_hud.source_drag.clone() {
            #[cfg(feature = "meteor")]
            if let Ok(mut box_) =
                serde_json::from_value::<meteor_client::hud_box::HudBox>(source.clone())
            {
                let size = self
                    .meteor_hud
                    .bounds
                    .iter()
                    .find(|(index, _)| *index == i)
                    .map_or([0, 0], |(_, rect)| {
                        [
                            (rect[2] * self.meteor_hud.scale).round() as i32,
                            (rect[3] * self.meteor_hud.scale).round() as i32,
                        ]
                    });
                let peers = self
                    .meteor_hud
                    .bounds
                    .iter()
                    .filter(|(index, _)| *index != i)
                    .map(|(index, r)| {
                        (
                            *index,
                            [
                                (r[0] * self.meteor_hud.scale).round() as i32,
                                ((r[1] - 29.) * self.meteor_hud.scale).round() as i32,
                                (r[2] * self.meteor_hud.scale).round() as i32,
                                (r[3] * self.meteor_hud.scale).round() as i32,
                            ],
                        )
                    })
                    .collect::<Vec<_>>();
                self.meteor_hud.snapper.step(
                    &mut box_,
                    size,
                    [
                        ((self.cursor.0 - start.0) * self.meteor_hud.scale).round() as i32,
                        ((self.cursor.1 - start.1) * self.meteor_hud.scale).round() as i32,
                    ],
                    self.meteor_hud.viewport,
                    self.meteor_hud.config["elements"][i]["auto_anchors"]
                        .as_bool()
                        .unwrap_or(true),
                    self.meteor_hud.config["border"].as_i64().unwrap_or(4) as i32,
                    self.meteor_hud.config["snapping_range"]
                        .as_i64()
                        .unwrap_or(10) as i32,
                    &peers,
                );
                source = serde_json::to_value(box_).unwrap();
            }
            self.meteor_hud.source_drag = Some(source.clone());
            if self.meteor_hud.drag.is_some() {
                self.meteor_hud.drag = Some((i, self.cursor, offset));
            }
            self.meteor_hud.config["elements"][i]["source_box"] = source;
        } else {
            self.meteor_hud.config["elements"][i]["offset"] = json!([
                offset[0] + (self.cursor.0 - start.0) as f64,
                offset[1] + (self.cursor.1 - start.1) as f64
            ]);
        }
    }
    #[cfg(feature = "meteor")]
    pub(super) fn meteor_hud_update_text(&mut self, tick: bool) {
        let editor = self.meteor_hud.editor && self.meteor_gui.open;
        let config = if editor {
            serde_json::from_value(self.meteor_hud.config.clone()).ok()
        } else {
            Some(self.meteor.client.config.hud.clone())
        };
        let Some(config): Option<meteor_client::hud::Config> = config else {
            return;
        };
        if !editor && !config.active {
            return;
        }
        if !config
            .elements
            .iter()
            .any(|e| e.kind() == "text" && (e.enabled || editor))
        {
            return;
        }
        self.meteor_starscript_refresh();
        self.meteor_hud
            .text_cache
            .retain(|id, _| config.elements.iter().any(|e| &e.id == id));
        for element in config
            .elements
            .iter()
            .filter(|e| e.kind() == "text" && (e.enabled || editor))
        {
            if let Some(settings) = &element.text {
                self.meteor_hud
                    .text_cache
                    .entry(element.id.clone())
                    .or_default()
                    .update(settings, &self.meteor_macros.environment, tick);
            }
        }
    }
}

#[cfg(feature = "meteor")]
fn draw_source_text(
    batch: &mut HudBatch,
    atlas: &horizon_client::hud::HudAtlas,
    element: &Value,
    config: &Value,
    runtime: &meteor_client::hud_text::Runtime,
    editor: bool,
    enabled: bool,
    viewport: [i32; 2],
) -> Option<[f32; 4]> {
    use meteor_client::{hud_box::HudBox, hud_text::TextSettings};
    let settings: TextSettings = serde_json::from_value(element["text"].clone()).ok()?;
    let scale = settings.effective_scale(config["text_scale"].as_f64().unwrap_or(1.)) as f32;
    let factor = scale * 2. / batch.scale;
    let string = runtime
        .output
        .iter()
        .map(|s| s.text.as_str())
        .collect::<String>();
    let empty = string.trim().is_empty();
    let text_width = if empty {
        100. / (scale * 2.)
    } else {
        HudBatch::text_width(atlas, &string) + if settings.shadow { 1. } else { 0. }
    };
    let width = (text_width * scale * 2. + settings.border as f32 * 2.)
        .ceil()
        .max(0.);
    let height = ((9. + if settings.shadow { 1. } else { 0. }) * scale * 2.
        + settings.border as f32 * 2.)
        .ceil()
        .max(0.);

    let source_box: HudBox =
        serde_json::from_value(element["source_box"].clone()).unwrap_or_default();
    let [px, py] = source_box.render(viewport, [width as i32, height as i32]);
    let [x, y] = [
        px as f32 / batch.scale,
        py as f32 / batch.scale + if editor { 29. } else { 0. },
    ];
    let rect = [x, y, width / batch.scale, height / batch.scale];
    if editor {
        batch.fill(
            atlas,
            x - 1.,
            y - 1.,
            rect[2] + 2.,
            rect[3] + 2.,
            if enabled {
                rgba(145, 61, 226, 100)
            } else {
                rgba(100, 100, 100, 90)
            },
        );
    }
    if runtime.visible {
        if settings.background {
            let c = settings.background_color;
            batch.fill(atlas, x, y, rect[2], rect[3], rgba(c.r, c.g, c.b, c.a));
        }
        let origin = [
            x + settings.border as f32 / batch.scale,
            y + settings.border as f32 / batch.scale,
        ];
        let mut tx = origin[0];
        for section in &runtime.output {
            let color = section
                .index
                .try_into()
                .ok()
                .and_then(|i: usize| config["text_colors"].get(i))
                .and_then(|v| {
                    serde_json::from_value::<meteor_client::colors::SettingColor>(v.clone()).ok()
                })
                .unwrap_or(meteor_client::colors::SettingColor::new(255, 255, 255, 255));
            // Source VanillaTextRenderer rounds each draw origin after adding half a glyph unit.
            let draw_x = (tx / factor + 0.5).trunc() * factor;
            let draw_y = (origin[1] / factor + 0.5).trunc() * factor;
            let begin = batch.vertices.len();
            batch.text(
                atlas,
                &section.text,
                draw_x,
                draw_y,
                rgba(color.r, color.g, color.b, color.a),
                settings.shadow,
            );
            batch.scale_since(begin, draw_x, draw_y, factor, factor);
            tx += (HudBatch::text_width(atlas, &section.text) - 0.5) * factor;
        }
    }
    Some(rect)
}
