use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use temivu::{Messages, PaginationState, Theme, pagination_localized};

#[test]
fn locale_lookup_uses_regions_and_falls_back() {
    let french = Messages::for_locale("fr-CA");
    assert_eq!(french.previous, "Précédent");
    assert_eq!(
        Messages::for_locale("fr_unknown"),
        Messages::for_locale("fr")
    );
    assert_eq!(Messages::for_locale("unknown"), Messages::for_locale("en"));
    assert_eq!(Messages::for_locale("ar").page_label(12), "الصفحة 12");
    assert_eq!(Messages::for_locale("en").progress, "Loading");
    assert_eq!(Messages::for_locale("zh-Hant-TW").page_label(5), "第 5 頁");
    assert_eq!(
        Messages::for_locale("zh-Hant-HK"),
        Messages::for_locale("zh-HK")
    );
    assert_eq!(
        Messages::for_locale("zh-Hant"),
        Messages::for_locale("zh-TW")
    );
    assert_eq!(
        Messages::for_locale("zh-Hans-TW"),
        Messages::for_locale("zh")
    );
    assert_eq!(
        Messages::for_locale("sr-Latn-RS"),
        Messages::for_locale("sr-Latn")
    );
}

#[test]
fn unsupported_locales_and_subtags_fall_back_to_english() {
    let english = Messages::for_locale("en");
    assert_eq!(Messages::for_locale("xx"), english);
    assert_eq!(Messages::for_locale("xx-YY"), english);
    assert_eq!(Messages::for_locale(""), english);
    // Locale tags are case-insensitive and accept underscores.
    assert_eq!(Messages::for_locale("DE_de"), Messages::for_locale("de"));
    assert_eq!(Messages::for_locale("ZH-HANS"), Messages::for_locale("zh"));
    // Regional locales inherit their language before falling back to English.
    assert_eq!(
        Messages::for_locale("de-CH").barrier,
        Messages::for_locale("de").barrier
    );
    // Several ARB files omit these labels at the pinned revision; the generated
    // table then keeps the English string rather than dropping the message.
    assert_eq!(Messages::for_locale("ar").previous, english.previous);
    assert_eq!(
        Messages::for_locale("ar").select_no_results,
        english.select_no_results
    );
    assert_eq!(Messages::for_locale("ja").dialog, english.dialog);
}

#[test]
fn new_messages_cover_several_scripts_and_locales() {
    let german = Messages::for_locale("de");
    assert_eq!(german.barrier, "Gitter");
    assert_eq!(german.dialog, "Dialogfeld");
    assert_eq!(german.header_close, "Schließen");
    assert_eq!(german.calendar_next_month, "Nächster Monat");
    assert_eq!(german.password_field, "Passwort");
    assert_eq!(german.text_field_email, "E-Mail");
    assert_eq!(german.text_field_clear_button, "Löschen");

    let japanese = Messages::for_locale("ja");
    assert_eq!(japanese.progress, "読み込み中");
    assert_eq!(japanese.select_hint, "アイテムを選択");
    assert_eq!(japanese.header_close, "閉じる");
    assert_eq!(japanese.calendar_next_month, "次の月");
    assert_eq!(japanese.password_field, "パスワード");

    let arabic = Messages::for_locale("ar");
    assert_eq!(arabic.barrier, "تمويه");
    assert_eq!(arabic.dialog, "مربع حوار");
    assert_eq!(arabic.calendar_next_month, "الشهر التالي");
    assert_eq!(arabic.password_field, "كلمة المرور");
    assert_eq!(arabic.text_field_email, "البريد الإلكتروني");

    let hindi = Messages::for_locale("hi");
    assert_eq!(hindi.progress, "लोड हो रहा है");
    assert_eq!(hindi.header_close, "बंद करें");
    assert_eq!(hindi.password_field, "पासवर्ड");

    // Traditional and simplified Chinese keep distinct translations.
    assert_eq!(Messages::for_locale("zh-TW").dialog, "對話方塊");
    assert_eq!(Messages::for_locale("zh").dialog, "对话框");
    assert_eq!(Messages::for_locale("zh-TW").header_close, "關閉");
    assert_eq!(Messages::for_locale("zh").header_close, "关闭");
    assert_eq!(Messages::for_locale("ru").select_hint, "Выберите элемент");
}

#[test]
fn templates_format_reordered_placeholders_and_leave_no_braces() {
    let english = Messages::for_locale("en");
    assert_eq!(english.page_label(7), "Page 7");
    assert_eq!(english.tab_position(2, 5), "Tab 2 of 5");
    assert_eq!(english.barrier_hint("Sheet"), "Close Sheet");
    assert_eq!(
        english.autocomplete_suggestion("Apple"),
        "Suggestion: Apple"
    );
    assert_eq!(english.slider_min_value("20%"), "Minimum, 20%");
    assert_eq!(english.slider_max_value("80%"), "Maximum, 80%");

    // Translations may reorder placeholders; helpers interpolate by name.
    let japanese = Messages::for_locale("ja");
    assert_eq!(japanese.tab_position(2, 5), "5 個のタブのうち 2 個目");
    assert_eq!(japanese.page_label(3), "3 ページ");
    let hindi = Messages::for_locale("hi");
    assert_eq!(hindi.tab_position(2, 5), "5 में से 2 टैब");
    let chinese = Messages::for_locale("zh");
    assert_eq!(chinese.tab_position(2, 5), "第 2 个标签，共 5 个");
    assert_eq!(chinese.autocomplete_suggestion("苹果"), "建议：苹果");

    let german = Messages::for_locale("de");
    assert_eq!(german.page_label(9), "Seite 9");
    assert_eq!(german.tab_position(3, 4), "Tab 3 von 4");
    assert_eq!(german.autocomplete_suggestion("Apfel"), "Vorschlag: Apfel");
    assert_eq!(german.slider_min_value("20%"), "Minimum, 20%");
    for formatted in [
        english.autocomplete_suggestion("x"),
        japanese.tab_position(1, 2),
        chinese.slider_max_value("y"),
        german.barrier_hint("z"),
    ] {
        assert!(!formatted.contains('{'), "unfilled template: {formatted}");
        assert!(!formatted.contains('}'), "unfilled template: {formatted}");
    }
}

#[test]
fn barrier_on_tap_hint_uses_the_validated_english_template() {
    // Every non-English upstream `barrierOnTapHint` uses Dart interpolation
    // (`$modalRouteContentName`) rather than an ARB placeholder, so the
    // extractor keeps only English and other locales fall back to it.
    let english = Messages::for_locale("en");
    assert_eq!(english.barrier_on_tap_hint, "Close {modalRouteContentName}");
    for locale in ["de", "fr", "ja", "ar", "zh-TW"] {
        assert_eq!(
            Messages::for_locale(locale).barrier_on_tap_hint,
            english.barrier_on_tap_hint,
            "unexpected template for {locale}"
        );
        assert_eq!(
            Messages::for_locale(locale).barrier_hint("Sheet"),
            "Close Sheet"
        );
    }
}

#[test]
fn messages_are_copy_and_compare_by_value() {
    let first = Messages::for_locale("fr");
    let second = first;
    assert_eq!(first, second);
    assert_ne!(first, Messages::for_locale("de"));
    assert_eq!(first.progress, "Chargement");
}

#[test]
fn translated_pagination_exposes_localized_page_names() {
    let messages = Messages::for_locale("fr");
    let mut harness = Harness::builder().build_ui_state(
        |ui, state: &mut PaginationState| {
            pagination_localized(ui, state, &Theme::light(), &messages)
        },
        PaginationState::new(3, 0, 1, true).unwrap(),
    );
    harness.run();
    assert!(
        harness
            .get_by_label("Précédent")
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label("Page 2").click();
    harness.run();
    assert_eq!(harness.state().page(), 1);
    assert_eq!(
        harness
            .get_by_label("Page 2")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}
