//! Preliminary multi-select with optional search; tags/chips, groups and full
//! parity remain open. See `docs/SELECT.md`.
//!
//! A [`MultiSelect`] is the multi-choice form of [`crate::Select`]: the caller
//! owns a `Vec` of selected option IDs, activating a row toggles its membership
//! and the popup stays open, and the field summarises the selection. It reuses
//! [`crate::SelectOption`] so single and multi selects can share option lists.
use crate::select::SelectOption;
use egui::{ComboBox, Id, Response, Ui};

/// A caller-owned multi-choice select. IDs must be stable and globally unique.
pub struct MultiSelect<'a> {
    id: Id,
    label: &'a str,
    placeholder: &'a str,
    options: &'a [SelectOption<'a>],
    selected: &'a mut Vec<Id>,
    enabled: bool,
    liquid: Option<crate::liquid::LiquidStyle>,
    searchable: bool,
    search_hint: &'a str,
    no_results: &'a str,
    count_summary: Option<&'a dyn Fn(usize) -> String>,
    menu_frame: Option<egui::Frame>,
}

impl<'a> MultiSelect<'a> {
    pub fn new(
        id: Id,
        label: &'a str,
        placeholder: &'a str,
        options: &'a [SelectOption<'a>],
        selected: &'a mut Vec<Id>,
    ) -> Self {
        Self {
            id,
            label,
            placeholder,
            options,
            selected,
            enabled: true,
            liquid: None,
            searchable: false,
            search_hint: "Search",
            no_results: "No results",
            count_summary: None,
            menu_frame: None,
        }
    }

    /// Configure popup motion and trigger separation without replacing egui input.
    pub fn liquid(mut self, motion: crate::liquid::LiquidStyle) -> Self {
        self.liquid = Some(motion);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Override the dropdown's corners, colors, padding and shadow.
    pub fn menu_frame(mut self, frame: egui::Frame) -> Self {
        self.menu_frame = Some(frame);
        self
    }

    /// Show a search field at the top of the popup and filter options by a
    /// case-insensitive substring match on their labels. The query lives in
    /// egui's temporary data keyed by this select's ID and clears on close.
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// The search field's hint and accessible name. Defaults to "Search".
    pub fn search_hint(mut self, hint: &'a str) -> Self {
        self.search_hint = hint;
        self
    }

    /// Text shown when the filter matches no option. Defaults to "No results".
    pub fn no_results(mut self, text: &'a str) -> Self {
        self.no_results = text;
        self
    }

    /// Localize the summary shown when more than three options are selected.
    /// The default renders `"N selected"` in English.
    pub fn count_summary(mut self, summary: &'a dyn Fn(usize) -> String) -> Self {
        self.count_summary = Some(summary);
        self
    }

    fn summary(&self) -> String {
        let labels: Vec<&str> = self
            .options
            .iter()
            .filter(|option| self.selected.contains(&option.id))
            .map(|option| option.label)
            .collect();
        match labels.len() {
            0 => self.placeholder.to_owned(),
            1..=3 => labels.join(", "),
            count => self
                .count_summary
                .map_or_else(|| format!("{count} selected"), |summary| summary(count)),
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let id = self.id;
        let label = self.label;
        ui.scope_builder(egui::UiBuilder::new().id(id), |ui| {
            ui.vertical(|ui| {
                let label_response = ui.label(label);
                self.show_inner(ui).labelled_by(label_response.id)
            })
            .inner
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui) -> Response {
        let summary = self.summary();
        let before = self.selected.clone();
        let can_change = self.enabled && ui.is_enabled();
        let popup_focus_key = self.id.with("temivu-multi-select-popup-focus");
        let previous_popup_focus = ui
            .ctx()
            .data(|data| data.get_temp::<egui::Id>(popup_focus_key));
        let focused_before = ui.ctx().memory(|memory| memory.focused());
        let escape = ui.input(|input| input.key_pressed(egui::Key::Escape));
        let mut focused_popup: Option<egui::Id> = None;
        let width = ui.available_width().clamp(0.0, 360.0);
        let search_key = self.id.with("temivu-multi-select-search");
        let mut was_open = false;
        let mut widget_id = None;
        let output = ui
            .add_enabled_ui(self.enabled, |ui| {
                ui.spacing_mut().button_padding = egui::vec2(12.0, 9.0);
                let id = ui.make_persistent_id(egui::IdSalt::new(self.id));
                widget_id = Some(id);
                was_open = ComboBox::is_open(ui.ctx(), id);
                let min_height = ui.spacing().interact_size.y.max(40.0);
                ui.spacing_mut().interact_size.y = min_height;
                if !can_change && was_open {
                    // egui 0.36 derives this popup ID from the combo widget ID.
                    egui::Popup::close_id(ui.ctx(), id.with("popup"));
                }
                let just_opened = !was_open;
                crate::liquid_combo::LiquidCombo::new(id, &summary, width)
                    .liquid(self.liquid)
                    .frame(self.menu_frame)
                    .show_ui(ui, |ui| {
                        let mut query = if self.searchable {
                            ui.ctx().data(|data| {
                                data.get_temp::<String>(search_key).unwrap_or_default()
                            })
                        } else {
                            String::new()
                        };
                        let mut search_has_focus = false;
                        if self.searchable {
                            let search = ui.add(
                                egui::TextEdit::singleline(&mut query)
                                    .vertical_align(egui::Align::Center)
                                    .margin(egui::Margin::symmetric(8, 6))
                                    .hint_text(self.search_hint)
                                    .desired_width(f32::INFINITY),
                            );
                            ui.ctx().accesskit_node_builder(search.id, |node| {
                                node.set_label(self.search_hint);
                            });
                            if just_opened {
                                search.request_focus();
                            }
                            search_has_focus = search.has_focus();
                            if search_has_focus {
                                focused_popup = Some(search.id);
                            }
                            ui.ctx().data_mut(|data| {
                                data.insert_temp(search_key, query.clone());
                            });
                        }
                        let needle = query.to_lowercase();
                        let matches = |option: &SelectOption<'_>| {
                            needle.is_empty() || option.label.to_lowercase().contains(&needle)
                        };
                        let mut visible_buttons: Vec<(egui::Id, bool)> = Vec::new();
                        let mut shown = 0usize;
                        for option in self.options.iter().filter(|option| matches(option)) {
                            shown += 1;
                            let selected = self.selected.contains(&option.id);
                            let response = ui
                                .scope_builder(
                                    egui::UiBuilder::new().id(self.id.with(option.id)),
                                    |ui| {
                                        let text_color = if selected {
                                            ui.visuals().selection.stroke.color
                                        } else {
                                            ui.visuals().text_color()
                                        };
                                        ui.visuals_mut().override_text_color = Some(text_color);
                                        ui.add_enabled(
                                            can_change && option.enabled,
                                            egui::Button::selectable(selected, option.label)
                                                .wrap()
                                                .min_size(egui::vec2(ui.available_width(), 44.0)),
                                        )
                                    },
                                )
                                .inner;
                            visible_buttons.push((response.id, can_change && option.enabled));
                            if focused_before == Some(response.id) {
                                focused_popup = Some(response.id);
                            }
                            ui.ctx().accesskit_node_builder(response.id, |node| {
                                node.set_role(egui::accesskit::Role::ListBoxOption);
                                node.set_label(option.label);
                                node.clear_toggled();
                                node.set_selected(selected);
                            });
                            if can_change && option.enabled && response.clicked() {
                                if selected {
                                    self.selected.retain(|id| *id != option.id);
                                } else {
                                    self.selected.push(option.id);
                                }
                            }
                        }
                        if shown == 0 {
                            ui.label(self.no_results);
                        }
                        if self.searchable && search_has_focus {
                            let (down, up) = ui.input(|input| {
                                (
                                    input.key_pressed(egui::Key::ArrowDown),
                                    input.key_pressed(egui::Key::ArrowUp),
                                )
                            });
                            let enabled_buttons = || {
                                visible_buttons
                                    .iter()
                                    .filter_map(|(id, enabled)| enabled.then_some(*id))
                            };
                            if down && let Some(first) = enabled_buttons().next() {
                                ui.memory_mut(|memory| memory.request_focus(first));
                            }
                            if up && let Some(last) = enabled_buttons().next_back() {
                                ui.memory_mut(|memory| memory.request_focus(last));
                            }
                        }
                    })
            })
            .inner;
        let mut response = output.response;
        let open_now = widget_id.is_some_and(|widget_id| ComboBox::is_open(ui.ctx(), widget_id));
        if !open_now {
            ui.ctx().data_mut(|data| {
                data.remove::<String>(search_key);
                data.remove::<egui::Id>(popup_focus_key);
            });
        } else if let Some(id) = focused_popup {
            ui.ctx().data_mut(|data| {
                data.insert_temp(popup_focus_key, id);
            });
        }
        // Restore field focus when Escape dismisses a popup that had keyboard
        // focus, including the search field. Clicks elsewhere must not steal it.
        let escape_from_popup = escape
            && was_open
            && (focused_before == Some(response.id)
                || focused_before.is_none()
                || previous_popup_focus.is_some_and(|previous| {
                    focused_before == Some(previous) || focused_popup == Some(previous)
                }));
        if can_change && escape_from_popup {
            response.request_focus();
        }
        if *self.selected != before {
            response.mark_changed();
        }
        let accesskit_summary = self.summary();
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_label(self.label);
            node.set_value(accesskit_summary.as_str());
            node.clear_toggled();
        });
        if let Some(error) = crate::select_group::inherited_select_error(ui) {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                let description = match node.description() {
                    Some(existing) if !existing.is_empty() => format!("{existing}. {error}"),
                    _ => error,
                };
                node.set_description(description);
                node.set_invalid(egui::accesskit::Invalid::True);
            });
        }
        response
    }
}
