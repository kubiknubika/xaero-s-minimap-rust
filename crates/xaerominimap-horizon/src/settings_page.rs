//! Horizon options page generated from the ported 115-entry
//! `MinimapConfigOptionUIRegister` table.
//!
//! The page uses Horizon's public `pages` registry: the engine draws the rows
//! with its own widgets. Every binding becomes a Toggle, Cycle, Slider or
//! Field row; the two HUD-visible toggles (display map / radar) are mirrored
//! into [`MinimapState`] so changes take effect without a restart.

use std::sync::{Arc, Mutex};

use horizon_hooks::pages::{Act, Label, Page, Place, Row};
use horizon_hooks::TooLate;
use xaerominimap_settings::{ConfigUiType, Settings, UI_BINDINGS};

use crate::MinimapState;

/// Translation key of one Java option field (`DISPLAY_MINIMAP` →
/// `gui.xaero_display_minimap`). The engine falls back to the key when no
/// translation is installed.
fn label_for(option: &str) -> &'static str {
    // The bindings are &'static str values, so leak the derived key once per
    // distinct option. The page rebuilds rows while it is open; the number of
    // leaked strings is bounded by the 115 static bindings.
    Box::leak(format!("gui.xaero_{}", option.to_ascii_lowercase()).into_boxed_str())
}

/// The minimap's settings page.
pub struct MinimapSettingsPage {
    settings: Arc<Mutex<Settings>>,
    state: Arc<MinimapState>,
}

impl MinimapSettingsPage {
    /// The default a toggle starts from when the setting file has no entry.
    fn toggle_default(&self, option: &str) -> bool {
        match option {
            "DISPLAY_MINIMAP" => self.state.is_enabled(),
            "DISPLAY_RADAR" => self.state.is_radar_enabled(),
            _ => false,
        }
    }

    pub fn new(settings: Arc<Mutex<Settings>>, state: Arc<MinimapState>) -> Self {
        MinimapSettingsPage { settings, state }
    }

    pub fn settings(&self) -> &Arc<Mutex<Settings>> {
        &self.settings
    }

    /// Mirrors the two HUD options into `MinimapState`.
    pub fn sync_state(&self) {
        let settings = self.settings.lock().unwrap();
        let mut view = self.state.view();
        view.enabled = settings.bool("DISPLAY_MINIMAP", view.enabled);
        view.radar_enabled = settings.bool("DISPLAY_RADAR", view.radar_enabled);
        drop(settings);
        self.state.set_view(view);
    }
}

impl Page for MinimapSettingsPage {
    fn id(&self) -> &'static str {
        "xaerominimap:settings"
    }

    fn title(&self) -> &'static str {
        "gui.xaero_minimap_settings"
    }

    fn place(&self) -> Place {
        Place::HorizonSettings
    }

    fn rows(&self, out: &mut Vec<Row>) {
        let settings = self.settings.lock().unwrap();
        out.push(Row::Heading("gui.xaero_minimap_settings"));
        for binding in UI_BINDINGS {
            let label = label_for(binding.option);
            match binding.ui_type {
                ConfigUiType::Toggle => out.push(Row::Toggle {
                    label,
                    on: settings.bool(binding.option, self.toggle_default(binding.option)),
                }),
                ConfigUiType::IntIndexedButton | ConfigUiType::IntIndexedSlider => {
                    let value = settings.int(binding.option, 0);
                    let text = Label::Value(value.to_string());
                    if binding.ui_type == ConfigUiType::IntIndexedSlider {
                        out.push(Row::Slider {
                            label,
                            value: (value.clamp(0, 100) as f32) / 100.0,
                            text,
                        });
                    } else {
                        out.push(Row::Cycle { label, value: text });
                    }
                }
                ConfigUiType::DoubleIndexedSlider => {
                    let value = settings.float(binding.option, 0.0);
                    out.push(Row::Slider {
                        label,
                        value: value.clamp(0.0, 1.0),
                        text: Label::Value(format!("{value:.2}")),
                    });
                }
                ConfigUiType::StringStringEdit => {
                    out.push(Row::field(label, settings.string(binding.option, "")).max(256))
                }
                ConfigUiType::InfoDisplayConfigEditor
                | ConfigUiType::RadarCategoriesEditor => {
                    out.push(Row::button(label));
                }
            }
        }
    }

    fn act(&self, row: usize, act: Act) {
        // Row 0 is the heading; the rest are in UI_BINDINGS order.
        let Some(binding) = UI_BINDINGS.get(row.saturating_sub(1)) else {
            return;
        };
        match (&act, binding.ui_type) {
            (Act::Press, ConfigUiType::Toggle) => {
                let mut settings = self.settings.lock().unwrap();
                let next = !settings.bool(binding.option, self.toggle_default(binding.option));
                settings.set(binding.option, next);
            }
            (Act::Slide(value), ConfigUiType::IntIndexedSlider) => {
                let mut settings = self.settings.lock().unwrap();
                settings.set(binding.option, (value.clamp(0.0, 1.0) * 100.0).round() as i32);
            }
            (Act::Slide(value), ConfigUiType::DoubleIndexedSlider) => {
                let mut settings = self.settings.lock().unwrap();
                settings.set(binding.option, value.clamp(0.0, 1.0));
            }
            (Act::Edit(text), ConfigUiType::StringStringEdit) => {
                let mut settings = self.settings.lock().unwrap();
                settings.set(binding.option, xaerominimap_settings::SettingValue::Str(text.clone()));
            }
            (Act::Press | Act::PressBack, ConfigUiType::IntIndexedButton) => {
                let mut settings = self.settings.lock().unwrap();
                let current = settings.int(binding.option, 0);
                let next = if matches!(act, Act::PressBack) {
                    (current - 1).max(0)
                } else {
                    current + 1
                };
                settings.set(binding.option, next);
            }
            _ => return,
        }
        self.sync_state();
    }

    fn tooltip(&self, row: usize) -> Option<Label> {
        UI_BINDINGS
            .get(row.saturating_sub(1))
            .map(|binding| Label::Value(format!("{} / {}", binding.owner, binding.option)))
    }
}

/// Registers the page. Like the HUD registry, the pages registry seals at
/// game start; call this from the trait mod's `hooks()`.
pub fn register_settings_page(
    settings: Arc<Mutex<Settings>>,
    state: Arc<MinimapState>,
) -> Result<Arc<MinimapSettingsPage>, TooLate> {
    let page = Arc::new(MinimapSettingsPage::new(settings, state));
    page.sync_state();
    horizon_hooks::pages::register(Box::leak(Box::new(MinimapSettingsPage::new(
        page.settings.clone(),
        page.state.clone(),
    ))))?;
    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> MinimapSettingsPage {
        MinimapSettingsPage::new(
            Arc::new(Mutex::new(Settings::new())),
            Arc::new(MinimapState::new()),
        )
    }

    #[test]
    fn page_exposes_every_registered_binding() {
        let page = page();
        let mut rows = Vec::new();
        Page::rows(&page, &mut rows);
        assert_eq!(rows.len(), UI_BINDINGS.len() + 1);
        assert_eq!(Page::id(&page), "xaerominimap:settings");
        assert_eq!(Page::place(&page), Place::HorizonSettings);
    }

    #[test]
    fn toggling_display_minimap_mirrors_the_hud_state() {
        let state = Arc::new(MinimapState::new());
        let page = MinimapSettingsPage::new(
            Arc::new(Mutex::new(Settings::new())),
            state.clone(),
        );
        // Row 1 is DISPLAY_MINIMAP, the first Toggle binding.
        Page::act(&page, 1, Act::Press);
        assert!(!state.is_enabled());
        Page::act(&page, 1, Act::Press);
        assert!(state.is_enabled());
    }

    #[test]
    fn field_edits_are_stored_in_settings() {
        let page = page();
        // Row 2 is MINIMAP_ITEM (StringStringEdit).
        Page::act(&page, 2, Act::Edit("custom_item".into()));
        let settings = page.settings.lock().unwrap();
        assert_eq!(settings.string("MINIMAP_ITEM", ""), "custom_item");
    }
}
