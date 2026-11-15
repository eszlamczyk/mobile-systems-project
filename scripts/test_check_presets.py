"""Behavior checks for the preset CI guard, using the actual app declarations."""

import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).with_name('check-presets.py')
SPEC = importlib.util.spec_from_file_location('check_presets', SCRIPT)
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)
ROOT = SCRIPT.parent.parent
KOTLIN = (ROOT / 'android/app/src/main/java/pl/edu/agh/student/mobiletuner/Preset.kt').read_text(encoding='utf-8')
SWIFT = (ROOT / 'ios/MobileTuner/Preset.swift').read_text(encoding='utf-8')


class PresetSyncTests(unittest.TestCase):
    def test_actual_presets_match(self):
        self.assertEqual(checker.check(KOTLIN, SWIFT), 14)

    def test_note_name_case_and_string_order_drift_fail(self):
        for before, after in [
            ('[67, 60, 64, 69]', '[55, 60, 64, 69]'),
            ('[67, 60, 64, 69]', '[60, 67, 64, 69]'),
            ('"Drop B"', '"Drop H"'),
            ('ukulele', 'ukuleleLowG'),
        ]:
            with self.subTest(after=after), self.assertRaises(ValueError):
                checker.check(KOTLIN, SWIFT.replace(before, after))

    def test_missing_mapping_and_extra_case_fail(self):
        changed = SWIFT.replace('case .violin: return TuningPreset(name: "Violin", notes: [55, 62, 69, 76])', '')
        with self.assertRaisesRegex(ValueError, 'declared cases'):
            checker.check(KOTLIN, changed)
        with self.assertRaises(ValueError):
            checker.check(KOTLIN, SWIFT.replace('case eStandard,', 'case extra, eStandard,'))

    def test_unsupported_or_empty_declarations_fail(self):
        for kotlin in ['', KOTLIN.replace('listOf(40,', 'mutableListOf(40,')]:
            with self.subTest(kotlin=kotlin[:20]), self.assertRaises(ValueError):
                checker.check(kotlin, SWIFT)

    def test_duplicate_and_invalid_midi_fail_even_if_both_match(self):
        for kotlin, swift in [
            (KOTLIN.replace('dropB(', 'dropC('), SWIFT.replace('dropB', 'dropC')),
            (KOTLIN.replace('67, 60, 64, 69', '128, 60, 64, 69'), SWIFT.replace('67, 60, 64, 69', '128, 60, 64, 69')),
        ]:
            with self.subTest(kotlin=kotlin[-100:]), self.assertRaises(ValueError):
                checker.check(kotlin, swift)

    def test_multiline_note_lists_are_supported(self):
        self.assertEqual(checker.check(KOTLIN.replace('40, 45,', '40,\n45,'), SWIFT.replace('40, 45,', '40,\n45,')), 14)


if __name__ == '__main__':
    unittest.main()
