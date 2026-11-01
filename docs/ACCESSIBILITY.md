# Accessibility notes

This project targets keyboard operability, AccessKit semantics and screen-reader
compatibility, but it does not yet have a screen-reader test run. This page
records the accessibility review that has been performed and the known gaps. It
is not an accessibility certification.

## Colour contrast

Contrast was measured with `Theme::contrast_ratio` (WCAG 2.2 relative
luminance) and `Theme::destructive_rendered_contrast`, which composites the
translucent destructive fill exactly as the destructive `Button` and `Badge`
render it (26/51 alpha in the light theme, 51/77 in the dark theme, on the page
and card surfaces). The pinned Forui palettes put these pairs below the WCAG AA
threshold of 4.5:1 for normal text:

| Rendered pair | Contrast | Where |
| --- | ---: | --- |
| Destructive text on its tint, page background, light | 3.98:1 | `Button`/`Badge` destructive |
| Destructive text on its hovered tint, page background, light | 3.31:1 | hovered destructive `Button` |
| Destructive text on its hovered tint, dark, page background | 4.36:1 | hovered destructive `Button` |
| Destructive text on its tint, dark card surface | 4.66:1 (passes) | destructive `Badge` |
| Destructive text on its hovered tint, dark card surface | 3.83:1 | hovered destructive `Badge`/`Button` |
| Muted foreground on the muted surface, light | 4.35:1 | `Tabs` unselected label |

The tokens are byte-identical to the pinned Forui release
(`eea1e8e651d263f14ef43897ccbb7a2576549da5`), so changing the default palette
would be a deliberate deviation from upstream. The project keeps upstream parity
by default and records the shortfall instead of silently "improving" palette
tokens that other widgets also use. `theme.readable_on` is used at paint time
where a component can preserve its hue (for example Item and Tile active rows) to
raise specific text pairs to AA.

## High-contrast preset

`Theme::high_contrast()` returns a preset that keeps every unrelated token, hue
and alpha unchanged and adjusts only the tokens needed for the pairs above:

- the destructive colour is darkened in the light theme or lightened in the dark
  theme until the base and hovered rendered pairs reach 4.5:1 on both the page
  and card surfaces, and the colour still reads as text on the page background;
- the muted foreground is moved away from the muted surface until it reaches
  4.5:1 on the muted, page and card surfaces (the light theme moves it from
  4.35:1 to 4.68:1).

Measured results of the preset (rendered pairs): light destructive base 5.48:1,
hover 4.52:1; dark destructive base 6.67:1, hover 5.19:1, card 5.79:1/4.52:1.
The preset composes with `Theme::touch()`, is idempotent, leaves
`destructive_foreground` untouched (the destructive components derive their label
colour from `destructive`), and is covered by tests that assert the rendered pairs
for both themes, both hover states and both surfaces. The showcase has a
"High contrast" switch on its Overview page; applications can apply the preset
with `Theme::light().high_contrast()` (or the dark/touch equivalents).

The adjustment treats tokens as opaque and the rendered fill as a single
translucent layer; custom translucent tokens or stacked fills may need a
component-level policy instead. The dark theme's other pairs,
forced-colours/high-contrast OS modes and automatic contrast-preference detection
remain unaudited.

## Implemented

- Every interactive component has an AccessKit role, accessible name and, where
  applicable, selected/disabled/value/expanded state. Component test files
  query these through `egui_kittest` rather than assuming them.
- Keyboard paths exist for buttons, checkbox, switch, radio, slider, tabs,
  select, pagination, dialog, context menu, popover, popover menu, sheet,
  calendar grid, OTP boxes and the resizable divider. Escape and focus
  restoration behaviour is covered where a component owns focus.
- Focus is redirected only by explicit caller requests; popovers, context menus
  and sheets avoid stealing unrelated focus.
- Reduced motion can be requested through `Theme::motion`; zero egui
  `animation_time` disables the indeterminate animations. Applications are
  responsible for reading the operating-system preference.
- Localized semantic messages (labels for pagination, overlays, calendar, menu,
  select, fields and slider values) are available for 113 locales through
  `Messages`.
- `#![forbid(unsafe_code)]` and no blocking work inside `ui`.

## Not verified or not implemented

- No screen-reader run (Orca/AT-SPI, NVDA, VoiceOver), no real platform input
  testing and no automated accessibility audit tool.
- No bidi/shaping verification for Arabic and other right-to-left locales; the
  library renders strings through egui and does not shape text itself.
- No locale-aware date, time or number formatting, and no ICU plural/select
  messages.
- Touch-target audits, high-contrast/forced-colours modes, text scaling beyond
  egui's zoom factor, and platform reduced-motion detection are open.
- The light-theme contrast shortfalls above.
- Several rich-content and icon slots are not migrated, so their accessible
  structure is missing with them.
