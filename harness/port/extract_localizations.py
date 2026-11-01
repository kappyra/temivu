#!/usr/bin/env python3
"""Extract a narrow, validated set of Forui ARB messages into a source-hashed IR.

Only two message shapes are supported:

* plain strings, containing no ICU syntax; and
* a simple ``{placeholder}`` template, where every placeholder is a bare
  identifier and every expected placeholder occurs exactly once.

ICU plural, select and ordinal messages (for example ``autocompleteResults``
and ``selectResults``) are deliberately rejected instead of being silently
mangled: the Rust port renders plain strings and has no plural-category,
number-formatting or date-formatting support. Rejected messages stay in the
upstream ARB files and are recorded as gaps in the localization module docs.
The extractor fails on an unsupported pattern rather than dropping it.
Messages listed in ``ENGLISH_ONLY`` keep only their validated English value
because the pinned translations are not ARB-template-shaped.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

KEYS = (
    # Pagination and progress.
    'paginationPreviousSemanticsLabel',
    'paginationNextSemanticsLabel',
    'paginationPageSemanticsLabel',
    'progressSemanticsLabel',
    # Overlays, menus and navigation semantics.
    'barrierLabel',
    'barrierOnTapHint',
    'bottomNavigationBarTabSemanticsLabel',
    'breadcrumbCollapsedSemanticsLabel',
    'contextMenuSemanticsLabel',
    'dialogSemanticsLabel',
    'headerBackSemanticsLabel',
    'headerCloseSemanticsLabel',
    'popoverSemanticsLabel',
    'sheetSemanticsLabel',
    # Calendar navigation.
    'calendarNextMonthSemanticsLabel',
    'calendarNextYearSemanticsLabel',
    'calendarNextYearsSemanticsLabel',
    'calendarPreviousMonthSemanticsLabel',
    'calendarPreviousYearSemanticsLabel',
    'calendarPreviousYearsSemanticsLabel',
    'calendarTodaySemanticsLabel',
    # Select and autocomplete.
    'selectHint',
    'selectSearchHint',
    'selectNoResults',
    'autocompleteNoResults',
    'autocompleteSuggestion',
    'multiSelectHint',
    # Fields.
    'dateFieldHint',
    'dateFieldInvalidDateError',
    'timeFieldHint',
    'passwordFieldLabel',
    'passwordFieldObscureTextButtonSemanticsLabel',
    'textFieldClearButtonSemanticsLabel',
    'textFieldEmailLabel',
    # Slider.
    'sliderSwitchThumbsSemanticsHint',
    'sliderMinSemanticsValue',
    'sliderMaxSemanticsValue',
)

# Messages whose ARB value is a simple template. The tuple lists the exact
# placeholders the value must contain; ordering may differ in translations.
PLACEHOLDERS = {
    'paginationPageSemanticsLabel': ('page',),
    'barrierOnTapHint': ('modalRouteContentName',),
    'bottomNavigationBarTabSemanticsLabel': ('tabIndex', 'tabCount'),
    'autocompleteSuggestion': ('suggestion',),
    'sliderMinSemanticsValue': ('value',),
    'sliderMaxSemanticsValue': ('value',),
}

_PLACEHOLDER = re.compile(r'\{([^{}]*)\}')
_IDENTIFIER = re.compile(r'[A-Za-z_][A-Za-z0-9_]*')

# `barrierOnTapHint` is the only extracted template with malformed translations
# at the pinned revision: every non-English ARB value interpolates the name with
# Dart's `$modalRouteContentName` syntax instead of an ARB placeholder, so it is
# not a string this port can format without leaking the marker. Only the English
# template is kept; other locales fall back to English in the generated table.
ENGLISH_ONLY = frozenset({'barrierOnTapHint'})


def message_error(key, locale, value):
    """Return why ``value`` is unsupported for ``key``, or ``None`` when valid.

    Any brace that is not one of the declared placeholders is rejected, which
    also catches ICU plural/select arguments: they contain commas, whitespace
    or nested braces and therefore cannot be a bare placeholder identifier.
    """
    expected = PLACEHOLDERS.get(key, ())
    found = []
    position = 0
    for match in _PLACEHOLDER.finditer(value):
        literal = value[position:match.start()]
        if '{' in literal or '}' in literal:
            return f'unsupported ICU brace pattern in {key} ({locale}): {value!r}'
        name = match.group(1)
        if not _IDENTIFIER.fullmatch(name):
            return f'unsupported ICU plural/select argument in {key} ({locale}): {value!r}'
        found.append(name)
        position = match.end()
    literal = value[position:]
    if '{' in literal or '}' in literal:
        return f'unsupported ICU brace pattern in {key} ({locale}): {value!r}'
    if sorted(found) != sorted(expected) or len(found) != len(expected):
        return (
            f'unexpected placeholders in {key} ({locale}): '
            f'expected {expected}, found {tuple(found)}'
        )
    return None


def extract(folder):
    files = sorted(folder.glob('f_*.arb'))
    if not files:
        raise ValueError('no ARB locale files found')
    digest = hashlib.sha256()
    locales = {}
    for path in files:
        data = path.read_bytes()
        name = path.name.encode()
        digest.update(len(name).to_bytes(8, 'big'))
        digest.update(name)
        digest.update(hashlib.sha256(data).digest())
        content = json.loads(data)
        name = path.stem[2:].replace('_', '-').lower()
        if name in locales:
            raise ValueError('duplicate normalized locale')
        selected = {}
        for key in KEYS:
            if key not in content:
                continue
            if name != 'en' and key in ENGLISH_ONLY:
                # The translation is not ARB-template-shaped; the generated
                # table falls back to the validated English message instead.
                continue
            value = content[key]
            if not isinstance(value, str):
                raise ValueError(f'expected string message for {key} in {name}')
            error = message_error(key, name, value)
            if error is not None:
                raise ValueError(error)
            selected[key] = value
        locales[name] = selected
    if set(locales.get('en', {})) != set(KEYS):
        raise ValueError('English fallback is incomplete')
    return {'schema': 1, 'source_sha256': digest.hexdigest(), 'locales': locales}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('folder', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    output = extract(args.folder)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
