# Visual and behavioral parity pass (1 November 2026)

This is a historical record of the first public snapshot.
The paths and results below describe the
[`v0.0.0` source tree](https://github.com/kappyra/temivu/tree/v0.0.0).
For current screenshots, see the [showcase](../README.md#demo).

Scope: a second-wave visual and behavioural parity pass across the component families, compared
against the pinned Forui checkout. The reviewed render set is the 86 component-sheet baselines
in `harness/visual-review/baselines/`; they are compared against earlier versions of this
project's own renders, not against upstream golden images.

Reference: pinned Forui checkout at
`eea1e8e651d263f14ef43897ccbb7a2576549da5`. Numbers below come from
`forui/lib/src/theme/` (colors, border radius, sizes, typography),
`forui/lib/src/widgets/*_style.dart`, and the golden images under
`forui/test/golden/macos/`. The goldens use Forui's **touch** theme
(`TestScaffold.themes` selects `_default.light.touch` / `dark.touch`), 800x600
logical at DPR 3 with 16-point page padding; measured logical sizes are quoted
in the table as touch sizes where the golden drove the value.

## What changed

### 1. Theme tokens (`src/theme.rs`, `harness/port/generate_palette.py`)

Forui's `FColors` exposes `hoverLighten` (0.075), `hoverDarken` (0.05) and
`disabledOpacity` (0.5). These are now tokens and helpers on `Theme`:

| Token/helper | Value / behaviour | Upstream source |
| --- | --- | --- |
| `Theme.touch` | `false` for `light()`/`dark()`, `true` after `touch()` | `FThemeData.touch` |
| `Theme.disabled_opacity` | `0.5` | `FColors.disabledOpacity` |
| `Theme::disable(color)` | alpha × `disabled_opacity` | `FColors.disable` |
| `Theme::disable_blend(color, background)` | `disable` composited over background | `FColors.disable(color, background)` |
| `Theme::hover(color)` | HSL-lightness nudge with the Forui factors | `FColors.hover` |
| `Theme::contrast_ratio(a, b)` | WCAG 2.2 ratio (1–21) | accessibility helper |
| `Theme::readable_on(color, bg, ratio)` | minimal blend toward black/white until AA | accessibility helper |

The palette generator emits `touch: false` and `disabled_opacity: 0.5` for both
palettes; `generate_palette.py --check` passes.

### 2. Components

| Component | Upstream observation | Before | After | Remaining gap |
| --- | --- | --- | --- | --- |
| **Item** | `FItemStyle`: content decoration radius `borderRadius.md` = 10, background/secondary fills, 4-point outer margin, content padding 10/6.5 desktop (10/12.5 touch), title `body.sm`, subtitle `body.xs`, disabled colors ×0.5 | rectangular row, 48-point height, card fill + 1-point muted outline when selected, 14/12 hard-coded text | rounded 10 surface inset 4 per side, 30-point desktop (44 touch) minimum height, `secondary` fill on hover/press/selected, no border, token typography, disabled ×0.5, focus ring in `primary` | leading/trailing icon slots, grouping, dividers, superellipse corners, 500-weight labels |
| **Tile** | `FTileStyle`: `card` fill + 1-point `border`, radius 10, content padding 15/14.5, `FSizes.tile` = 48 | delegated to the old item (rect outline, 48 height) | card + border + radius 10, 15/14.5 padding, `secondary` on hover/selected, 48-point minimum height | icon/suffix slots, groups, long press |
| **Switch** | CupertinoSwitch geometry: 51x31 track, 27-point (r 13.5) thumb, 2-point inset; track `secondary`/`primary`, thumb `background`/`foreground` | 36x20 track, r 8 thumb, r 10 track | 51x31 track (r 15.5), thumb r 13.5 centred 15.5 from the track end, upstream colors, disabled single 0.5 dim | drag gestures, thumb shadow, transition animation; the unselected 1-point `mutedForeground` outline is kept on purpose (upstream has none) so the off track keeps 3:1 non-text contrast |
| **Checkbox** | size 16 desktop / 20 touch, icon 14/18, 0.6 border `mutedForeground`, radius `xs2` = 4, label `body.sm` 500, trailing padding 8 | correct box/border/radius, label used egui `Body` 13 and 4-point gap | unchanged geometry + label `body.size` (14/16) and 8-point gap | check-mark icon shape, error icon, transition |
| **Radio** | outer 16 desktop / 20 touch, indicator 8/12, 1-point border, 3-point padding | outer 14/18, indicator 12/18 | outer 16/20, indicator 8/12, 8-point label gap, token label size | mutual exclusion/group, animation |
| **Button** | `FButtonStyle`: min 24/32/36/40 desktop (32/40/44/48 touch), content padding xs 8/6, sm 10/9, md 10/11, lg 10/13 desktop (touch 10/9, 12/12, 12/14, 12/16), radius md 10 (xs sm 8), text `body.sm` | square minimums only, egui default padding, `Body` 13, hover HSL helper private | token padding per size/mode, token text size, hover through `Theme::hover`, disabled dim 0.5 once | icon slots, selected/selected-hover variants, 500 weight |
| **Tabs** | container `muted` fill + 1-point `muted` border, radius 10, padding 4, tab min height 36, indicator `background` radius 10, label `body.sm` 500 muted→foreground | muted fill/radius/padding/36 height, no border, 8-point gaps, `Body` 13 labels | border added, tabs edge-to-edge (gap 0), token label size | animated indicator, 500 weight |
| **Pagination** | item constraints 32x32 desktop / 44x44 touch, radius 10, idle transparent, hover `secondary`, selected `primary` + `primaryForeground`, no borders; ellipsis `body.sm` | filled page buttons, 32 targets, `Body` 13 | transparent idle, `secondary` hover, `primary` selected, 32/44 targets, token text, radius 10, borderless | icon/chevron-only actions (API takes caller labels), focus outline token |
| **Dialog** | card fill, 1-point border, radius md 10, `barrier` backdrop; content padding 16 and title `display.md` 16/18 in the golden composition | egui `Modal` default popup frame and `black alpha 100` backdrop | `Dialog::theme` draws card + border + radius 10 + `theme.barrier`; the review sheet mirrors the golden's 16-point pad, 16/18 title and `body.sm` copy | content typography still caller-owned; blur needs a new dependency and stays out; scale/fade animation |
| **Toast** | card + 1-point border, radius 10, padding 16/12 desktop (16/14 touch), title `display.sm` 14/16, description `body.sm` muted, spacing 2/4 | 12-point all padding, `.strong()` 13 title, 13 description | upstream padding and spacing, token title/description sizes and colors | weight, icons, exit animation |
| **Alert** | card + 1-point border, radius 10, padding 16/12, icon `body.md` 16/18 then 10-point gap, title `display.sm` 14/16, subtitle `body.sm` desktop / `body.xs` touch | 16 icon + 2 gap, bold 13 title, 13 subtitle | 16/18 icon, 10-point gap, token title/subtitle sizes, destructive colors unchanged | icon glyphs, weight |
| **Header** | min height 54 desktop / 62 touch, title `display.xl2` 22/30 700, `pagePadding` 12 horizontal / 8 top / 10 bottom | single 30-point title, no padding | 22/30 title, pagePadding margins, min height preserved | 700 weight, nested header |
| **Breadcrumb** | `body.sm` 14/16 text, muted→foreground, items 6 points horizontal each (touch +10 vertical) | `Body` 13, 8-point gaps, 4/2 button padding | token size, 12-point gaps (6+6), touch vertical padding | collapsed menu, chevron icon, hover transition |
| **Tooltip** | sm radius 8, inverted surface (`foreground` fill, `background` text), `body.xs` 12, 12/6 padding | egui default tooltip window frame | inverted fill/text derived from `Visuals`, radius 8, 12/6 padding, body−2 text size | arrow, pointer-following anchor, exact 500 ms hover delays are egui's |
| **Scaffold** | `FStyle.pagePadding` 12 horizontal / 8 vertical | 12 horizontal / 0 vertical | 12/8 | safe areas, keyboard insets |
| **Progress** | height 6, pill radius | already 6 / radius 3 | verified unchanged (pinned by existing tests) | indeterminate curve parity |

## Verification

- `cargo fmt --check`: pass.
- `cargo clippy --offline --all-targets --all-features -- -D warnings`: pass.
- `cargo nextest run --offline --all-features`: **239 tests run, 239 passed**
  (was 227 before this pass; +12 geometry/token tests, all pre-existing tests
  still pass, none loosened).
- `cargo test --offline --doc --all-features`: pass.
- `python3 harness/port/generate_palette.py --check`: pass.
- Rendered review against the reviewed baselines (`harness/visual-review/baselines`),
  changed pixel counts:
  - `light.png` 20,832/288,000 (7.2 %), `dark.png` 7.2 %, 150 % sheets 15.1 %/15.2 %
  - `item-*` 21.8–22.6 %, `tile-*` 28.1–52.1 %, `switch-*-150` 8.2–8.6 %
  - `toast-*` 2.6–2.9 %, `scaffold-*` 1.3–2.2 %, `dialog-*` ≈96 % (new themed surface)
  - `context-*`, `nav-*`, `popover-*`, `select-*` 0 % (untouched)
- Images inspected in both themes and at 150 %: no clipping, overlap or contrast
  regression. `touch-{light,dark}.png` are new screenshots using
  `Theme::light().touch()` to compare against Forui's touch goldens.

## Accessibility notes

- All rewrites keep the same semantic nodes: Item and Tile still expose
  `selected`, description, keyboard Enter/Space activation, focus ring and
  disabled state; Switch/Checkbox/Radio keep their roles and dim-disabled
  states.
- Selected/hovered item rows paint text on `secondary`, where Forui's
  `mutedForeground` (4.35:1) and light `destructive` (4.38:1) are just below
  WCAG AA. `Theme::readable_on` nudges those active-row colors to ≥4.5:1 while
  keeping hue; idle rows keep the upstream colors. This is a documented,
  minimal deviation.
- The switch keeps a 1-point `mutedForeground` outline on the off track
  (3:1+ non-text contrast) although Forui draws none; the focus ring is
  unchanged. Pagination keeps the caller's text labels instead of icon-only
  chevrons, which also keeps accessible names.
- No test was relaxed to accommodate a regression.

## Remaining gaps (unavoidable without new dependencies or upstream assets)

- **Font weights**: egui's bundled fonts have a single weight; Forui's
  500/600/700 tokens are stored (`Typography`) and documented but not rendered.
- **Icons**: Forui ships a Lucide icon font; no icon font or image assets may be
  added, so icon slots/chevrons/check marks use text glyphs or are omitted.
- **Blur**: dialog/tooltip background filters need a renderer extension; only
  the translucent barrier color is ported.
- **Superellipse corners**: Flutter's `RoundedSuperellipseBorder` is approximated
  with rounded rectangles.
- **Flutter text metrics/layout**: line heights, leading distribution and
  intrinsic layout differ from egui; wrapping was checked at 200 and 300 px and
  at 150 % zoom but is not pixel-identical.
- **Animation parity**: dialog scale/fade, switch/tabs/checkbox transitions and
  tooltip delays follow the existing reduced-motion helper only where integrated
  (toast), not Forui's curves.
- **Select**: its public API has no theme parameter, so its field radius/height
  remain egui defaults; it needs a theme-aware constructor before it can be
  aligned. `context_menu`/`popover`/`bottom_navigation` were out of this pass.

## Suggested `docs/COVERAGE.md` lines

- Visual parity pass (2026-11-01): Item/Tile now use Forui's rounded md container
  and `secondary` selected/hover fill with 30/48 point heights; Switch matches
  Cupertino's 51x31 track and 27-point thumb; Button content padding matches
  `FButtonStyle` per size; Tabs gained the 1-point muted border; Pagination uses
  32/44 point items with primary selected fill; Dialog accepts a theme for the
  card/border/radius/barrier; Toast/Alert/Header/Breadcrumb/Tooltip use Forui
  radii, padding and `FTypography` sizes; `Theme` gained `touch`,
  `disabledOpacity`, `disable`, `hover` and WCAG contrast helpers. Upstream
  parity is measurably closer but still unproven: font weights, icons, blur,
  superellipse corners and animation curves remain gaps.
