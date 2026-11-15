#!/usr/bin/env python3

import json
from pathlib import Path
import re
import sys


STRING = r'"(?:[^"\\]|\\.)*"'
NOTES = r'\d+(?:\s*,\s*\d+)*\s*,?'


def extract(text, body_pattern, entry_pattern, language):
    text = re.sub(r'//[^\n]*', '', text)
    body = re.search(body_pattern, text, re.DOTALL)
    if body is None:
        raise ValueError(f'{language}: preset body not found')
    body = body.group(1)
    entries = []
    for match in re.finditer(entry_pattern, body):
        case, name, notes = match.groups()
        midi = [int(note.strip()) for note in notes.split(',') if note.strip()]
        if any(not 0 <= note <= 127 for note in midi):
            raise ValueError(f'{language}: MIDI note out of range in {case}')
        entries.append((case, json.loads(name), midi))
    remainder = re.sub(entry_pattern, '', body)
    if not entries or remainder.strip():
        raise ValueError(f'{language}: unsupported or empty preset declarations')
    cases = [entry[0] for entry in entries]
    if len(cases) != len(set(cases)):
        raise ValueError(f'{language}: duplicate preset case')
    return entries


def check(kotlin, swift):
    kotlin_entries = extract(
        kotlin, r'enum class Preset\b[^{]*\{(.*?)\}',
        rf'\s*(\w+)\(\s*TuningPreset\(\s*({STRING})\s*,\s*listOf\(\s*({NOTES})\s*\)\s*\)\s*\)\s*,?',
        'Kotlin',
    )
    swift_entries = extract(
        swift, r'switch self\s*\{(.*?)\}',
        rf'\s*case\s+\.(\w+)\s*:\s*return\s+Preset\(\s*name:\s*({STRING})\s*,\s*notes:\s*\[\s*({NOTES})\s*\]\s*\)',
        'Swift',
    )
    declarations = re.search(r'enum BuiltInPreset\b[^{]*\{(.*?)\bvar tuning', swift, re.DOTALL)
    if declarations is None:
        raise ValueError('Swift: enum cases not found')
    case_groups = re.findall(r'\bcase\s+(\w+(?:\s*,\s*\w+)*)', declarations.group(1))
    cases = [case.strip() for group in case_groups for case in group.split(',')]
    if cases != [entry[0] for entry in swift_entries]:
        raise ValueError('Swift: declared cases and tuning mappings differ')
    if kotlin_entries != swift_entries:
        raise ValueError(f'Presets differ:\nKotlin: {kotlin_entries}\nSwift:  {swift_entries}')
    return len(kotlin_entries)


def main():
    root = Path(__file__).resolve().parent.parent
    try:
        kotlin = (root / 'android/app/src/main/java/pl/edu/agh/student/mobiletuner/Preset.kt').read_text(encoding='utf-8')
        swift = (root / 'ios/MobileTuner/Model/BuiltInPreset.swift').read_text(encoding='utf-8')
        count = check(kotlin, swift)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f'All {count} presets match: cases, names, string order, and MIDI notes.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
