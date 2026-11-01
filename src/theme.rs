//! Explicit design tokens adapted from Forui's neutral color schemes,
//! typography scale, size tokens, breakpoints and motion settings.
//!
//! [`Theme`] keeps the original color fields stable and adds the semantic
//! typography, size, breakpoint and motion tokens that migrated widgets read.
//! Every field is public, so applications can override tokens without a
//! builder. Typography colors are derived from the palette when the theme is
//! constructed ([`Theme::light`], [`Theme::dark`], [`Theme::touch`]); after
//! changing `foreground` or `muted_foreground` directly, override
//! `theme.typography` as well.
use egui::{Color32, FontId, TextStyle as EguiTextStyle};

use crate::tokens::{Breakpoints, Sizes};

/// Logical font weights used by Forui's semantic text styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontWeight {
    /// 400: body copy and captions.
    Regular,
    /// 500: labels, buttons, badges and other interactive text.
    Medium,
    /// 600: dialog and card titles.
    Semibold,
    /// 700: prominent header titles.
    Bold,
}

impl FontWeight {
    /// The numeric CSS weight (`400`, `500`, `600` or `700`).
    ///
    /// egui's bundled fonts expose a single weight, so this value is a token for
    /// applications that install weighted font families and for documentation
    /// of the upstream style it was derived from.
    pub const fn number(self) -> u16 {
        match self {
            Self::Regular => 400,
            Self::Medium => 500,
            Self::Semibold => 600,
            Self::Bold => 700,
        }
    }
}

/// A semantic text style with a Forui-derived size, weight, color and line
/// height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyleToken {
    /// Font size in logical pixels.
    pub size: f32,
    /// Logical font weight. See [`FontWeight::number`].
    pub weight: FontWeight,
    /// Text color, derived from the palette.
    pub color: Color32,
    /// Line height as a multiple of the font size.
    pub line_height: f32,
}

impl TextStyleToken {
    /// The egui font identifier for this token.
    ///
    /// egui's built-in fonts have one weight per family, so [`Self::weight`] is
    /// not applied here.
    pub fn font_id(&self) -> FontId {
        FontId::proportional(self.size)
    }
}

/// The semantic text styles from Forui's `FTypography`.
///
/// The tokens follow Forui's desktop scale by default; [`Typography::touch`]
/// uses its larger touch scale. Sizes are taken from `FTypeface` (`display.lg`
/// for headings and `body.sm`/`body.xs` for content) and weights from the
/// styles Forui widgets apply to those faces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typography {
    /// Section and page headings (`display.lg`, medium weight).
    pub heading: TextStyleToken,
    /// Content text (`body.sm`, regular weight).
    pub body: TextStyleToken,
    /// Button labels (`body.sm`, medium weight, tight line height).
    pub button: TextStyleToken,
    /// Field and control labels (`body.sm`, medium weight).
    pub label: TextStyleToken,
    /// Secondary and caption text (`body.xs`, regular weight, muted color).
    pub caption: TextStyleToken,
}

impl Typography {
    /// Forui's desktop typography scale: 18 pt headings, 14 pt content.
    pub fn desktop(foreground: Color32, muted_foreground: Color32) -> Self {
        Self {
            heading: TextStyleToken {
                size: 18.0,
                weight: FontWeight::Semibold,
                color: foreground,
                line_height: 1.75,
            },
            body: TextStyleToken {
                size: 14.0,
                weight: FontWeight::Regular,
                color: foreground,
                line_height: 1.25,
            },
            button: TextStyleToken {
                size: 14.0,
                weight: FontWeight::Medium,
                color: foreground,
                line_height: 1.0,
            },
            label: TextStyleToken {
                size: 14.0,
                weight: FontWeight::Medium,
                color: foreground,
                line_height: 1.25,
            },
            caption: TextStyleToken {
                size: 12.0,
                weight: FontWeight::Regular,
                color: muted_foreground,
                line_height: 1.25,
            },
        }
    }

    /// Forui's touch typography scale: 20 pt headings, 16 pt content.
    pub fn touch(foreground: Color32, muted_foreground: Color32) -> Self {
        Self {
            heading: TextStyleToken {
                size: 20.0,
                weight: FontWeight::Semibold,
                color: foreground,
                line_height: 1.75,
            },
            body: TextStyleToken {
                size: 16.0,
                weight: FontWeight::Regular,
                color: foreground,
                line_height: 1.5,
            },
            button: TextStyleToken {
                size: 16.0,
                weight: FontWeight::Medium,
                color: foreground,
                line_height: 1.0,
            },
            label: TextStyleToken {
                size: 16.0,
                weight: FontWeight::Medium,
                color: foreground,
                line_height: 1.5,
            },
            caption: TextStyleToken {
                size: 14.0,
                weight: FontWeight::Regular,
                color: muted_foreground,
                line_height: 1.25,
            },
        }
    }

    /// Apply the token sizes to egui's named text styles.
    ///
    /// `Heading`, `Body`, `Button` and `Small` receive the heading, body, button
    /// and caption sizes. `Monospace` is left unchanged. Weights and colors stay
    /// on the tokens because egui's default font families have a single weight;
    /// text color comes from the active visuals or from callers passing
    /// [`TextStyleToken::color`].
    pub fn apply(&self, style: &mut egui::Style) {
        for (text_style, token) in [
            (EguiTextStyle::Heading, self.heading),
            (EguiTextStyle::Body, self.body),
            (EguiTextStyle::Button, self.button),
            (EguiTextStyle::Small, self.caption),
        ] {
            style
                .text_styles
                .insert(text_style, FontId::proportional(token.size));
        }
    }
}

/// A plain `f32` easing curve, evaluated with no external dependency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    /// Linear interpolation; Forui's default theme transition curve.
    #[default]
    Linear,
    /// Accelerating quadratic curve (`t²`).
    EaseIn,
    /// Decelerating quadratic curve (`1 - (1 - t)²`).
    EaseOut,
    /// Accelerating then decelerating quadratic curve.
    EaseInOut,
}

impl Easing {
    /// Evaluate the curve at `t`.
    ///
    /// Values outside `0..=1` are clamped and non-finite inputs evaluate to
    /// `0.0`, so callers can use this directly on animation progress.
    pub fn apply(self, t: f32) -> f32 {
        if !t.is_finite() {
            return 0.0;
        }
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
        }
    }
}

/// Motion settings derived from Forui's `FThemeMotion` and the animation
/// durations used by its widgets.
///
/// The helper in [`crate::motion`] snaps to the target when [`Self::enabled`]
/// is false, when [`Self::reduced_motion`] is set, or when egui's active style
/// has `animation_time == 0.0` (in egui 0.36 that style is
/// `Context::global_style()`; older releases call it `Context::style()`). egui does not read the operating
/// system's reduced-motion preference, so applications that observe it must set
/// [`Self::reduced_motion`] themselves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionSettings {
    /// Master switch for animations ported by this crate.
    pub enabled: bool,
    /// Accessibility preference that forces animations to snap to their target.
    pub reduced_motion: bool,
    /// Duration in seconds of quick fades such as popover and tooltip entrances
    /// (Forui defaults to 100 ms).
    pub fast: f32,
    /// Duration in seconds of standard transitions such as theme and modal
    /// fades (Forui's `FThemeMotion.duration` defaults to 200 ms).
    pub normal: f32,
    /// Duration in seconds of larger transitions such as tab changes (Forui
    /// defaults to 300 ms).
    pub slow: f32,
    /// Easing curve. Forui's theme motion defaults to a linear curve.
    pub easing: Easing,
}

impl Default for MotionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            reduced_motion: false,
            fast: 0.1,
            normal: 0.2,
            slow: 0.3,
            easing: Easing::Linear,
        }
    }
}

impl MotionSettings {
    /// Whether animations may run in `ctx`.
    ///
    /// Requires the master switch to be on, reduced motion to be off and
    /// egui's `animation_time` to be positive.
    pub fn animations_enabled(&self, ctx: &egui::Context) -> bool {
        self.enabled && !self.reduced_motion && ctx.global_style().animation_time > 0.0
    }
}

/// The full design token set: neutral palette, typography, sizes, breakpoints
/// and motion.
///
/// Consumers may override any field. Equality is derived from the color,
/// typography and motion values; the type is not `Eq` because several tokens
/// are floating point, as in egui.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub dark: bool,
    pub background: Color32,
    pub foreground: Color32,
    pub primary: Color32,
    pub primary_foreground: Color32,
    pub secondary: Color32,
    pub secondary_foreground: Color32,
    pub muted: Color32,
    pub muted_foreground: Color32,
    pub destructive: Color32,
    pub destructive_foreground: Color32,
    pub error: Color32,
    pub error_foreground: Color32,
    pub card: Color32,
    pub border: Color32,
    pub barrier: Color32,
    /// Semantic text styles derived from Forui's `FTypography`.
    pub typography: Typography,
    /// Reference control heights from Forui's `FSizes`.
    pub sizes: Sizes,
    /// Responsive breakpoints from Forui's `FBreakpoints`.
    pub breakpoints: Breakpoints,
    /// Motion and easing settings derived from Forui's `FThemeMotion`.
    pub motion: MotionSettings,
    /// Whether this theme uses Forui's touch platform variant. [`Theme::light`]
    /// and [`Theme::dark`] are desktop themes; [`Theme::touch`] sets this flag.
    pub touch: bool,
    /// Forui's `FColors.disabledOpacity`: the alpha multiplier applied to
    /// disabled widget colors. Defaults to 0.5.
    pub disabled_opacity: f32,
}

impl Theme {
    /// Forui's neutral light palette with the desktop token scale.
    pub fn light() -> Self {
        crate::generated_palette::light()
    }

    /// Forui's neutral dark palette with the desktop token scale.
    pub fn dark() -> Self {
        crate::generated_palette::dark()
    }

    /// Return this theme with Forui's touch token scale: larger control heights
    /// and typography sizes, with the current palette colors retained.
    pub fn touch(mut self) -> Self {
        self.touch = true;
        self.sizes = Sizes::touch();
        self.typography = Typography::touch(self.foreground, self.muted_foreground);
        self
    }

    /// A high-contrast variant of this theme.
    ///
    /// The pinned Forui neutral palettes put the destructive button and badge
    /// text below the WCAG AA threshold of 4.5:1 in some states (see
    /// `docs/ACCESSIBILITY.md`). Those components render [`Self::destructive`]
    /// as both the text colour and the translucent fill, so the preset adjusts
    /// that one token until the pair *as rendered* passes:
    ///
    /// * light theme: darken the destructive colour until the base and hovered
    ///   destructive fills over the page/card backgrounds and the destructive
    ///   text on the page background all reach 4.5:1;
    /// * dark theme: lighten the destructive colour for the same rendered
    ///   pairs, because the tint is composited over a dark surface;
    /// * both themes: move the muted foreground away from the muted surface
    ///   until it reaches 4.5:1 on the muted, page and card surfaces.
    ///
    /// Hue, alpha and every unrelated token are preserved; `touch()` composes
    /// and [`Self::destructive_foreground`] is left untouched (the current
    /// destructive components derive their label colour from
    /// [`Self::destructive`]). Call it after [`Theme::touch`] if both variants
    /// are wanted; the preset is idempotent and does not change a theme that
    /// already meets the ratios.
    ///
    /// The adjustment treats the tokens as opaque and the rendered fill as a
    /// single translucent layer; custom translucent tokens or stacked fills may
    /// need a component-level policy instead.
    pub fn high_contrast(mut self) -> Self {
        let (dark, background, card) = (self.dark, self.background, self.card);
        let passes = |candidate: Color32| {
            [
                Self::destructive_pair_contrast(candidate, dark, false, background),
                Self::destructive_pair_contrast(candidate, dark, true, background),
                Self::destructive_pair_contrast(candidate, dark, false, card),
                Self::destructive_pair_contrast(candidate, dark, true, card),
                Self::contrast_ratio(candidate, background),
            ]
            .into_iter()
            .all(|ratio| ratio >= 4.5)
        };
        let target = if dark { Color32::WHITE } else { Color32::BLACK };
        self.destructive = Self::adjust_until(self.destructive, target, passes);
        let muted_target = if Self::luminance(self.muted) > 0.5 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        self.muted_foreground =
            Self::adjust_until(self.muted_foreground, muted_target, |candidate| {
                Self::contrast_ratio(candidate, self.muted) >= 4.5
                    && Self::contrast_ratio(candidate, self.background) >= 4.5
                    && Self::contrast_ratio(candidate, self.card) >= 4.5
            });
        self.typography.caption.color = self.muted_foreground;
        self
    }

    /// Contrast of the destructive colour against its translucent fill exactly
    /// as the destructive `Button`/`Badge` render it, composited over `surface`.
    ///
    /// The fill alphas mirror the components: `26`/`51` for the base and
    /// hovered fills in the light theme, and `51`/`77` in the dark theme.
    pub fn destructive_rendered_contrast(&self, hovered: bool, surface: Color32) -> f64 {
        Self::destructive_pair_contrast(self.destructive, self.dark, hovered, surface)
    }

    fn destructive_pair_contrast(
        destructive: Color32,
        dark: bool,
        hovered: bool,
        surface: Color32,
    ) -> f64 {
        let alpha = match (dark, hovered) {
            (false, false) => 26,
            (false, true) => 51,
            (true, false) => 51,
            (true, true) => 77,
        };
        let [red, green, blue, _] = destructive.to_srgba_unmultiplied();
        let fill = blend_over(
            Color32::from_rgba_unmultiplied(red, green, blue, alpha),
            surface,
        );
        Self::contrast_ratio(destructive, fill)
    }

    /// Blend `color` towards `target` in small steps until `satisfied` accepts
    /// the result, returning the last candidate if no step qualifies.
    fn adjust_until(
        color: Color32,
        target: Color32,
        mut satisfied: impl FnMut(Color32) -> bool,
    ) -> Color32 {
        if satisfied(color) {
            return color;
        }
        let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
        let [target_red, target_green, target_blue, _] = target.to_srgba_unmultiplied();
        let mix = |from: u8, to: u8, amount: f32| {
            (f32::from(from) + (f32::from(to) - f32::from(from)) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        let mut candidate = color;
        for step in 1..=50 {
            let amount = step as f32 / 50.0;
            candidate = Color32::from_rgba_unmultiplied(
                mix(red, target_red, amount),
                mix(green, target_green, amount),
                mix(blue, target_blue, amount),
                alpha,
            );
            if satisfied(candidate) {
                return candidate;
            }
        }
        candidate
    }

    /// WCAG 2.2 relative luminance of an opaque color.
    pub fn luminance(color: Color32) -> f64 {
        let linear = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.040_45 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }

    /// WCAG 2.2 contrast ratio between two opaque colors, from 1.0 to 21.0.
    pub fn contrast_ratio(foreground: Color32, background: Color32) -> f64 {
        let (foreground, background) = (Self::luminance(foreground), Self::luminance(background));
        (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
    }

    /// Adjust `color` minimally so it reaches `minimum_ratio` contrast against
    /// `background`, blending towards black on light backgrounds and towards
    /// white on dark ones.
    ///
    /// Forui paints secondary text on the `secondary` fill and destructive text
    /// on tinted fills; in the neutral light palette those combinations are
    /// slightly below WCAG AA (about 4.35:1). The helper keeps the upstream hue
    /// and changes only as much as the contrast requirement needs.
    pub fn readable_on(&self, color: Color32, background: Color32, minimum_ratio: f32) -> Color32 {
        if Self::contrast_ratio(color, background) >= f64::from(minimum_ratio) {
            return color;
        }
        let target = if Self::luminance(background) > 0.5 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
        let [target_red, target_green, target_blue, _] = target.to_srgba_unmultiplied();
        let mix = |from: u8, to: u8, amount: f32| {
            (f32::from(from) + (f32::from(to) - f32::from(from)) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        let mut adjusted = color;
        for step in 1..=20 {
            let amount = step as f32 / 20.0;
            let candidate = Color32::from_rgba_unmultiplied(
                mix(red, target_red, amount),
                mix(green, target_green, amount),
                mix(blue, target_blue, amount),
                alpha,
            );
            adjusted = candidate;
            if Self::contrast_ratio(candidate, background) >= f64::from(minimum_ratio) {
                break;
            }
        }
        adjusted
    }

    /// Forui's disabled variant of `color`: the alpha channel is multiplied by
    /// [`Self::disabled_opacity`] (0.5 by default). RGB is premultiplied to keep
    /// the color compositing correctly over other widgets.
    pub fn disable(&self, color: Color32) -> Color32 {
        let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
        let alpha = (f32::from(alpha) * self.disabled_opacity.clamp(0.0, 1.0)).round() as u8;
        Color32::from_rgba_unmultiplied(red, green, blue, alpha)
    }

    /// Forui's `FColors.disable(color, background)` variant: [`Self::disable`]
    /// composited over `background`, for cases where a translucent disabled
    /// color would otherwise reveal unrelated widgets underneath.
    pub fn disable_blend(&self, color: Color32, background: Color32) -> Color32 {
        let [red, green, blue, alpha] = self.disable(color).to_srgba_unmultiplied();
        let [
            background_red,
            background_green,
            background_blue,
            background_alpha,
        ] = background.to_srgba_unmultiplied();
        if alpha == 255 || background_alpha == 0 {
            return Color32::from_rgba_unmultiplied(red, green, blue, alpha);
        }
        let alpha_f = f32::from(alpha) / 255.0;
        let background_alpha_f = f32::from(background_alpha) / 255.0;
        let mix = |foreground: u8, background: u8| {
            let value = f32::from(foreground) * alpha_f
                + f32::from(background) * background_alpha_f * (1.0 - alpha_f);
            value.round().clamp(0.0, 255.0) as u8
        };
        Color32::from_rgba_unmultiplied(
            mix(red, background_red),
            mix(green, background_green),
            mix(blue, background_blue),
            ((alpha_f + background_alpha_f * (1.0 - alpha_f)) * 255.0).round() as u8,
        )
    }

    /// Forui's `FColors.hover` transformation: nudge the color's HSL lightness
    /// away from the mid-point, preserving hue, saturation and alpha. Light
    /// colors darken by the `hoverDarken` factor (0.05) and dark colors lighten
    /// by `hoverLighten` (0.075), more aggressively near the extremes.
    pub fn hover(&self, color: Color32) -> Color32 {
        let [red, green, blue, alpha] = color.to_srgba_unmultiplied();
        let [red, green, blue] = [
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
        ];
        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let lightness = (max + min) / 2.0;
        let delta = max - min;
        let saturation = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lightness - 1.0).abs()).max(f32::EPSILON)
        };
        let hue = if delta == 0.0 {
            0.0
        } else if max == red {
            ((green - blue) / delta).rem_euclid(6.0)
        } else if max == green {
            (blue - red) / delta + 2.0
        } else {
            (red - green) / delta + 4.0
        };
        let space = if lightness > 0.5 {
            1.0 - lightness
        } else {
            lightness
        };
        let factor = if lightness > 0.5 { -0.05 } else { 0.075 };
        let lightness = (lightness + (1.0 + (0.5 - space) / 0.5) * factor).clamp(0.0, 1.0);
        let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let x = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match hue as u8 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = lightness - chroma / 2.0;
        Color32::from_rgba_unmultiplied(
            ((r + m) * 255.0).round() as u8,
            ((g + m) * 255.0).round() as u8,
            ((b + m) * 255.0).round() as u8,
            alpha,
        )
    }

    /// Configure egui's core visual colors and named text style sizes for a UI
    /// context.
    ///
    /// Visuals and text style sizes are applied to both stored egui theme
    /// styles, so an operating-system light/dark switch does not discard the
    /// explicit Temivu palette or the typography tokens. Font weights and
    /// semantic colors stay on [`Theme::typography`]; egui's bundled fonts
    /// expose a single weight and text color is driven by the visuals.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = self.background;
        visuals.window_fill = self.card;
        visuals.override_text_color = Some(self.foreground);
        visuals.selection.bg_fill = self.primary;
        visuals.selection.stroke.color = self.primary_foreground;
        // Write both stored theme styles so an operating-system theme change
        // cannot silently replace the explicit Temivu palette with egui's.
        ctx.set_visuals_of(egui::Theme::Light, visuals.clone());
        ctx.set_visuals_of(egui::Theme::Dark, visuals);
        let typography = self.typography;
        ctx.all_styles_mut(|style| typography.apply(style));
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

/// Composite a translucent `overlay` over an opaque `background` in sRGB space.
fn blend_over(overlay: Color32, background: Color32) -> Color32 {
    let [red, green, blue, alpha] = overlay.to_srgba_unmultiplied();
    let [background_red, background_green, background_blue, _] = background.to_srgba_unmultiplied();
    let alpha_f = f32::from(alpha) / 255.0;
    let mix = |overlay: u8, background: u8| {
        (f32::from(overlay) * alpha_f + f32::from(background) * (1.0 - alpha_f))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(red, background_red),
        mix(green, background_green),
        mix(blue, background_blue),
    )
}
