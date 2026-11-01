//! Preliminary single select with optional search; tags/chips, groups, async
//! search and full parity remain open. See `docs/SELECT.md`.
use egui::{ComboBox, Id, Response, Ui};

/// A select option with an ID that remains stable and unique within the select.
#[derive(Clone, Copy, Debug)]
pub struct SelectOption<'a> {
    pub id: Id,
    pub label: &'a str,
    pub enabled: bool,
}
impl<'a> SelectOption<'a> {
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            enabled: true,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A caller-owned single-choice select. IDs must be stable and globally unique.
pub struct Select<'a> {
    id: Id,
    label: &'a str,
    placeholder: &'a str,
    options: &'a [SelectOption<'a>],
    selected: &'a mut Option<Id>,
    enabled: bool,
    searchable: bool,
    search_hint: &'a str,
    no_results: &'a str,
}
impl<'a> Select<'a> {
    pub fn new(
        id: Id,
        label: &'a str,
        placeholder: &'a str,
        options: &'a [SelectOption<'a>],
        selected: &'a mut Option<Id>,
    ) -> Self {
        Self {
            id,
            label,
            placeholder,
            options,
            selected,
            enabled: true,
            searchable: false,
            search_hint: "Search",
            no_results: "No results",
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Show a search field at the top of the popup and filter options by a
    /// case-insensitive substring match on their labels.
    ///
    /// The query is kept in egui's temporary data keyed by this select's ID and
    /// is cleared whenever the popup closes. Options are filtered in place; no
    /// async or fuzzy matching is performed.
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
        let current = self
            .options
            .iter()
            .find(|option| Some(option.id) == *self.selected)
            .map_or(self.placeholder, |option| option.label);
        let before = *self.selected;
        let can_change = self.enabled && ui.is_enabled();
        let popup_focus_key = self.id.with("temivu-select-popup-focus");
        let previous_popup_focus = ui
            .ctx()
            .data(|data| data.get_temp::<egui::Id>(popup_focus_key));
        let focused_before = ui.ctx().memory(|memory| memory.focused());
        let (escape, keyboard_activate) = ui.input(|input| {
            (
                input.key_pressed(egui::Key::Escape),
                input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::Space),
            )
        });
        let mut focused_popup: Option<egui::Id> = None;
        let mut accepted_from_keyboard = false;
        let mut was_open = false;
        let width = ui.available_width().clamp(48.0, 240.0);
        let search_key = self.id.with("temivu-select-search");
        let mut widget_id = None;
        let output = ui
            .add_enabled_ui(self.enabled, |ui| {
                let id = ui.make_persistent_id(egui::IdSalt::new(self.id));
                widget_id = Some(id);
                was_open = ComboBox::is_open(ui.ctx(), id);
                let min_height = ui.spacing().interact_size.y.max(32.0);
                ui.spacing_mut().interact_size.y = min_height;
                if !can_change && was_open {
                    // egui 0.36 derives this popup ID from the combo widget ID.
                    egui::Popup::close_id(ui.ctx(), id.with("popup"));
                }
                let just_opened = !was_open;
                let mut visible_buttons: Vec<(egui::Id, bool)> = Vec::new();
                ComboBox::from_id_salt(self.id)
                    .selected_text(current)
                    .width(width)
                    .truncate()
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
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
                        let mut shown = 0usize;
                        for option in self.options.iter().filter(|option| matches(option)) {
                            shown += 1;
                            let selected = *self.selected == Some(option.id);
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
                                                .min_size(egui::vec2(0.0, 32.0)),
                                        )
                                    },
                                )
                                .inner;
                            visible_buttons.push((response.id, can_change && option.enabled));
                            if focused_before == Some(response.id) {
                                focused_popup = Some(response.id);
                            }
                            ui.ctx().accesskit_node_builder(response.id, |node| {
                                node.set_label(option.label);
                                node.clear_toggled();
                                node.set_selected(selected);
                            });
                            if can_change && option.enabled && response.clicked() {
                                accepted_from_keyboard =
                                    focused_before == Some(response.id) && keyboard_activate;
                                *self.selected = Some(option.id);
                                ui.close();
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
        // focus, including the search field (which egui's Escape handling
        // otherwise leaves unfocused). Clicks elsewhere must not steal focus.
        let escape_from_popup = escape
            && was_open
            && (focused_before == Some(response.id)
                || focused_before.is_none()
                || previous_popup_focus.is_some_and(|previous| {
                    focused_before == Some(previous) || focused_popup == Some(previous)
                }));
        if can_change && (escape_from_popup || accepted_from_keyboard) {
            response.request_focus();
        }
        if *self.selected != before {
            response.mark_changed();
        }
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_label(self.label);
            node.set_value(current);
        });
        // Propagate the validation message of an enclosing `SelectGroup`, if
        // any, so the field itself is announced as invalid.
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
