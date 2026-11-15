package pl.edu.agh.student.mobiletuner

/** UI value shared by built-in tunings and any future custom tuning. */
data class TuningPreset(val name: String, val notes: List<Int>)

/** String order follows the instrument, not ascending pitch (ukulele uses re-entrant G4). */
enum class Preset(val tuning: TuningPreset) {
    eStandard(TuningPreset("E standard", listOf(40, 45, 50, 55, 59, 64))),
    halfStepDown(TuningPreset("Half step down", listOf(39, 44, 49, 54, 58, 63))),
    fullStepDown(TuningPreset("Full step down", listOf(38, 43, 48, 53, 57, 62))),
    dropD(TuningPreset("Drop D", listOf(38, 45, 50, 55, 59, 64))),
    dropC(TuningPreset("Drop C", listOf(36, 43, 48, 53, 57, 62))),
    dropB(TuningPreset("Drop B", listOf(35, 42, 47, 52, 56, 61))),
    openG(TuningPreset("Open G", listOf(38, 43, 50, 55, 59, 62))),
    openD(TuningPreset("Open D", listOf(38, 45, 50, 54, 57, 62))),
    openE(TuningPreset("Open E", listOf(40, 47, 52, 56, 59, 64))),
    openC(TuningPreset("Open C", listOf(36, 43, 48, 55, 60, 64))),
    dadgad(TuningPreset("DADGAD", listOf(38, 45, 50, 55, 57, 62))),
    bass4String(TuningPreset("Bass 4 string", listOf(28, 33, 38, 43))),
    // Re-entrant G4 belongs first; string order is not ascending pitch.
    ukulele(TuningPreset("Ukulele", listOf(67, 60, 64, 69))),
    violin(TuningPreset("Violin", listOf(55, 62, 69, 76))),
}
