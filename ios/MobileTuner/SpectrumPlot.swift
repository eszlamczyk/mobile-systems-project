import SwiftUI
import UIKit

/// Component for the live spectrum over the target ghost. New frames go straight to Anim layers, so SwiftUI
/// never invalidates composition or layout.
struct SpectrumPlot: UIViewRepresentable {
    let engine: TunerEngine
    
    func makeUIView(context: Context) -> SpectrumPlotView { SpectrumPlotView(engine: engine) }
    func updateUIView(_ view: SpectrumPlotView, context: Context) {}
}

final class SpectrumPlotView: UIView, PlotObserver {
    private static let c0Hz: Float = 16.3516
    
    private let engine: TunerEngine
    private let gridLayer = CAShapeLayer()
    private let ghostLayer = CAShapeLayer()
    private let liveLayer = CAShapeLayer()
    
    init(engine: TunerEngine) {
        self.engine = engine
        super.init(frame: .zero)
        for sub in [gridLayer, ghostLayer, liveLayer] {
            sub.actions = ["path": NSNull(), "bounds": NSNull(), "position": NSNull()]
            sub.lineJoin = .round
            layer.addSublayer(sub)
        }
        gridLayer.fillColor = nil
        ghostLayer.fillColor = nil
        ghostLayer.lineWidth = 1.5
        liveLayer.lineWidth = 1.5
        applyColors()
        registerForTraitChanges([UITraitUserInterfaceStyle.self]) { (view: Self, _) in
            view.applyColors()
        }
        engine.plotObserver = self
    }
    
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    
    override func layoutSubviews() {
        super.layoutSubviews()
        for sub in [gridLayer, ghostLayer, liveLayer] { sub.frame = bounds }
        gridLayer.path = gridPath()
        ghostDidChange()
        plotDidUpdate()
    }
    
    func plotDidUpdate() {
        liveLayer.path = trace(engine.latestPlot())
    }
    
    func ghostDidChange() {
        ghostLayer.path = engine.ghost?.withUnsafeBufferPointer { trace($0) }
    }
    
    private func trace(_ yVec: UnsafeBufferPointer<Float>) -> CGPath {
        let path = CGMutablePath()
        let w = bounds.width, h = bounds.height
        let dx = w / CGFloat(yVec.count - 1)
        path.move(to: CGPoint(x: 0, y: h))
        
        for (i, y) in yVec.enumerated() {
            path.addLine(to: CGPoint(x: CGFloat(i) * dx, y: h * CGFloat(1 - y)))
        }
        path.addLine(to: CGPoint(x: w, y: h))
        return path
    }
    
    private func gridPath() -> CGPath? {
        guard let grid = engine.grid else { return nil }
        let path = CGMutablePath()
        for octave in 2...8 {
            let x = bounds.width * CGFloat(grid.x(ofHz: Self.c0Hz * pow(2, Float(octave))))
            path.move(to: CGPoint(x: x, y: 0))
            path.addLine(to: CGPoint(x: x, y: bounds.height))
        }
        return path
    }
    
    private func applyColors() {
        let tint = tintColor.resolvedColor(with: traitCollection)
        liveLayer.strokeColor = tint.cgColor
        liveLayer.fillColor = tint.withAlphaComponent(0.2).cgColor
        ghostLayer.strokeColor = UIColor.secondaryLabel.resolvedColor(with: traitCollection).cgColor
        gridLayer.strokeColor = UIColor.separator.resolvedColor(with: traitCollection).cgColor
        gridLayer.lineWidth = 1 / traitCollection.displayScale
    }
}
