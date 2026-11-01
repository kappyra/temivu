//! Locale messages for the migrated Forui widgets.
//!
//! [`Messages::for_locale`] resolves a BCP 47 locale against the 113 Forui
//! locales checked into `forui/lib/l10n` at the pinned revision, falling back
//! through locale subtags to English. English also fills in translations that
//! are missing from a locale's ARB file at the pin (for example
//! `selectNoResults`).
//!
//! # Upstream coverage and gaps
//!
//! Plain strings and simple `{placeholder}` templates are supported. The
//! following upstream message shapes remain unimplemented:
//!
//! * ICU plural and select messages (`autocompleteResults`, `selectResults`,
//!   `calendarTodaySemanticsLabel` variants, and so on): this port renders text
//!   through egui without plural categories or locale-aware number formatting,
//!   so the extractor rejects those messages instead of dropping alternatives.
//! * Dates, times, numbers, currency and bidi shaping: strings are rendered
//!   verbatim by egui; callers format values themselves before passing them to
//!   the template helpers.
//! * `barrierOnTapHint`: every non-English upstream value uses Dart's
//!   `$modalRouteContentName` interpolation instead of an ARB placeholder, so
//!   only the English template is retained and other locales fall back to it.
//!
//! The remainder of Forui's localization surface (widget-specific hints and
//! accessibility actions outside [`Messages`]) is not ported yet.

/// Localized semantic messages, generated from the pinned Forui ARB files.
///
/// The generated table is source-hashed and validated by
/// `harness/port/verify_sources.py`; regenerate it with
/// `harness/port/extract_localizations.py` and
/// `harness/port/generate_localizations.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Messages {
    /// `paginationPreviousSemanticsLabel`: label for the previous-page button.
    pub previous: &'static str,
    /// `paginationNextSemanticsLabel`: label for the next-page button.
    pub next: &'static str,
    /// `paginationPageSemanticsLabel`: page button label template (`{page}`).
    pub page: &'static str,
    /// `progressSemanticsLabel`: default semantics label for progress indicators.
    pub progress: &'static str,
    /// `barrierLabel`: label for the barrier under a bottom sheet or modal.
    pub barrier: &'static str,
    /// `barrierOnTapHint`: dismiss hint template (`{modalRouteContentName}`).
    ///
    /// Only the English template is available at the pinned revision; every
    /// other locale falls back to it.
    pub barrier_on_tap_hint: &'static str,
    /// `bottomNavigationBarTabSemanticsLabel`: tab position template
    /// (`{tabIndex} of {tabCount}`).
    pub bottom_navigation_bar_tab: &'static str,
    /// `breadcrumbCollapsedSemanticsLabel`: label for the collapsed breadcrumb
    /// trigger that reveals hidden items.
    pub breadcrumb_collapsed: &'static str,
    /// `contextMenuSemanticsLabel`: semantic label for a context menu.
    pub context_menu: &'static str,
    /// `dialogSemanticsLabel`: semantic label for a modal dialog.
    pub dialog: &'static str,
    /// `headerBackSemanticsLabel`: default label for a header's back button.
    pub header_back: &'static str,
    /// `headerCloseSemanticsLabel`: default label for a header's close button.
    pub header_close: &'static str,
    /// `popoverSemanticsLabel`: semantic label for a popover.
    pub popover: &'static str,
    /// `sheetSemanticsLabel`: semantic label for a bottom sheet.
    pub sheet: &'static str,
    /// `calendarNextMonthSemanticsLabel`: label for the next-month button.
    pub calendar_next_month: &'static str,
    /// `calendarNextYearSemanticsLabel`: label for the next-year button.
    pub calendar_next_year: &'static str,
    /// `calendarNextYearsSemanticsLabel`: label for the next-years button.
    pub calendar_next_years: &'static str,
    /// `calendarPreviousMonthSemanticsLabel`: label for the previous-month
    /// button.
    pub calendar_previous_month: &'static str,
    /// `calendarPreviousYearSemanticsLabel`: label for the previous-year
    /// button.
    pub calendar_previous_year: &'static str,
    /// `calendarPreviousYearsSemanticsLabel`: label for the previous-years
    /// button.
    pub calendar_previous_years: &'static str,
    /// `calendarTodaySemanticsLabel`: suffix for a calendar cell that is today.
    pub calendar_today: &'static str,
    /// `selectHint`: hint text for a single-select field.
    pub select_hint: &'static str,
    /// `selectSearchHint`: hint text for a select field's search input.
    pub select_search_hint: &'static str,
    /// `selectNoResults`: message shown when a select search matches nothing.
    pub select_no_results: &'static str,
    /// `autocompleteNoResults`: message shown when autocomplete matches nothing.
    pub autocomplete_no_results: &'static str,
    /// `autocompleteSuggestion`: inline suggestion template (`{suggestion}`).
    pub autocomplete_suggestion: &'static str,
    /// `multiSelectHint`: hint text for a multi-select field.
    pub multi_select_hint: &'static str,
    /// `dateFieldHint`: hint text for a picker-only date field.
    pub date_field_hint: &'static str,
    /// `dateFieldInvalidDateError`: error shown for an unparsable date.
    pub date_field_invalid_date_error: &'static str,
    /// `timeFieldHint`: hint text for a picker-only time field.
    pub time_field_hint: &'static str,
    /// `passwordFieldLabel`: label for a password field.
    pub password_field: &'static str,
    /// `passwordFieldObscureTextButtonSemanticsLabel`: label for the button
    /// that hides an obscured password.
    pub password_obscure_text_button: &'static str,
    /// `textFieldClearButtonSemanticsLabel`: label for a text field's clear
    /// button.
    pub text_field_clear_button: &'static str,
    /// `textFieldEmailLabel`: label for an email text field.
    pub text_field_email: &'static str,
    /// `sliderSwitchThumbsSemanticsHint`: hint that a range slider's track
    /// switches thumbs on double tap.
    pub slider_switch_thumbs_hint: &'static str,
    /// `sliderMinSemanticsValue`: range slider minimum value template
    /// (`{value}`).
    pub slider_min_value: &'static str,
    /// `sliderMaxSemanticsValue`: range slider maximum value template
    /// (`{value}`).
    pub slider_max_value: &'static str,
}

impl Messages {
    /// Choose a generated locale, falling back through locale subtags to English.
    /// Locale lookup is best performed when the application locale changes.
    pub fn for_locale(locale: &str) -> Self {
        let normalized = locale.replace('_', "-").to_ascii_lowercase();
        // Forui's Chinese ARB data uses region-specific traditional forms, but
        // BCP 47 callers often supply a script before the region.
        let mut parts = normalized.split('-');
        if parts.next() == Some("zh") {
            match parts.next() {
                Some("hant") => {
                    let traditional = match parts.next() {
                        Some("hk") => "zh-hk",
                        _ => "zh-tw",
                    };
                    return crate::generated_localizations::lookup(traditional)
                        .unwrap_or_else(crate::generated_localizations::english);
                }
                Some("hans") => {
                    return crate::generated_localizations::lookup("zh")
                        .unwrap_or_else(crate::generated_localizations::english);
                }
                _ => {}
            }
        }
        let mut candidate = normalized.as_str();
        loop {
            if let Some(messages) = crate::generated_localizations::lookup(candidate) {
                return messages;
            }
            match candidate.rsplit_once('-') {
                Some((parent, _)) => candidate = parent,
                None => return crate::generated_localizations::english(),
            }
        }
    }

    /// Format a one-based page label using the upstream message template.
    pub fn page_label(&self, page: usize) -> String {
        fill(self.page, &[("page", &page.to_string())])
    }

    /// Format the `"Tab {tabIndex} of {tabCount}"` position label.
    ///
    /// Both values are one-based, matching Forui's
    /// `bottomNavigationBarTabSemanticsLabel(index + 1, length)` call. Convert a
    /// zero-based position by adding one before calling this helper.
    pub fn tab_position(&self, tab_index: usize, tab_count: usize) -> String {
        fill(
            self.bottom_navigation_bar_tab,
            &[
                ("tabIndex", &tab_index.to_string()),
                ("tabCount", &tab_count.to_string()),
            ],
        )
    }

    /// Format the barrier dismiss hint for the named modal content.
    pub fn barrier_hint(&self, content_name: &str) -> String {
        fill(
            self.barrier_on_tap_hint,
            &[("modalRouteContentName", content_name)],
        )
    }

    /// Format the inline autocomplete suggestion hint.
    pub fn autocomplete_suggestion(&self, suggestion: &str) -> String {
        fill(self.autocomplete_suggestion, &[("suggestion", suggestion)])
    }

    /// Format the range slider's minimum-thumb value announcement.
    pub fn slider_min_value(&self, value: &str) -> String {
        fill(self.slider_min_value, &[("value", value)])
    }

    /// Format the range slider's maximum-thumb value announcement.
    pub fn slider_max_value(&self, value: &str) -> String {
        fill(self.slider_max_value, &[("value", value)])
    }
}

/// Replace every `{placeholder}` in a validated upstream template.
///
/// The generated table only contains templates accepted by the extractor, so
/// every expected placeholder occurs exactly once; unknown placeholders are
/// left untouched for forward compatibility instead of panicking.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut result = template.to_owned();
    for (placeholder, value) in values {
        let pattern = format!("{{{placeholder}}}");
        result = result.replace(&pattern, value);
    }
    result
}
