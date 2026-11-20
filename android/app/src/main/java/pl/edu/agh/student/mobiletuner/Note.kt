package pl.edu.agh.student.mobiletuner

import kotlin.math.pow

object Note {
    val names = listOf("C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B")

    fun name(midi: Int): String = names[((midi % 12) + 12) % 12]

    fun octave(midi: Int): Int = midi / 12 - 1

    fun fullName(midi: Int): String = "${name(midi)}${octave(midi)}"

    fun frequency(midi: Int): Double = 440.0 * 2.0.pow((midi - 69) / 12.0)
}
