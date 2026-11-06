//! Caller-owned queue for responsive notification cards.
//! Visible entries are limited, and closing cards release their space smoothly.
use crate::{Theme, Toast, ToastState, ToastVariant};
use egui::{Align2, Area, Id, Order, Response, Ui, Vec2};
use std::time::Duration;

/// A toast to queue in a [`ToasterState`].
#[derive(Clone, Debug)]
pub struct ToastEntry {
    id: Id,
    title: String,
    description: Option<String>,
    timeout: Option<Duration>,
    variant: ToastVariant,
    icon: Option<String>,
}

impl ToastEntry {
    /// Create an entry with the default timeout of five seconds.
    pub fn new(id: Id, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            timeout: Some(Duration::from_secs(5)),
            variant: ToastVariant::Primary,
            icon: None,
        }
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// `None` keeps the toast visible until it is dismissed.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }
    /// The upstream `FToastVariant` (primary or destructive).
    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }
    /// An optional decorative leading icon (text glyph approximation).
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn id(&self) -> Id {
        self.id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
}

struct QueuedToast {
    entry: ToastEntry,
    state: ToastState,
}

/// Caller-owned queue of toasts.
pub struct ToasterState {
    entries: Vec<QueuedToast>,
    limit: usize,
}

impl Default for ToasterState {
    fn default() -> Self {
        Self::new(3)
    }
}

impl ToasterState {
    /// A queue that renders at most `limit` toasts at once (clamped to 1..=8).
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            limit: limit.clamp(1, 8),
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Queue a toast. Re-pushing an existing ID replaces and restarts it.
    pub fn push(&mut self, entry: ToastEntry) {
        self.entries.retain(|queued| queued.entry.id != entry.id);
        self.entries.push(QueuedToast {
            state: ToastState::new(entry.timeout),
            entry,
        });
    }

    /// Dismiss a queued toast by ID; unknown IDs are ignored.
    pub fn dismiss(&mut self, id: Id) {
        if let Some(queued) = self.entries.iter_mut().find(|queued| queued.entry.id == id) {
            queued.state.dismiss();
        }
        self.prune();
    }

    pub fn clear(&mut self) {
        for queued in &mut self.entries {
            queued.state.dismiss();
        }
        self.prune();
    }

    pub fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|queued| queued.state.is_visible())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        !self.entries.iter().any(|queued| queued.state.is_visible())
    }

    /// Whether there are visible cards, queued cards, or exit animations to render.
    /// Use this instead of `is_empty` to skip rendering an idle overlay.
    pub fn needs_render(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn contains(&self, id: Id) -> bool {
        self.entries
            .iter()
            .any(|queued| queued.entry.id == id && queued.state.is_visible())
    }

    fn prune(&mut self) {
        self.entries.retain(|queued| queued.state.needs_render());
    }

    /// Render the visible queue as a foreground overlay with the upstream
    /// default anchor (bottom-end on desktop, top-centre on touch) and prune
    /// expired toasts.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme, dismiss_label: &str) -> ToasterResponse {
        let id = ui.make_persistent_id("temivu-toaster");
        let anchor = default_anchor(ui, theme);
        self.show_with(ui, theme, dismiss_label, id, anchor, default_offset(anchor))
    }

    /// Render the queue with an explicit overlay ID and anchor. Give each
    /// toaster in one parent its own ID.
    pub fn show_with(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        dismiss_label: &str,
        id: Id,
        anchor: Align2,
        offset: Vec2,
    ) -> ToasterResponse {
        let viewport = ui.ctx().content_rect();
        let width = (viewport.width() - 2.0 * offset.x.abs().max(12.0)).clamp(0.0, 392.0);
        let height =
            (viewport.height() - offset.y.abs() - 16.0).clamp(0.0, viewport.height() * 0.62);
        let area = Area::new(id)
            .order(Order::Foreground)
            .anchor(anchor, offset)
            .constrain_to(viewport)
            .fade_in(false)
            .show(ui.ctx(), |ui| {
                if self.entries.is_empty() {
                    return 0;
                }
                ui.set_width(width);
                egui::ScrollArea::vertical()
                    .max_height(height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;
                        let mut rendered = 0;
                        for queued in &mut self.entries {
                            let QueuedToast { entry, state } = queued;
                            if state.is_visible() && rendered >= self.limit {
                                continue;
                            }
                            let shown = ui
                                .push_id(entry.id, |ui| {
                                    let mut toast =
                                        Toast::new(state, theme, &entry.title, dismiss_label)
                                            .variant(entry.variant);
                                    if let Some(description) = &entry.description {
                                        toast = toast.description(description);
                                    }
                                    if let Some(icon) = &entry.icon {
                                        toast = toast.icon(icon);
                                    }
                                    let accessible = format!("{dismiss_label}: {}", entry.title);
                                    toast
                                        .dismiss_accessible_label(&accessible)
                                        .show(ui)
                                        .is_some()
                                })
                                .inner;
                            rendered += usize::from(shown && state.is_visible());
                        }
                        rendered
                    })
                    .inner
            });
        self.prune();
        ToasterResponse {
            response: area.response,
            visible: area.inner,
        }
    }
}

fn default_anchor(ui: &Ui, theme: &Theme) -> Align2 {
    if theme.touch || ui.ctx().content_rect().width() < 600.0 {
        Align2::CENTER_TOP
    } else {
        Align2::RIGHT_BOTTOM
    }
}

/// Keep the overlay 16 points away from whichever screen edge(s) the anchor
/// touches, and centred otherwise, so every [`Align2`] maps to a sane offset.
fn default_offset(anchor: Align2) -> Vec2 {
    let x = match anchor.x() {
        egui::Align::Min => 16.0,
        egui::Align::Center => 0.0,
        egui::Align::Max => -16.0,
    };
    let y = match anchor.y() {
        egui::Align::Min => 16.0,
        egui::Align::Center => 0.0,
        egui::Align::Max => -16.0,
    };
    egui::vec2(x, y)
}

/// The result of rendering a [`ToasterState`]. The area may be used to attach
/// hover or keyboard behaviour; `visible` is the number of toasts on screen.
pub struct ToasterResponse {
    pub response: Response,
    pub visible: usize,
}

/// Renders a [`ToasterState`] as a stacked foreground overlay.
pub struct Toaster<'a> {
    state: &'a mut ToasterState,
    theme: &'a Theme,
    dismiss_label: &'a str,
    id: Option<Id>,
    anchor: Option<Align2>,
}

impl<'a> Toaster<'a> {
    pub fn new(state: &'a mut ToasterState, theme: &'a Theme, dismiss_label: &'a str) -> Self {
        Self {
            state,
            theme,
            dismiss_label,
            id: None,
            anchor: None,
        }
    }

    /// Use a caller-supplied overlay ID (one per toaster under a parent).
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Override the default anchor (bottom-end on desktop, top-centre on touch).
    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn show(self, ui: &mut Ui) -> ToasterResponse {
        let id = self
            .id
            .unwrap_or_else(|| ui.make_persistent_id("temivu-toaster"));
        let anchor = self
            .anchor
            .unwrap_or_else(|| default_anchor(ui, self.theme));
        self.state.show_with(
            ui,
            self.theme,
            self.dismiss_label,
            id,
            anchor,
            default_offset(anchor),
        )
    }
}
