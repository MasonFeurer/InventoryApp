import UIKit

class ViewController: UIViewController {
    @IBOutlet var metalV: MetalView!
    static var nativeApp: OpaquePointer?
    static var activeTextField: InputCollector = InputCollector()
    
    lazy var displayLink: CADisplayLink = {
        CADisplayLink.init(target: self, selector: #selector(enterFrame))
    }()
    
    override func viewDidLoad() {
        super.viewDidLoad()
       
        self.displayLink.add(to: .current, forMode: .default)
        self.displayLink.isPaused = true
    }
    
    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        self.view.backgroundColor = .green
        if ViewController.nativeApp == nil {
            let viewPointer = Unmanaged.passUnretained(self.metalV).toOpaque()
            let metalLayer = Unmanaged.passUnretained(self.metalV.layer).toOpaque()
            let maximumFrames = UIScreen.main.maximumFramesPerSecond
            
            let viewObj = ios_view_obj(
                view: viewPointer,
                metal_layer: metalLayer,
                maximum_frames: Int32(maximumFrames),
                open_keyboard: open_keyboard,
                close_keyboard: close_keyboard
            )
            
            ViewController.nativeApp = create_app(viewObj)
        }
        self.displayLink.isPaused = false
    }
    
    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        displayLink.isPaused = true
    }
    
    @objc func enterFrame() {
        guard let canvas = ViewController.nativeApp else {
            return
        }
        draw_frame(canvas)
    }
}

func open_keyboard() {
    let rs = ViewController.activeTextField.becomeFirstResponder()
    if !rs {
        print("[ERROR] Failed to open keyboard")
    }
}
func close_keyboard() {
    ViewController.activeTextField.resignFirstResponder()
}
