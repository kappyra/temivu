//! Preliminary port of Forui's one-time password field: a caller-owned code
//! string rendered as fixed-width boxes with digit filtering, caret
//! auto-advance, backspace, arrow-key and paste handling.
//!
//! A single focusable input group backs all boxes, like upstream's single text
//! field. Cursor blinking, dividers, per-platform keyboard appearance, form
//! integration and upstream visual parity remain open. See `docs/OTP_FIELD.md`.
//!
//! ```
//! # fn example(ui: &mut egui::Ui) {
//! use temivu::{OtpField, Theme};
//!
//! let theme = Theme::light();
//! let mut code = String::new();
//! let output = OtpField::new(&mut code, &theme, "One-time password")
//!     .length(6)
//!     .show(ui);
//! if output.completed {
//!     let _submitted = code;
//! }
//! # }
//! ```

use crate::Theme;
use crate::tokens::Sizes;
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

/// Horizontal gap between boxes, in logical pixels.
const BOX_SPACING: f32 = 6.0;
/// Corner radius shared by all boxes.
const BOX_RADIUS: u8 = 6;

/// English defaults for the OTP field's accessible labels.
#[derive(Clone, Copy)]
pub struct OtpLabels<'a> {
    /// Builds a box's accessible label from its zero-based index and the total
    /// number of boxes, for example `"Digit 1 of 6"`.
    pub digit: &'a dyn Fn(usize, usize) -> String,
}

impl<'a> Default for OtpLabels<'a> {
    fn default() -> Self {
        OtpLabels::english()
    }
}

impl OtpLabels<'static> {
    /// The English label set used when the caller supplies none.
    pub fn english() -> Self {
        Self {
            digit: &english_digit_label,
        }
    }
}

fn english_digit_label(index: usize, length: usize) -> String {
    format!("Digit {} of {length}", index + 1)
}

/// What [`OtpField::show`] reports for the current frame.
pub struct OtpFieldResponse {
    /// The focusable response that owns the input group.
    pub response: Response,
    /// Whether the caller-owned string changed this frame.
    pub changed: bool,
    /// Whether the code is now full length.
    pub completed: bool,
}

/// Caller-owned view state that must survive between frames.
#[derive(Clone, Copy, Debug, Default)]
struct ViewState {
    /// Index of the box that receives the next digit. May equal the code
    /// length, which is the first empty box after a partially filled code.
    focused_index: usize,
}

/// A one-time password input bound to a caller-owned [`String`].
pub struct OtpField<'a> {
    value: &'a mut String,
    theme: &'a Theme,
    label: &'a str,
    description: Option<&'a str>,
    error: Option<&'a str>,
    length: usize,
    sizes: Sizes,
    enabled: bool,
    invalid: bool,
    labels: OtpLabels<'a>,
    id: Option<Id>,
}

impl<'a> OtpField<'a> {
    /// Create a six-box OTP field for `value` using `theme`'s colors.
    pub fn new(value: &'a mut String, theme: &'a Theme, label: &'a str) -> Self {
        Self {
            value,
            theme,
            label,
            description: None,
            error: None,
            length: 6,
            sizes: Sizes::desktop(),
            enabled: true,
            invalid: false,
            labels: OtpLabels::english(),
            id: None,
        }
    }

    /// The largest supported number of boxes.
    ///
    /// The bound keeps per-frame box layout and allocation proportional to a
    /// sane code length; a caller that needs more can render multiple fields.
    pub const MAX_LENGTH: usize = 32;

    /// Set the number of boxes, clamped to `1..=MAX_LENGTH`.
    pub fn length(mut self, length: usize) -> Self {
        self.length = length.clamp(1, Self::MAX_LENGTH);
        self
    }

    /// Set supporting text shown below the boxes.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Show a validation error below the boxes and mark the group invalid.
    pub fn error(mut self, error: &'a str) -> Self {
        self.error = Some(error);
        self.invalid = true;
        self
    }

    /// Explicitly mark the field invalid without an error message.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disable input and pointer interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the responsive size tokens. The box side is `sizes.field.md`.
    pub fn sizes(mut self, sizes: Sizes) -> Self {
        self.sizes = sizes;
        self
    }

    /// Replace the English per-box label defaults.
    pub fn labels(mut self, labels: OtpLabels<'a>) -> Self {
        self.labels = labels;
        self
    }

    /// Give the field a stable, globally unique ID across layout changes.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Draw the field and return its response.
    pub fn show(self, ui: &mut Ui) -> OtpFieldResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("temivu-otp"));
        ui.scope_builder(egui::UiBuilder::new().scope_id(id), |ui| {
            self.show_inner(ui, id)
        })
        .inner
    }

    fn show_inner(self, ui: &mut Ui, id: Id) -> OtpFieldResponse {
        let Self {
            value,
            theme,
            label,
            description,
            error,
            length,
            sizes,
            enabled,
            invalid,
            labels,
            ..
        } = self;
        let length = length.max(1);
        let mut changed = normalize_value(value, length);

        let key = id.with("view-state");
        let mut view = ui
            .data(|data| data.get_temp::<ViewState>(key))
            .unwrap_or_default();
        clamp_focus(&mut view.focused_index, value, length);

        if !label.is_empty() {
            ui.label(egui::RichText::new(label).color(theme.foreground));
            ui.add_space(6.0);
        }

        let spacing = BOX_SPACING;
        let available = ui.available_width();
        let box_size = sizes.field.md.min(
            ((available - spacing * (length.saturating_sub(1)) as f32) / length as f32).max(12.0),
        );
        let box_size = box_size.max(12.0);
        let row_width = box_size * length as f32 + spacing * (length.saturating_sub(1)) as f32;

        let output = ui.add_enabled_ui(enabled, |ui| {
            ui.allocate_exact_size(Vec2::new(row_width, box_size), Sense::click())
        });
        let (rect, mut response) = output.inner;
        let box_step = box_size + spacing;

        // Pointer handling: a click focuses the group and the pressed box.
        if enabled && response.clicked() {
            response.request_focus();
            if let Some(pointer) = response.interact_pointer_pos() {
                let index = ((pointer.x - rect.min.x) / box_step).floor();
                let index = if index.is_finite() {
                    (index as isize).clamp(0, length as isize - 1) as usize
                } else {
                    0
                };
                view.focused_index = index.min(value.chars().count());
            }
        }

        // Keyboard and text input. The group keeps focus while the user types.
        if enabled && response.has_focus() {
            let events = ui.input(|input| input.events.clone());
            for event in &events {
                match event {
                    egui::Event::Text(text) | egui::Event::Paste(text) => {
                        changed |= insert_digits(value, &mut view.focused_index, length, text);
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => match key {
                        egui::Key::Backspace => {
                            changed |= backspace(value, &mut view.focused_index);
                        }
                        egui::Key::Delete => {
                            changed |= delete_forward(value, &mut view.focused_index);
                        }
                        egui::Key::ArrowLeft => {
                            view.focused_index = view.focused_index.saturating_sub(1);
                        }
                        egui::Key::ArrowRight => {
                            view.focused_index = (view.focused_index + 1)
                                .min(value.chars().count())
                                .min(length.saturating_sub(1));
                        }
                        egui::Key::Home if modifiers.is_none() => {
                            view.focused_index = 0;
                        }
                        egui::Key::End if modifiers.is_none() => {
                            view.focused_index = value.chars().count().min(length - 1);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        clamp_focus(&mut view.focused_index, value, length);

        // Paint the boxes and register one accessible node per box.
        let focused = response.has_focus();
        let digit_label = labels.digit;
        for index in 0..length {
            let box_rect = Rect::from_min_size(
                egui::pos2(rect.min.x + index as f32 * box_step, rect.min.y),
                Vec2::splat(box_size),
            );
            let box_id = id.with(("box", index));
            ui.interact(box_rect, box_id, Sense::hover());
            let character = value.chars().nth(index);
            let box_label = digit_label(index, length);
            ui.ctx().accesskit_node_builder(box_id, |node| {
                // AccessKit stores a Role::Label's text in `value`, which makes
                // the per-box position label awkward to query and read. Each
                // box is exposed as a single-character text input instead, so
                // its accessible name is the position and its value the digit.
                node.set_role(egui::accesskit::Role::TextInput);
                node.set_label(box_label.as_str());
                node.set_bounds(accesskit_rect(box_rect));
                match character {
                    Some(character) => node.set_value(character.to_string()),
                    None => node.clear_value(),
                }
                if !enabled {
                    node.set_disabled();
                }
                if invalid {
                    node.set_invalid(egui::accesskit::Invalid::True);
                }
            });
            let focus_ring = focused && view.focused_index == index;
            paint_box(
                ui.painter(),
                box_rect,
                theme,
                BoxVisual {
                    invalid,
                    disabled: !enabled,
                    focus_ring,
                    character,
                },
                box_size,
            );
        }

        if enabled && response.hovered() {
            ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::Text);
        }

        // Group node: one accessible name, current code and states for all boxes.
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(egui::accesskit::Role::TextInput);
            node.set_label(label);
            node.set_value(value.clone());
            if invalid || error.is_some() {
                node.set_invalid(egui::accesskit::Invalid::True);
            }
            if !enabled {
                node.set_disabled();
            }
        });

        if error.is_some() || description.is_some() {
            ui.add_space(6.0);
        }
        if let Some(description) = description {
            ui.label(egui::RichText::new(description).color(theme.muted_foreground));
        }
        if let Some(error) = error {
            ui.label(egui::RichText::new(error).color(theme.error));
        }

        ui.data_mut(|data| data.insert_temp(key, view));

        let completed = value.chars().count() >= length;
        if changed {
            response.mark_changed();
        }
        OtpFieldResponse {
            response,
            changed,
            completed,
        }
    }
}

/// Keep only ASCII digits and at most `length` characters.
fn normalize_value(value: &mut String, length: usize) -> bool {
    if value.chars().all(|character| character.is_ascii_digit()) && value.chars().count() <= length
    {
        return false;
    }
    let cleaned: String = value
        .chars()
        .filter(char::is_ascii_digit)
        .take(length)
        .collect();
    if cleaned == *value {
        false
    } else {
        *value = cleaned;
        true
    }
}

fn clamp_focus(focused: &mut usize, value: &str, length: usize) {
    *focused = (*focused)
        .min(length.saturating_sub(1))
        .min(value.chars().count());
}

/// Insert the filtered digits of `text` at the focused box.
///
/// A single typed digit replaces the focused box's digit, matching typing over
/// an existing item. Longer text, such as a paste or an operating-system
/// autofill, is inserted at the focused box and the resulting code is truncated
/// to `length` from the start, matching upstream's truncation behaviour.
fn insert_digits(value: &mut String, focused: &mut usize, length: usize, text: &str) -> bool {
    let digits: Vec<char> = text.chars().filter(char::is_ascii_digit).collect();
    match digits.as_slice() {
        [] => false,
        [digit] => {
            insert_digit(value, focused, length, *digit);
            true
        }
        _ => {
            let mut characters: Vec<char> = value.chars().collect();
            let at = (*focused).min(characters.len());
            characters.splice(at..at, digits.iter().copied());
            characters.truncate(length);
            *value = characters.into_iter().collect();
            *focused = (at + digits.len())
                .min(length.saturating_sub(1))
                .min(value.chars().count());
            true
        }
    }
}

/// Insert or replace the digit at the focused box and auto-advance.
fn insert_digit(value: &mut String, focused: &mut usize, length: usize, digit: char) {
    let mut characters: Vec<char> = value.chars().collect();
    if *focused < characters.len() {
        characters[*focused] = digit;
    } else {
        characters.push(digit);
    }
    characters.truncate(length);
    *value = characters.into_iter().collect();
    *focused = (*focused + 1)
        .min(length.saturating_sub(1))
        .min(value.chars().count());
}

/// Delete the digit before the focused box; returns whether the value changed.
fn backspace(value: &mut String, focused: &mut usize) -> bool {
    let mut characters: Vec<char> = value.chars().collect();
    if characters.is_empty() {
        *focused = 0;
        return false;
    }
    let delete_index = (*focused).min(characters.len() - 1);
    characters.remove(delete_index);
    let new_length = characters.len();
    *value = characters.into_iter().collect();
    *focused = if delete_index == new_length {
        new_length
    } else {
        delete_index.saturating_sub(1)
    };
    true
}

/// Delete the digit at the focused box; returns whether the value changed.
fn delete_forward(value: &mut String, focused: &mut usize) -> bool {
    let mut characters: Vec<char> = value.chars().collect();
    if *focused >= characters.len() {
        return false;
    }
    characters.remove(*focused);
    *value = characters.into_iter().collect();
    *focused = (*focused).min(value.chars().count());
    true
}

struct BoxVisual {
    invalid: bool,
    disabled: bool,
    focus_ring: bool,
    character: Option<char>,
}

fn paint_box(painter: &egui::Painter, rect: Rect, theme: &Theme, visual: BoxVisual, box_size: f32) {
    let radius = egui::CornerRadius::same(BOX_RADIUS);
    painter.rect_filled(rect, radius, theme.card);
    let (stroke, width) = if visual.invalid {
        (theme.error, 2.0)
    } else if visual.focus_ring {
        (theme.foreground, 2.0)
    } else {
        (theme.border, 1.0)
    };
    painter.rect_stroke(rect, radius, Stroke::new(width, stroke), StrokeKind::Inside);
    let text_color = if visual.disabled {
        theme.muted_foreground
    } else {
        theme.foreground
    };
    match visual.character {
        Some(character) => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                character,
                egui::FontId::proportional((box_size * 0.45).clamp(10.0, 22.0)),
                text_color,
            );
        }
        None if visual.focus_ring => {
            let height = box_size * 0.5;
            let x = rect.center().x;
            painter.line_segment(
                [
                    egui::pos2(x, rect.center().y - height / 2.0),
                    egui::pos2(x, rect.center().y + height / 2.0),
                ],
                Stroke::new(2.0, theme.primary),
            );
        }
        None => {}
    }
}

fn accesskit_rect(rect: Rect) -> egui::accesskit::Rect {
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
    fn normalization_filters_digits_and_truncates() {
        let mut value = "1a2b34".to_owned();
        assert!(normalize_value(&mut value, 4));
        assert_eq!(value, "1234");
        assert!(!normalize_value(&mut value, 4));
        let mut clean = "12".to_owned();
        assert!(!normalize_value(&mut clean, 6));
        assert_eq!(clean, "12");
    }

    #[test]
    fn insert_digit_replaces_then_advances() {
        let mut value = String::new();
        let mut focused = 0;
        insert_digit(&mut value, &mut focused, 6, '1');
        assert_eq!((value.as_str(), focused), ("1", 1));
        insert_digit(&mut value, &mut focused, 6, '2');
        assert_eq!((value.as_str(), focused), ("12", 2));
        // Replacing at a focused middle box keeps the length and advances.
        focused = 1;
        insert_digit(&mut value, &mut focused, 6, '9');
        assert_eq!((value.as_str(), focused), ("19", 2));
        // Typing when full replaces the final digit and keeps focus at the end.
        value = "123456".to_owned();
        focused = 5;
        insert_digit(&mut value, &mut focused, 6, '0');
        assert_eq!((value.as_str(), focused), ("123450", 5));
    }

    #[test]
    fn multi_digit_text_inserts_and_truncates_from_the_start() {
        let mut value = String::new();
        let mut focused = 0;
        assert!(insert_digits(&mut value, &mut focused, 6, "1234567890"));
        assert_eq!((value.as_str(), focused), ("123456", 5));

        let mut value = "12".to_owned();
        let mut focused = 2;
        assert!(insert_digits(&mut value, &mut focused, 6, "34"));
        assert_eq!((value.as_str(), focused), ("1234", 4));

        let mut value = "123456".to_owned();
        let mut focused = 2;
        assert!(insert_digits(&mut value, &mut focused, 6, "99"));
        assert_eq!((value.as_str(), focused), ("129934", 4));

        let mut value = "123".to_owned();
        let mut focused = 1;
        assert!(!insert_digits(&mut value, &mut focused, 6, "abc"));
        assert_eq!((value.as_str(), focused), ("123", 1));

        // A single typed digit replaces in place instead of shifting the code.
        let mut value = "123".to_owned();
        let mut focused = 1;
        assert!(insert_digits(&mut value, &mut focused, 6, "9"));
        assert_eq!((value.as_str(), focused), ("193", 2));
    }

    #[test]
    fn backspace_deletes_before_the_caret_and_moves_back() {
        let mut value = "123".to_owned();
        let mut focused = 3;
        assert!(backspace(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("12", 2));
        assert!(backspace(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("1", 1));
        assert!(backspace(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("", 0));
        assert!(!backspace(&mut value, &mut focused));
    }

    #[test]
    fn backspace_in_the_middle_moves_focus_back() {
        let mut value = "123456".to_owned();
        let mut focused = 2;
        assert!(backspace(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("12456", 1));
        // Deleting the first digit keeps focus on the first box.
        let mut value = "23456".to_owned();
        let mut focused = 0;
        assert!(backspace(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("3456", 0));
    }

    #[test]
    fn delete_forward_removes_the_focused_digit() {
        let mut value = "123".to_owned();
        let mut focused = 1;
        assert!(delete_forward(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("13", 1));
        assert!(delete_forward(&mut value, &mut focused));
        assert_eq!((value.as_str(), focused), ("1", 1));
        assert!(!delete_forward(&mut value, &mut focused));
    }

    #[test]
    fn focus_is_clamped_to_the_code() {
        let mut focused = 9;
        clamp_focus(&mut focused, "12", 6);
        assert_eq!(focused, 2);
        let mut focused = 3;
        clamp_focus(&mut focused, "123456", 6);
        assert_eq!(focused, 3);
    }

    #[test]
    fn labels_and_sizes_have_stable_defaults() {
        assert_eq!(english_digit_label(0, 6), "Digit 1 of 6");
        assert_eq!(english_digit_label(5, 6), "Digit 6 of 6");
        assert_eq!(Sizes::desktop().field.md, 36.0);
    }
}
