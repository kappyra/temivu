# Autocomplete (experimental)

A caller-owned text input with a filtered suggestion list, ported from Forui's
`FAutocomplete`. The query string is caller-owned; suggestions come from a
caller-owned candidate slice, so filtering is deterministic and asynchronous
search is out of scope.

```rust
use temivu::{Autocomplete, AutocompleteOption, Messages, Theme};

fn example(ui: &mut egui::Ui, query: &mut String) {
    let theme = Theme::light();
    let fruits = [
        AutocompleteOption::new(egui::Id::new("apple"), "Apple"),
        AutocompleteOption::new(egui::Id::new("banana"), "Banana"),
        AutocompleteOption::new(egui::Id::new("avocado"), "Avocado").enabled(false),
    ];
    let output = Autocomplete::new(query, &theme, "Fruit", &fruits)
        .hint("Start typing")
        .messages(Messages::for_locale("en"))
        .show(ui);
    if let Some(accepted) = output.accepted {
        let _chosen = accepted;
    }
}
```

## Filtering and state

The default filter is a case-insensitive substring match over the candidate
labels, using simple Unicode case mapping without allocating (locale-tailored
folding such as Turkish dotless `ı` is not applied). `Autocomplete::filter`
replaces it with a deterministic `Fn(&str, &str) -> bool` over `(query, label)`;
the callback runs at least twice per frame and twice more per keyboard step.

The list opens while the field has keyboard focus and stays open for one extra
frame after focus leaves so pointer presses on suggestions are still delivered.
It closes on Escape, on acceptance and on a pointer press elsewhere. Focus
returns to the field after an acceptance. The query is never truncated: users
can type values that are not suggestions, matching upstream's "text field with
suggestions, not a searchable select" contract.

## Keyboard and pointer

- Up/Down highlight the previous/next enabled suggestion and wrap around.
- Enter or Tab accept the highlighted suggestion, falling back to the first
  enabled match.
- Escape dismisses the list and keeps focus on the field.
- A pointer press on an enabled suggestion accepts it and returns focus to the
  field. Disabled suggestions stay listed but are skipped by navigation and
  ignore presses.

## Accessibility

The text field is exposed as a `ComboBox` with its visible label, current query
value and expanded state; while open it controls the suggestion `ListBox` and
names the highlighted row as its active descendant. Each suggestion is a
`ListBoxOption` with its label, selected (highlighted) state and disabled state.
The empty-result message uses the localized `Messages::autocomplete_no_results`
text as a polite `Alert` live region.

## Known gaps

Asynchronous/loading filters, error builders, suggestion sections and groups,
rich item contents, inline typeahead completion (Tab completes the whole label
here), right-arrow completion, form integration, validation, `onSubmit`,
multi-line and IME behaviour, per-widget styles, announcement de-duplication,
screen-reader testing on a real platform and upstream visual parity remain open.
See `COVERAGE.md`.
