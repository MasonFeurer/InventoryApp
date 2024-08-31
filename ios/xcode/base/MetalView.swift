import UIKit
import Foundation

class MetalView: UIView {
    override class var layerClass: AnyClass {
        return CAMetalLayer.self
    }
    
    override func awakeFromNib() {
        super.awakeFromNib()
        configLayer()
        self.layer.backgroundColor = UIColor.clear.cgColor
    }
    
    private func configLayer() {
        print("running configLayer on MetalView");
        guard let layer = self.layer as? CAMetalLayer else {
            return
        }
        
        print("MetalView got layer");
        // https://developer.apple.com/documentation/quartzcore/cametallayer/1478157-presentswithtransaction/
        layer.presentsWithTransaction = false
        layer.framebufferOnly = true
        // nativeScale is real physical pixel scale
        // https://tomisacat.xyz/tech/2017/06/17/scale-nativescale-contentsscale.html
        self.contentScaleFactor = UIScreen.main.nativeScale
    }
}

