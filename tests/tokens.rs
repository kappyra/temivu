use temivu::{Breakpoints, Sizes, ViewportClass};

#[test]
fn size_tokens_scale_and_interpolate() {
    let desktop = Sizes::desktop();
    let touch = Sizes::touch();
    assert_eq!(
        (
            desktop.field.xs,
            desktop.field.md,
            desktop.item,
            desktop.tile,
            desktop.calendar
        ),
        (24.0, 36.0, 30.0, 48.0, 32.0)
    );
    assert_eq!(
        (
            touch.field.xs,
            touch.field.md,
            touch.item,
            touch.tile,
            touch.calendar
        ),
        (32.0, 44.0, 44.0, 48.0, 44.0)
    );
    assert_eq!(desktop.scale(2.0).calendar, 64.0);
    assert_eq!(desktop.lerp(touch, 0.5).item, 37.0);
    assert_eq!(desktop.lerp(touch, 1.0), touch);
}

#[test]
fn responsive_boundaries_are_inclusive_and_invalid_values_rejected() {
    let bp = Breakpoints::default();
    for (width, expected) in [
        (0.0, ViewportClass::BelowSmall),
        (639.9, ViewportClass::BelowSmall),
        (640.0, ViewportClass::Small),
        (768.0, ViewportClass::Medium),
        (1024.0, ViewportClass::Large),
        (1280.0, ViewportClass::ExtraLarge),
        (1536.0, ViewportClass::ExtraExtraLarge),
    ] {
        assert_eq!(bp.classify(width), Ok(expected));
    }
    assert!(bp.classify(f32::NAN).is_err());
    assert!(bp.classify(-1.0).is_err());
    assert!(Breakpoints { sm: 900.0, ..bp }.classify(1000.0).is_err());
}
