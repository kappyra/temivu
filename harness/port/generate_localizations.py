#!/usr/bin/env python3
"""Generate a Rust lookup table from the checked-in localization IR."""
import argparse
import json
from pathlib import Path
import re
import sys
import unicodedata

from extract_localizations import KEYS, PLACEHOLDERS

ROOT = Path(__file__).resolve().parents[2]
INPUT = ROOT / 'harness/port/localizations.json'
OUTPUT = ROOT / 'src/generated_localizations.rs'

# The existing four fields keep their short names for API stability; the new
# fields are named after their ARB message without the redundant suffix.
FIELDS = {
    'paginationPreviousSemanticsLabel': 'previous',
    'paginationNextSemanticsLabel': 'next',
    'paginationPageSemanticsLabel': 'page',
    'progressSemanticsLabel': 'progress',
    'barrierLabel': 'barrier',
    'barrierOnTapHint': 'barrier_on_tap_hint',
    'bottomNavigationBarTabSemanticsLabel': 'bottom_navigation_bar_tab',
    'breadcrumbCollapsedSemanticsLabel': 'breadcrumb_collapsed',
    'contextMenuSemanticsLabel': 'context_menu',
    'dialogSemanticsLabel': 'dialog',
    'headerBackSemanticsLabel': 'header_back',
    'headerCloseSemanticsLabel': 'header_close',
    'popoverSemanticsLabel': 'popover',
    'sheetSemanticsLabel': 'sheet',
    'calendarNextMonthSemanticsLabel': 'calendar_next_month',
    'calendarNextYearSemanticsLabel': 'calendar_next_year',
    'calendarNextYearsSemanticsLabel': 'calendar_next_years',
    'calendarPreviousMonthSemanticsLabel': 'calendar_previous_month',
    'calendarPreviousYearSemanticsLabel': 'calendar_previous_year',
    'calendarPreviousYearsSemanticsLabel': 'calendar_previous_years',
    'calendarTodaySemanticsLabel': 'calendar_today',
    'selectHint': 'select_hint',
    'selectSearchHint': 'select_search_hint',
    'selectNoResults': 'select_no_results',
    'autocompleteNoResults': 'autocomplete_no_results',
    'autocompleteSuggestion': 'autocomplete_suggestion',
    'multiSelectHint': 'multi_select_hint',
    'dateFieldHint': 'date_field_hint',
    'dateFieldInvalidDateError': 'date_field_invalid_date_error',
    'timeFieldHint': 'time_field_hint',
    'passwordFieldLabel': 'password_field',
    'passwordFieldObscureTextButtonSemanticsLabel': 'password_obscure_text_button',
    'textFieldClearButtonSemanticsLabel': 'text_field_clear_button',
    'textFieldEmailLabel': 'text_field_email',
    'sliderSwitchThumbsSemanticsHint': 'slider_switch_thumbs_hint',
    'sliderMinSemanticsValue': 'slider_min_value',
    'sliderMaxSemanticsValue': 'slider_max_value',
}


def needs_escape(c):
    """Whether a character must be written as a Rust escape.

    Control characters and invisible Unicode characters are escaped so the
    generated file is readable and passes Clippy's `invisible_characters` lint;
    the Khmer translations, for example, separate words with U+200B.
    """
    code = ord(c)
    if code < 32 or code == 127:
        return True
    category = unicodedata.category(c)
    if category in ('Cc', 'Cf', 'Cs', 'Co', 'Cn', 'Zl', 'Zp'):
        return True
    return category == 'Zs' and c != ' '


def rust_string(value):
    # JSON's simple escapes are also valid Rust. All other characters stay
    # UTF-8, except control and invisible code points, which use Rust's brace
    # escape form.
    result = '"'
    for c in value:
        if c == '"': result += '\\"'
        elif c == '\\': result += '\\\\'
        elif c == '\n': result += '\\n'
        elif c == '\r': result += '\\r'
        elif c == '\t': result += '\\t'
        elif needs_escape(c): result += f'\\u{{{ord(c):x}}}'
        else: result += c
    return result + '"'


def generate(data):
    if data.get('schema') != 1:
        raise ValueError('unsupported IR schema')
    if set(FIELDS) != set(KEYS) or len(FIELDS) != len(KEYS):
        raise ValueError('generator fields do not match the extractor keys')
    lines = ['//! Generated from Forui ARB messages; see THIRD_PARTY_NOTICES.md.',
             '//! Regenerate with `harness/port/generate_localizations.py`.',
             'use crate::localization::Messages;', '',
             '/// The validated English fallback used when a locale is missing or unknown.',
             'pub(crate) fn english() -> Messages {',
             '    Messages {']
    locales = data['locales']
    english = locales.get('en')
    if not english:
        raise ValueError('English fallback is missing')
    for key in KEYS:
        if key not in english:
            raise ValueError(f'missing English fallback value for {key}')
        lines.append(f'        {FIELDS[key]}: {rust_string(english[key])},')
    lines += ['    }', '}', '',
              'pub(super) fn lookup(locale: &str) -> Option<Messages> {',
              '    match locale {']
    for locale in sorted(locales):
        if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', locale):
            raise ValueError('invalid locale')
        # Match Forui generated fallback: regional locale -> language -> English.
        merged = dict(locales['en'])
        parts = locale.split('-')
        for length in range(1, len(parts) + 1):
            merged.update(locales.get('-'.join(parts[:length]), {}))
        lines.append(f'        {rust_string(locale)} => Some(Messages {{')
        for key in KEYS:
            if key not in merged:
                raise ValueError(f'missing fallback value for {key} in {locale}')
            lines.append(f'            {FIELDS[key]}: {rust_string(merged[key])},')
        lines.append('        }),')
    lines += ['        _ => None,', '    }', '}', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    output = generate(json.loads(INPUT.read_text(encoding='utf-8')))
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding='utf-8') != output:
            print('generated localizations differ from IR', file=sys.stderr)
            return 1
    else:
        OUTPUT.write_text(output, encoding='utf-8')
    return 0


if __name__ == '__main__':
    sys.exit(main())
