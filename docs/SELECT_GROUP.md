# Select group (experimental)

A labelled grouping container for select fields. It draws a shared heading, an optional description and a validation message around caller-supplied fields, and propagates its disabled and validation state to them.

```rust
use temivu::{Select, SelectGroup, SelectOption, Theme};

fn example(ui: &mut egui::Ui, theme: &Theme, color: &mut Option<egui::Id>) {
let options = [SelectOption::new(egui::Id::unique("red"), "Red")];
SelectGroup::new(egui::Id::unique("shipping"), "Shipping color", theme)
    .description("Used on the label")
    .error("Choose a color")
    .show(ui, |ui| {
        Select::new(
            egui::Id::unique("shipping-color"),
            "Color",
            "Choose a color",
            &options,
            color,
        )
        .show(ui)
    });
}
```

The group node exposes the group role, its label and its description. Drawing happens inside egui's inherited enabled state, so a disabled group disables the fields inside it, including nested `Select` popups. When `error` is set the group is marked invalid, the message is shown below the content, added to the group's accessible description, and published to child fields: `Select` reads the inherited message through `inherited_select_error`, marks itself invalid and exposes the message as its description. Callers writing their own fields can read `inherited_select_error` too. The value is only valid during the group's content callback, and sibling groups do not inherit each other's errors.

Unsupported or unverified: form integration and validation callbacks, required markers, rich or localized labels, per-field error styling, error animation, RTL, screen-reader testing and upstream visual parity. See `COVERAGE.md` for the current limitations.
