import Foundation

/// Fixed log-freq grid of [POINTS] points over the band. Refer to docs/SPECTRUM_GRID.md
nonisolated struct SpectrumGrid: Sendable {
    static let points = 400
    static let dbMin: Float = -100
    static let dbMax: Float = 0
    private static let placeholderDb: Float = -20
    
    let minHz: Float
    let maxHz: Float
    private let first: [Int]
    private let last: [Int]
    private let pos: [Float]
    private let lo: Int
    private let hi: Int
    
    init(sampleRate: Float, fftSize: Int, minHz: Float, maxHz: Float) {
        self.minHz = minHz
        self.maxHz = maxHz
        let hzPerBin = sampleRate / Float(fftSize)
        let hz = { (i: Float) in Self.hz(at: i, minHz: minHz, maxHz: maxHz) }
        let lo = Int((minHz / hzPerBin).rounded(.up))
        let hi = min(Int((maxHz / hzPerBin).rounded(.down)), fftSize / 2 - 1)
        self.lo = lo
        self.hi = hi
        first = (0..<Self.points).map { max(Int((hz(Float($0) - 0.5) / hzPerBin).rounded(.up)), lo) }
        last = (0..<Self.points).map { min(Int((hz(Float($0) + 0.5) / hzPerBin).rounded(.down)), hi) }
        pos = (0..<Self.points).map { min(max(hz(Float($0)) / hzPerBin, Float(lo)), Float(hi)) }
    }
    
    static func hz(at i: Float, minHz: Float, maxHz: Float) -> Float {
        minHz * pow(maxHz / minHz, i / Float(points - 1))
    }
    
    /// horizontal position - normalized
    func x(ofHz hz: Float) -> Float { log2(hz / minHz) / log2(maxHz/minHz)}
    
    func downsample(_ src: UnsafeRawBufferPointer, into out: UnsafeMutableBufferPointer<Float>) {
        func bin(_ k: Int) -> Float {
            src.loadUnaligned(fromByteOffset: min(max(k, lo), hi) * 4, as: Float.self)
        }
        
        for i in 0..<Self.points {
            let db: Float
            if last[i] >= first[i] {
                var m = bin(first[i])
                for k in first[i] ..< last[i] + 1 { m = max(m, bin(k)) }
                db = m
            } else {
                let k = Int(pos[i])
                let t = pos[i] - Float(k)
                let p0 = bin(k - 1), p1 = bin(k), p2 = bin(k+1), p3 = bin(k+2)
                db = p1 + 0.5 * t * (p2 - p0 + t * (2 * p0 - 5 * p1 + 4 * p2 - p3 + t * (3 * (p1 - p2) + p3 - p0)))
            }
            out[i] = Self.normalise(db)
        }
    }
    
    ///TODO: REMOVE - placeholder until target_spectrum
    func placeholderGhost(midi: Int32, into out: UnsafeMutableBufferPointer<Float>) {
        let f0 = 440 * pow(2, Float(midi - 69) / 12)
        for i in 0..<Self.points {
            let f = Self.hz(at: Float(i), minHz: minHz, maxHz: maxHz)
            var db = Self.dbMin
            var n: Float = 1
            while n * f0 <= maxHz {
                let semis = 12 * log2(f / (n * f0))
                db = max(db, Self.placeholderDb - 20 * log10(n) - 40 * semis * semis)
                n += 1
            }
            out[i] = Self.normalise(db)
        }
    }
    
    private static func normalise(_ db: Float) -> Float {
        min(max((db - dbMin) / (dbMax - dbMin), 0), 1)
    }
 
}
