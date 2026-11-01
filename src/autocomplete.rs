//! Preliminary port of Forui's autocomplete: a caller-owned text input plus a
//! filtered suggestion list.
//!
//! The default filter is a case-insensitive substring match over the candidate
//! labels. Filtering walks the caller-owned candidate slice without allocating
//! a filtered list each frame; keyboard navigation keeps a highlighted
//! suggestion in egui's temporary data and Enter/Tab/pointer accept it.
//!
//! Asynchronous filtering, sections/groups, rich items, inline typeahead
//! completion, an `onSubmit` callback, multi-line fields and upstream visual
//! parity remain open. See `docs/AUTOCOMPLETE.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui, query: &mut String) {
//! use temivu::{Autocomplete, AutocompleteOption, Messages, Theme};
//!
//! let theme = Theme::light();
//! let fruits = [
//!     AutocompleteOption::new(egui::Id::new("apple"), "Apple"),
//!     AutocompleteOption::new(egui::Id::new("banana"), "Banana"),
//! ];
//! let output = Autocomplete::new(query, &theme, "Fruit", &fruits)
//!     .hint("Start typing")
//!     .messages(Messages::for_locale("en"))
//!     .show(ui);
//! if let Some(accepted) = output.accepted {
//!     let _chosen = accepted;
//! }
//! # }
//! ```

use crate::Theme;
use crate::localization::Messages;
use crate::tokens::Sizes;
use egui::{Align2, EventFilter, FontId, Id, Key, Modifiers, Response, Sense, Ui, Vec2};

/// A deterministic suggestion filter receiving the query and a candidate label.
///
/// Replace the default case-insensitive substring filter through
/// [`Autocomplete::filter`]. The callback runs at least twice per frame and
/// twice more per keyboard step, so it must be cheap and side-effect free.
pub type AutocompleteFilter<'a> = dyn Fn(&str, &str) -> bool + 'a;

/// One suggestion in an [`Autocomplete`].
///
/// Give every suggestion an ID that is stable across frames and unique within
/// the autocomplete so keyboard highlighting and acceptance stay attached to
/// the same value.
#[derive(Clone, Copy, Debug)]
pub struct AutocompleteOption<'a> {
    pub id: Id,
    pub label: &'a str,
    pub enabled: bool,
}

impl<'a> AutocompleteOption<'a> {
    /// Create an enabled suggestion from a stable ID and its visible label.
    pub fn new(id: Id, label: &'a str) -> Self {
        Self {
            id,
            label,
            enabled: true,
        }
    }

    /// Disable the suggestion: it stays listed and announced, but keyboard
    /// navigation skips it and pressing it has no effect.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// What [`Autocomplete::show`] reports for the current frame.
pub struct AutocompleteResponse {
    /// The text field's response.
    pub response: Response,
    /// Whether the query changed or a suggestion was accepted this frame.
    pub changed: bool,
    /// The suggestion accepted this frame by Enter, Tab or a pointer press.
    pub accepted: Option<Id>,
    /// Whether the suggestion list is open after this frame.
    pub open: bool,
    /// The suggestion highlighted for keyboard acceptance, if any.
    pub highlighted: Option<Id>,
}

/// Why the suggestion list is currently dismissed while the field keeps focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DismissReason {
    /// A suggestion was accepted; do not reopen when focus returns.
    Accepted,
    /// Escape closed the list.
    Escape,
    /// A pointer press outside the popup closed the list.
    Outside,
}

#[derive(Clone, Copy, Debug, Default)]
struct AutocompleteMemory {
    dismissed: Option<DismissReason>,
    highlight: Option<Id>,
    query_hash: Option<u64>,
    restore_focus: bool,
}

/// A text input with caller-owned query state and a filtered suggestion list.
///
/// The suggestion list opens while the field has keyboard focus and closes on
/// Escape, on acceptance or on a pointer press outside the popup. Pass a stable
/// [`Id`] through [`Autocomplete::id`] when several autocompletes can share a
/// window or when preceding conditional widgets change the call order.
pub struct Autocomplete<'a> {
    query: &'a mut String,
    theme: &'a Theme,
    label: &'a str,
    hint: &'a str,
    candidates: &'a [AutocompleteOption<'a>],
    filter: Option<&'a AutocompleteFilter<'a>>,
    messages: Messages,
    id: Option<Id>,
    sizes: Sizes,
    enabled: bool,
    max_popup_height: f32,
}

impl<'a> Autocomplete<'a> {
    /// Create an autocomplete for the caller-owned `query` and `candidates`.
    pub fn new(
        query: &'a mut String,
        theme: &'a Theme,
        label: &'a str,
        candidates: &'a [AutocompleteOption<'a>],
    ) -> Self {
        Self {
            query,
            theme,
            label,
            hint: "",
            candidates,
            filter: None,
            messages: Messages::for_locale("en"),
            id: None,
            sizes: Sizes::desktop(),
            enabled: true,
            max_popup_height: 240.0,
        }
    }

    /// Set the hint text shown while the query is empty.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    /// Replace the default case-insensitive substring filter.
    ///
    /// The callback receives the current query and each candidate label in the
    /// caller's order and must be deterministic: it runs at least twice per
    /// frame and twice more per keyboard step.
    pub fn filter(mut self, filter: &'a AutocompleteFilter<'a>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Replace the English localization messages.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
        self
    }

    /// Give the autocomplete a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// The responsive size tokens used for the row height. Defaults to desktop.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// Disable the field and its suggestion list.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The maximum height of the scrollable suggestion list in points.
    /// Defaults to 240.
    pub fn max_popup_height(mut self, height: f32) -> Self {
        self.max_popup_height = height.max(1.0);
        self
    }

    /// Draw the field and its suggestion list.
    pub fn show(self, ui: &mut Ui) -> AutocompleteResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-autocomplete"));
        ui.scope_builder(egui::UiBuilder::new().id(id), |ui| self.show_inner(ui, id))
            .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> AutocompleteResponse {
        let Self {
            query,
            theme,
            label,
            hint,
            candidates,
            filter,
            messages,
            sizes,
            enabled,
            max_popup_height,
            ..
        } = self;
        let field_id = id.with("field");
        let popup_id = id.with("popup");
        let memory_key = id.with("memory");
        let mut memory = ui
            .ctx()
            .data(|data| data.get_temp::<AutocompleteMemory>(memory_key))
            .unwrap_or_default();

        let query_hash = hash_query(query);
        if memory.query_hash != Some(query_hash) {
            memory.query_hash = Some(query_hash);
            memory.dismissed = None;
            memory.highlight = None;
        }

        let mut accepted = None;
        let mut changed = false;
        let mut dismissed = memory.dismissed;
        let mut restore_focus = memory.restore_focus;
        let mut highlighted = memory.highlight.filter(|highlighted| {
            candidates.iter().any(|option| {
                option.id == *highlighted && option.enabled && is_match(query, option.label, filter)
            })
        });
        let field_had_focus = ui.memory(|memory| memory.has_focus(field_id));
        let previous_open = enabled && field_had_focus && dismissed.is_none();

        // Keyboard handling runs before the text field is drawn so the field
        // never sees (and consumes) the keys that drive the suggestion list.
        if previous_open {
            let (up, down, enter, escape, tab) = ui.input_mut(|input| {
                (
                    input.consume_key(Modifiers::NONE, Key::ArrowUp),
                    input.consume_key(Modifiers::NONE, Key::ArrowDown),
                    input.consume_key(Modifiers::NONE, Key::Enter),
                    input.consume_key(Modifiers::NONE, Key::Escape),
                    input.consume_key(Modifiers::NONE, Key::Tab),
                )
            });
            if up || down {
                highlighted = step_highlight(
                    candidates,
                    query,
                    filter,
                    highlighted,
                    if down { 1 } else { -1 },
                );
            }
            if enter || tab {
                let target =
                    highlighted.or_else(|| step_highlight(candidates, query, filter, None, 1));
                if let Some(target) = target
                    && let Some(option) = candidates.iter().find(|option| option.id == target)
                {
                    *query = option.label.to_owned();
                    accepted = Some(target);
                    changed = true;
                    highlighted = None;
                    dismissed = Some(DismissReason::Accepted);
                }
            }
            if escape {
                dismissed = Some(DismissReason::Escape);
                highlighted = None;
            }
        }

        let label_response = ui.label(label);
        let event_filter = EventFilter {
            tab: previous_open,
            escape: previous_open,
            horizontal_arrows: true,
            vertical_arrows: true,
        };
        let edit = egui::TextEdit::singleline(query)
            .hint_text(hint)
            .id(field_id)
            .event_filter(event_filter)
            .desired_width(ui.available_width());
        let field_response = ui.add_enabled(enabled, edit).labelled_by(label_response.id);
        let field_focused = field_response.has_focus();
        let restoring = restore_focus;
        restore_focus = false;
        if field_response.gained_focus() && !restoring {
            dismissed = None;
        }
        if field_response.clicked() {
            dismissed = None;
        }
        if field_response.changed() {
            dismissed = None;
            highlighted = None;
        }

        // Keep the list drawn for one frame after the field loses focus so a
        // pointer press on a suggestion can still be delivered to it.
        let mut open = enabled && dismissed.is_none() && (field_focused || previous_open);
        let row_height = sizes.item.max(8.0);
        let width = field_response.rect.width().max(1.0);
        let mut popup_open = open;
        let mut clicked_option = None;
        if open {
            let popup = egui::Popup::from_response(&field_response)
                .id(popup_id)
                .open_bool(&mut popup_open)
                // Outside clicks are handled by the field-focus check below;
                // egui's own click-close would race the first opening pass.
                .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                .width(width);
            let field_for_popup = field_response.clone();
            let popup_response = popup.show(|ui| {
                ui.set_width(width);
                let mut matched = 0usize;
                let row_width = ui.available_width().max(1.0);
                egui::ScrollArea::vertical()
                    .max_height(max_popup_height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        for option in candidates {
                            if !is_match(query, option.label, filter) {
                                continue;
                            }
                            matched += 1;
                            let (row_rect, _) = ui.allocate_exact_size(
                                Vec2::new(row_width, row_height),
                                Sense::hover(),
                            );
                            let row = ui.interact(
                                row_rect,
                                popup_id.with(("option", option.id)),
                                if option.enabled {
                                    Sense::click()
                                } else {
                                    Sense::hover()
                                },
                            );
                            let is_highlighted = highlighted == Some(option.id);
                            if is_highlighted || (option.enabled && row.hovered()) {
                                ui.painter().rect_filled(
                                    row_rect.shrink2(Vec2::new(2.0, 0.0)),
                                    3.0,
                                    theme.muted,
                                );
                            }
                            let color = if option.enabled {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            };
                            ui.painter().text(
                                row_rect.center(),
                                Align2::CENTER_CENTER,
                                option.label,
                                FontId::proportional(theme.typography.body.size.max(8.0)),
                                color,
                            );
                            ui.ctx().accesskit_node_builder(row.id, |node| {
                                node.set_role(egui::accesskit::Role::ListBoxOption);
                                node.set_label(option.label);
                                node.set_selected(is_highlighted);
                                node.set_bounds(accesskit_rect(row_rect));
                                if option.enabled {
                                    node.clear_disabled();
                                } else {
                                    node.set_disabled();
                                }
                            });
                            if option.enabled && row.clicked() {
                                clicked_option = Some((option.id, option.label));
                                field_for_popup.request_focus();
                                restore_focus = true;
                            }
                        }
                        if matched == 0 {
                            let empty = ui.label(
                                egui::RichText::new(messages.autocomplete_no_results)
                                    .color(theme.muted_foreground),
                            );
                            ui.ctx().accesskit_node_builder(empty.id, |node| {
                                node.set_role(egui::accesskit::Role::Alert);
                                node.set_live(egui::accesskit::Live::Polite);
                                node.set_label(messages.autocomplete_no_results);
                                node.set_bounds(accesskit_rect(empty.rect));
                            });
                        }
                    });
            });
            if !popup_open {
                dismissed = Some(DismissReason::Outside);
                open = false;
            }
            if !field_focused && clicked_option.is_none() && accepted.is_none() {
                dismissed = Some(DismissReason::Outside);
                open = false;
            }
            if let Some(shown) = &popup_response {
                ui.ctx().accesskit_node_builder(shown.response.id, |node| {
                    node.set_role(egui::accesskit::Role::ListBox);
                    node.set_label(label);
                    node.set_bounds(accesskit_rect(shown.response.rect));
                    if let Some(highlighted) = highlighted {
                        node.set_active_descendant(
                            popup_id.with(("option", highlighted)).accesskit_id(),
                        );
                    } else {
                        node.clear_active_descendant();
                    }
                });
            }
        }
        if let Some((option_id, option_label)) = clicked_option {
            *query = option_label.to_owned();
            accepted = Some(option_id);
            changed = true;
            dismissed = Some(DismissReason::Accepted);
            highlighted = None;
            open = false;
        }

        ui.ctx().accesskit_node_builder(field_id, |node| {
            node.set_role(egui::accesskit::Role::ComboBox);
            node.set_label(label);
            node.set_value(query.as_str());
            if open {
                node.set_expanded(true);
                node.set_controls([popup_id.accesskit_id()]);
            } else {
                node.set_expanded(false);
                node.clear_controls();
            }
            if let Some(highlighted) = highlighted {
                node.set_active_descendant(popup_id.with(("option", highlighted)).accesskit_id());
            } else {
                node.clear_active_descendant();
            }
            if enabled {
                node.clear_disabled();
            } else {
                node.set_disabled();
            }
        });

        let final_hash = hash_query(query);
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                memory_key,
                AutocompleteMemory {
                    dismissed,
                    highlight: highlighted,
                    query_hash: Some(final_hash),
                    restore_focus,
                },
            );
        });

        let mut response = field_response;
        if changed {
            response.mark_changed();
        }
        AutocompleteResponse {
            response,
            changed,
            accepted,
            open,
            highlighted,
        }
    }
}

fn is_match(query: &str, label: &str, filter: Option<&AutocompleteFilter<'_>>) -> bool {
    match filter {
        Some(filter) => filter(query, label),
        None => contains_ignore_case(label, query),
    }
}

/// The next enabled matching suggestion in `delta` direction, wrapping around.
///
/// Returns `current` unchanged when no suggestion matches.
fn step_highlight(
    candidates: &[AutocompleteOption<'_>],
    query: &str,
    filter: Option<&AutocompleteFilter<'_>>,
    current: Option<Id>,
    delta: isize,
) -> Option<Id> {
    let total = candidates
        .iter()
        .filter(|option| option.enabled && is_match(query, option.label, filter))
        .count();
    if total == 0 || delta == 0 {
        return current;
    }
    let current_position = current.and_then(|current| {
        candidates
            .iter()
            .filter(|option| option.enabled && is_match(query, option.label, filter))
            .position(|option| option.id == current)
    });
    let next = match current_position {
        Some(position) => (position as isize + delta).rem_euclid(total as isize) as usize,
        None if delta > 0 => 0,
        None => total - 1,
    };
    candidates
        .iter()
        .filter(|option| option.enabled && is_match(query, option.label, filter))
        .nth(next)
        .map(|option| option.id)
}

fn hash_query(query: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.hash(&mut hasher);
    hasher.finish()
}

/// Case-insensitive substring search using simple Unicode case mapping.
///
/// Allocates nothing: both sides are compared char-by-char through
/// [`char::to_lowercase`], which covers one-to-many mappings such as `İ`.
/// Locale-tailored folding (for example Turkish dotless `ı`) is not applied.
fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    let needle_len = needle.chars().flat_map(char::to_lowercase).count();
    if needle_len == 0 {
        return true;
    }
    for (index, _) in haystack.char_indices() {
        let haystack_chars = haystack[index..].chars().flat_map(char::to_lowercase);
        let needle_chars = needle.chars().flat_map(char::to_lowercase);
        let mut matched = 0;
        for (haystack_char, needle_char) in haystack_chars.zip(needle_chars) {
            if haystack_char == needle_char {
                matched += 1;
            } else {
                break;
            }
        }
        if matched == needle_len {
            return true;
        }
    }
    false
}

fn accesskit_rect(rect: egui::Rect) -> egui::accesskit::Rect {
    egui::accesskit::Rect {
        x0: f64::from(rect.min.x),
        y0: f64::from(rect.min.y),
        x1: f64::from(rect.max.x),
        y1: f64::from(rect.max.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_matching_is_case_insensitive_and_unicode_aware() {
        assert!(contains_ignore_case("Apple", "ap"));
        assert!(contains_ignore_case("Apple", "PLE"));
        assert!(contains_ignore_case("apple", ""));
        assert!(!contains_ignore_case("Apple", "banana"));
        assert!(contains_ignore_case("Café", "CAFÉ"));
        assert!(!contains_ignore_case("Straße", "STRASSE"));
        assert!(contains_ignore_case("İstanbul", "i̇stanbul"));
        assert!(!contains_ignore_case("apple", "apples"));
    }

    #[test]
    fn highlight_steps_skip_disabled_and_wrap() {
        let candidates = [
            AutocompleteOption::new(Id::new("a"), "Alpha"),
            AutocompleteOption::new(Id::new("b"), "Beta").enabled(false),
            AutocompleteOption::new(Id::new("c"), "Gamma"),
            AutocompleteOption::new(Id::new("d"), "Delta"),
        ];
        assert_eq!(
            step_highlight(&candidates, "a", None, None, 1),
            Some(Id::new("a"))
        );
        assert_eq!(
            step_highlight(&candidates, "a", None, Some(Id::new("a")), 1),
            Some(Id::new("c")),
            "disabled suggestions are skipped"
        );
        assert_eq!(
            step_highlight(&candidates, "a", None, Some(Id::new("c")), -1),
            Some(Id::new("a"))
        );
        assert_eq!(
            step_highlight(&candidates, "a", None, None, -1),
            Some(Id::new("d"))
        );
        assert_eq!(
            step_highlight(&candidates, "a", None, Some(Id::new("d")), 1),
            Some(Id::new("a")),
            "navigation wraps around"
        );
        assert_eq!(step_highlight(&candidates, "zzz", None, None, 1), None);
    }
}
