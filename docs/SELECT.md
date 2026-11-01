# Select, multi-select and search

The preliminary select family covers Forui's single-choice `FSelect`, the
multi-choice mode and an optional search field. It is built on an egui
`ComboBox` popup with caller-owned state and explicit focus handling.

## Single select

```rust
use temivu::{Select, SelectOption};

fn example(ui: &mut egui::Ui, selected: &mut Option<egui::Id>) {
    let options = [
        SelectOption::new(egui::Id::new("red"), "Red"),
        SelectOption::new(egui::Id::new("blue"), "Blue"),
    ];
    Select::new(egui::Id::new("color"), "Color", "Choose a color", &options, selected)
        .searchable(true)
        .show(ui);
}
```

- Stable option IDs and a caller-owned `Option<Id>` selection.
- Disabled options are skipped by pointer and stale accessibility activation.
- Keyboard: Enter/Space opens from the field, options are focusable and
  Enter/Space accepts. Escape closes the popup and restores focus to the field
  (from an option row, the search field or the combo field itself); an outside
  click dismisses the popup without moving focus to the field.
- `.searchable(true)` adds a search field at the top of the popup. Matching is a
  case-insensitive substring match on the option label, the query lives in egui's
  temporary data keyed by the select's ID, and it is cleared when the popup
  closes. Arrow Down/Up from the search field moves focus to the first/last
  enabled match; `.search_hint` and `.no_results` override the English defaults.
- The popup closes on an outside click or Escape; selecting an option closes it.

## Multi-select

```rust
use temivu::{MultiSelect, SelectOption};

fn example(ui: &mut egui::Ui, selected: &mut Vec<egui::Id>) {
    let options = [
        SelectOption::new(egui::Id::new("email"), "Email"),
        SelectOption::new(egui::Id::new("sms"), "SMS"),
    ];
    MultiSelect::new(egui::Id::new("channels"), "Channels", "Choose channels", &options, selected)
        .show(ui);
}
```

- Caller-owned `Vec<Id>` selection; activating a row toggles its membership and
  the popup stays open, so several options can be chosen in one visit.
- The field summarises up to three selected labels (`"Email, SMS"`) and switches
  to `"N selected"` for longer selections. Tags/chips inside the field are not
  implemented yet.
- Rows expose the list-box-option role with their selected state; disabled rows
  cannot be toggled. Escape closes the popup and restores field focus (also from
  the search field); outside clicks dismiss it without taking focus.
- The same `.searchable`, `.search_hint` and `.no_results` options are available.

## Known gaps

- No tags/chips, groups, async/remote search, fuzzy matching, custom item
  builders, per-option icons, virtualised long lists, form integration/validation
  beyond `SelectGroup`'s inherited error, RTL behaviour or a list-box parent for
  the option rows. The `"N selected"` summary defaults to English and is
  localizable through `MultiSelect::count_summary`.
- Search and result lists allocate per frame (small option lists); performance
  for very large option sets is unverified.
- Screen-reader testing and upstream visual parity remain open; see
  `COVERAGE.md` and `docs/SELECT_GROUP.md` for grouping semantics.
