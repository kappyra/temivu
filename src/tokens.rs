//! Responsive and size reference tokens adapted from Forui's `FSizes` and
//! `FBreakpoints`. These are reference values; they do not resize all widgets.

/// The four field-height scales, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldSizes {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

/// Reference heights for field, menu item, tile and calendar controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    pub field: FieldSizes,
    pub item: f32,
    pub tile: f32,
    pub calendar: f32,
}

impl Sizes {
    /// Forui desktop reference heights.
    pub const fn desktop() -> Self {
        Self {
            field: FieldSizes {
                xs: 24.0,
                sm: 32.0,
                md: 36.0,
                lg: 40.0,
            },
            item: 30.0,
            tile: 48.0,
            calendar: 32.0,
        }
    }

    /// Forui touch reference heights.
    pub const fn touch() -> Self {
        Self {
            field: FieldSizes {
                xs: 32.0,
                sm: 40.0,
                md: 44.0,
                lg: 48.0,
            },
            item: 44.0,
            tile: 48.0,
            calendar: 44.0,
        }
    }

    /// Multiply every reference height by `factor`.
    pub fn scale(self, factor: f32) -> Self {
        Self {
            field: FieldSizes {
                xs: self.field.xs * factor,
                sm: self.field.sm * factor,
                md: self.field.md * factor,
                lg: self.field.lg * factor,
            },
            item: self.item * factor,
            tile: self.tile * factor,
            calendar: self.calendar * factor,
        }
    }

    /// Linearly interpolate each field; `t` may extrapolate beyond zero or one.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let mix = |a: f32, b: f32| a + (b - a) * t;
        Self {
            field: FieldSizes {
                xs: mix(self.field.xs, other.field.xs),
                sm: mix(self.field.sm, other.field.sm),
                md: mix(self.field.md, other.field.md),
                lg: mix(self.field.lg, other.field.lg),
            },
            item: mix(self.item, other.item),
            tile: mix(self.tile, other.tile),
            calendar: mix(self.calendar, other.calendar),
        }
    }
}

/// Minimum logical widths for the five responsive breakpoints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Breakpoints {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xl2: f32,
}

/// Largest breakpoint whose minimum width is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportClass {
    BelowSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    ExtraExtraLarge,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            sm: 640.0,
            md: 768.0,
            lg: 1024.0,
            xl: 1280.0,
            xl2: 1536.0,
        }
    }
}

impl Breakpoints {
    /// Classify a finite, non-negative width using ordered, finite breakpoint values.
    pub fn classify(self, width: f32) -> Result<ViewportClass, &'static str> {
        let limits = [self.sm, self.md, self.lg, self.xl, self.xl2];
        if !width.is_finite()
            || width < 0.0
            || limits.iter().any(|v| !v.is_finite() || *v < 0.0)
            || limits.windows(2).any(|pair| pair[0] > pair[1])
        {
            return Err("width and ordered breakpoints must be finite and non-negative");
        }
        Ok(if width < self.sm {
            ViewportClass::BelowSmall
        } else if width < self.md {
            ViewportClass::Small
        } else if width < self.lg {
            ViewportClass::Medium
        } else if width < self.xl {
            ViewportClass::Large
        } else if width < self.xl2 {
            ViewportClass::ExtraLarge
        } else {
            ViewportClass::ExtraExtraLarge
        })
    }
}
