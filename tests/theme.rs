use egui::Color32;
use temivu::{
    Breakpoints, Easing, FontWeight, MotionSettings, Sizes, Theme, Typography, ViewportClass,
};

#[test]
fn spring_motion_is_bounded_and_settles() {
    let mut previous = 0.0;
    for step in 0..=100 {
        let value = Easing::Spring.apply(step as f32 / 100.0);
        assert!((0.0..=1.0 + f32::EPSILON).contains(&value));
        assert!(value + f32::EPSILON >= previous);
        previous = value;
    }
    assert!((previous - 1.0).abs() < 0.0001);
}

#[test]
fn mint_palettes_keep_text_readable_and_support_touch() {
    for theme in [Theme::mint_light(), Theme::mint_dark()] {
        for (foreground, background) in [
            (theme.foreground, theme.background),
            (theme.foreground, theme.card),
            (theme.muted_foreground, theme.background),
            (theme.primary_foreground, theme.primary),
        ] {
            assert!(Theme::contrast_ratio(foreground, background) >= 4.5);
        }
        assert_eq!(theme.typography.body.color, theme.foreground);
        assert_eq!(theme.clone().touch().sizes.field.md, 44.0);
    }
}

#[test]
fn generated_neutral_palettes_keep_argb_tokens_and_transparency() {
    let light = Theme::light();
    let dark = Theme::dark();
    assert_eq!(light.primary, Color32::from_rgb(23, 23, 23));
    assert_eq!(light.destructive, Color32::from_rgb(231, 0, 11));
    assert_eq!(light.barrier.to_srgba_unmultiplied(), [0, 0, 0, 51]);
    assert_eq!(dark.primary, Color32::from_rgb(229, 229, 229));
    assert_eq!(dark.border.to_srgba_unmultiplied(), [255, 255, 255, 26]);
    assert_eq!(dark.barrier.to_srgba_unmultiplied(), [0, 0, 0, 122]);
}

#[test]
fn typography_defaults_follow_forui_desktop_scale() {
    let light = Theme::light();
    let typography = light.typography;
    assert_eq!(
        (
            typography.heading.size,
            typography.body.size,
            typography.button.size,
            typography.label.size,
            typography.caption.size,
        ),
        (18.0, 14.0, 14.0, 14.0, 12.0)
    );
    assert_eq!(typography.heading.weight, FontWeight::Semibold);
    assert_eq!(typography.body.weight, FontWeight::Regular);
    assert_eq!(typography.button.weight, FontWeight::Medium);
    assert_eq!(typography.label.weight, FontWeight::Medium);
    assert_eq!(typography.heading.line_height, 1.75);
    assert_eq!(typography.button.line_height, 1.0);
    assert_eq!(typography.body.color, light.foreground);
    assert_eq!(typography.caption.color, light.muted_foreground);
    assert_eq!(typography.body.font_id().size, 14.0);

    let dark = Theme::dark();
    assert_eq!(dark.typography.body.color, dark.foreground);
    assert_eq!(dark.typography.caption.color, dark.muted_foreground);
    assert_ne!(
        dark.typography.body.color,
        Theme::light().typography.body.color
    );
}

#[test]
fn font_weights_map_to_css_numbers() {
    assert_eq!(FontWeight::Regular.number(), 400);
    assert_eq!(FontWeight::Medium.number(), 500);
    assert_eq!(FontWeight::Semibold.number(), 600);
    assert_eq!(FontWeight::Bold.number(), 700);
}

#[test]
fn theme_exposes_sizes_and_breakpoints_and_a_touch_scale() {
    let theme = Theme::light();
    assert_eq!(theme.sizes, Sizes::desktop());
    assert_eq!(theme.sizes.field.md, 36.0);
    assert_eq!(theme.breakpoints, Breakpoints::default());
    assert_eq!(theme.breakpoints.classify(800.0), Ok(ViewportClass::Medium));

    let touch = theme.clone().touch();
    assert_eq!(touch.sizes, Sizes::touch());
    assert_eq!(touch.sizes.field.md, 44.0);
    assert_eq!(touch.breakpoints, theme.breakpoints);
    assert_eq!(touch.typography.heading.size, 20.0);
    assert_eq!(touch.typography.body.size, 16.0);
    assert_eq!(touch.typography.caption.size, 14.0);
    assert_eq!(touch.typography.body.color, touch.foreground);
    assert_eq!(touch.background, theme.background);

    // Semantic presets match the standalone token constructors.
    assert_eq!(
        Typography::desktop(theme.foreground, theme.muted_foreground),
        theme.typography
    );
    assert_eq!(
        Typography::touch(touch.foreground, touch.muted_foreground),
        touch.typography
    );
}

#[test]
fn typography_and_motion_fields_can_be_overridden() {
    let mut theme = Theme::light();
    assert_eq!(theme, Theme::light());
    theme.typography.heading.size = 26.0;
    theme.typography.body.weight = FontWeight::Bold;
    theme.typography.caption.color = Color32::RED;
    assert_ne!(theme, Theme::light());
    assert_eq!(theme.typography.heading.size, 26.0);
    assert_eq!(theme.typography.body.weight, FontWeight::Bold);
    assert_eq!(theme.typography.caption.color, Color32::RED);

    // Copying a theme preserves every token.
    let copy = theme.clone();
    assert_eq!(copy, theme);
    assert_eq!(copy.sizes, theme.sizes);
    assert_eq!(copy.motion, theme.motion);
}

#[test]
fn apply_configures_visuals_and_named_text_styles() {
    let mut theme = Theme::dark();
    theme.typography.heading.size = 26.0;
    theme.typography.body.size = 15.0;
    theme.typography.button.size = 13.0;
    theme.typography.caption.size = 11.0;
    let ctx = egui::Context::default();
    theme.apply(&ctx);
    let style = ctx.global_style();
    assert_eq!(
        style.text_styles[&egui::TextStyle::Heading].size,
        theme.typography.heading.size
    );
    assert_eq!(
        style.text_styles[&egui::TextStyle::Body].size,
        theme.typography.body.size
    );
    assert_eq!(
        style.text_styles[&egui::TextStyle::Button].size,
        theme.typography.button.size
    );
    assert_eq!(
        style.text_styles[&egui::TextStyle::Small].size,
        theme.typography.caption.size
    );
    assert_eq!(style.visuals.panel_fill, theme.background);
    assert_eq!(style.visuals.window_fill, theme.card);
    assert_eq!(style.visuals.override_text_color, Some(theme.foreground));

    // Typography and the palette are applied to both stored styles, so egui
    // theme switches keep the token sizes and the explicit Temivu palette.
    theme.apply(&ctx);
    for stored in [egui::Theme::Dark, egui::Theme::Light] {
        let style = ctx.style_of(stored);
        assert_eq!(
            style.text_styles[&egui::TextStyle::Body].size,
            theme.typography.body.size
        );
        assert_eq!(style.visuals.panel_fill, theme.background);
        assert_eq!(style.visuals.window_fill, theme.card);
        assert_eq!(style.visuals.override_text_color, Some(theme.foreground));
        assert_eq!(style.visuals.selection.bg_fill, theme.primary);
    }
}

#[test]
fn motion_defaults_match_forui_durations_and_linear_easing() {
    let motion = Theme::light().motion;
    assert_eq!(motion, MotionSettings::default());
    assert!(motion.enabled);
    assert!(!motion.reduced_motion);
    assert_eq!(motion.fast, 0.1);
    assert_eq!(motion.normal, 0.2);
    assert_eq!(motion.slow, 0.3);
    assert_eq!(motion.easing, Easing::Linear);
    assert_eq!(Theme::dark().motion, MotionSettings::default());
}

#[test]
fn motion_settings_control_whether_animations_run() {
    let ctx = egui::Context::default();
    let motion = MotionSettings::default();
    assert!(motion.animations_enabled(&ctx));
    assert!(
        !MotionSettings {
            enabled: false,
            ..motion
        }
        .animations_enabled(&ctx)
    );
    assert!(
        !MotionSettings {
            reduced_motion: true,
            ..motion
        }
        .animations_enabled(&ctx)
    );
    ctx.all_styles_mut(|style| style.animation_time = 0.0);
    assert!(!motion.animations_enabled(&ctx));
    let mut overridden = Theme::light();
    overridden.motion.reduced_motion = true;
    assert_ne!(overridden, Theme::light());
    assert!(
        !overridden
            .motion
            .animations_enabled(&egui::Context::default())
    );
}

#[test]
fn easing_curves_are_plain_clamped_quadratics() {
    assert_eq!(Easing::Linear.apply(0.25), 0.25);
    assert_eq!(Easing::EaseIn.apply(0.5), 0.25);
    assert_eq!(Easing::EaseOut.apply(0.5), 0.75);
    assert_eq!(Easing::EaseInOut.apply(0.25), 0.125);
    assert_eq!(Easing::EaseInOut.apply(0.5), 0.5);
    assert_eq!(Easing::EaseInOut.apply(0.75), 0.875);
    for easing in [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
    ] {
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
        assert_eq!(easing.apply(-1.0), 0.0);
        assert_eq!(easing.apply(2.0), 1.0);
        assert_eq!(easing.apply(f32::NAN), 0.0);
        assert_eq!(easing.apply(f32::INFINITY), 0.0);
    }
}

#[test]
fn duration_overrides_are_stored_and_comparable() {
    let mut theme = Theme::light();
    theme.motion.fast = 0.05;
    theme.motion.normal = 0.15;
    theme.motion.slow = 0.4;
    theme.motion.easing = Easing::EaseOut;
    assert_eq!(theme.motion.fast, 0.05);
    assert_eq!(theme.motion.normal, 0.15);
    assert_eq!(theme.motion.slow, 0.4);
    assert_eq!(theme.motion.easing, Easing::EaseOut);
    assert_ne!(theme, Theme::light());
}

#[test]
fn theme_exposes_forui_disabled_hover_and_contrast_tokens() {
    let theme = Theme::light();
    assert!(!theme.touch);
    assert_eq!(theme.disabled_opacity, 0.5);
    assert!(theme.clone().touch().touch);

    let disabled = theme.disable(theme.secondary);
    assert_eq!(disabled.to_srgba_unmultiplied()[3], 128);
    // Blending a disabled color over an opaque background keeps it opaque.
    let blended = theme.disable_blend(theme.muted_foreground, theme.card);
    assert_eq!(blended.to_srgba_unmultiplied()[3], 255);

    // Forui's hover darkens light colors and lightens dark ones.
    assert_ne!(theme.hover(theme.secondary), theme.secondary);
    assert_ne!(theme.hover(theme.primary), theme.primary);
    let hovered_primary = theme.hover(theme.primary).to_srgba_unmultiplied();
    let primary = theme.primary.to_srgba_unmultiplied();
    assert!(
        hovered_primary[0] > primary[0],
        "dark colors lighten on hover"
    );

    assert_eq!(
        Theme::contrast_ratio(Color32::WHITE, Color32::BLACK).round() as i32,
        21
    );
    assert!(Theme::contrast_ratio(theme.foreground, theme.card) > 19.0);
    // The upstream muted-on-secondary pair is just below AA; readable_on fixes it.
    assert!(Theme::contrast_ratio(theme.muted_foreground, theme.secondary) < 4.5);
    let readable = theme.readable_on(theme.muted_foreground, theme.secondary, 4.5);
    assert!(Theme::contrast_ratio(readable, theme.secondary) >= 4.5);
    // Compliant colors are left untouched.
    assert_eq!(
        theme.readable_on(theme.foreground, theme.card, 4.5),
        theme.foreground
    );
}

#[test]
fn high_contrast_light_meets_the_rendered_destructive_pairs() {
    let base = Theme::light();
    let theme = base.clone().high_contrast();
    // The rendered pairs (destructive text on its translucent fill) are the ones
    // the destructive Button and Badge actually paint.
    assert!(
        base.destructive_rendered_contrast(false, base.background) < 4.5,
        "the upstream light destructive button base pair is expected to be below AA"
    );
    assert!(
        base.destructive_rendered_contrast(true, base.background) < 4.5,
        "the upstream light destructive button hover pair is expected to be below AA"
    );
    for surface in [theme.background, theme.card] {
        for hovered in [false, true] {
            assert!(
                theme.destructive_rendered_contrast(hovered, surface) >= 4.5,
                "high-contrast destructive pair failed on {surface:?} (hovered={hovered})"
            );
        }
    }
    assert!(Theme::contrast_ratio(theme.destructive, theme.background) >= 4.5);
    assert!(
        Theme::contrast_ratio(base.muted_foreground, base.muted) < 4.5,
        "the upstream muted foreground is expected to be below AA"
    );
    assert!(Theme::contrast_ratio(theme.muted_foreground, theme.muted) >= 4.5);
    assert!(Theme::contrast_ratio(theme.muted_foreground, theme.background) >= 4.5);
    assert!(Theme::contrast_ratio(theme.muted_foreground, theme.card) >= 4.5);
    assert_eq!(theme.typography.caption.color, theme.muted_foreground);
    // Unrelated tokens and geometry are preserved and the preset is idempotent.
    assert_eq!(theme.background, base.background);
    assert_eq!(theme.primary, base.primary);
    assert_eq!(theme.secondary, base.secondary);
    assert_eq!(theme.sizes, base.sizes);
    assert_eq!(theme.typography.body, base.typography.body);
    assert_eq!(theme.clone().high_contrast(), theme);
}

#[test]
fn high_contrast_dark_meets_the_rendered_destructive_pairs() {
    let base = Theme::dark();
    let theme = base.clone().high_contrast();
    // On the dark page background the base pair passes upstream, but the card
    // and hovered variants do not; all rendered variants must pass afterwards.
    assert!(
        base.destructive_rendered_contrast(true, base.background) < 4.5
            || base.destructive_rendered_contrast(true, base.card) < 4.5,
        "the upstream dark destructive hover pair is expected to be below AA"
    );
    for surface in [theme.background, theme.card] {
        for hovered in [false, true] {
            assert!(
                theme.destructive_rendered_contrast(hovered, surface) >= 4.5,
                "high-contrast destructive pair failed on {surface:?} (hovered={hovered})"
            );
        }
    }
    assert!(Theme::contrast_ratio(theme.destructive, theme.background) >= 4.5);
    assert!(Theme::contrast_ratio(theme.muted_foreground, theme.background) >= 4.5);
    assert_eq!(theme.background, base.background);
    assert_eq!(theme.foreground, base.foreground);
    assert_eq!(theme.clone().high_contrast(), theme);
}

#[test]
fn touch_and_high_contrast_compose() {
    let theme = Theme::light().touch().high_contrast();
    assert!(theme.touch);
    assert_eq!(theme.sizes, Sizes::touch());
    assert!(
        theme.destructive_rendered_contrast(false, theme.background) >= 4.5,
        "touch and high contrast must compose"
    );
    assert_eq!(
        theme.typography.caption.size,
        Typography::touch(theme.foreground, theme.muted_foreground)
            .caption
            .size
    );
}

#[test]
fn palette_switch_preserves_font_rasterization_options() {
    let ctx = egui::Context::default();
    let options = ctx.global_style().visuals.text_options;
    for theme in [
        Theme::light(),
        Theme::dark(),
        Theme::mint_light(),
        Theme::mint_dark(),
    ] {
        theme.apply(&ctx);
        assert_eq!(
            ctx.global_style().visuals.text_options,
            options,
            "palette changes must not invalidate the font atlas"
        );
    }
}
