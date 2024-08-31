import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?
        
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        window = UIWindow(frame: UIScreen.main.bounds)
        let mainStroryBoard = UIStoryboard(name: "Main", bundle: nil)
        window?.rootViewController = mainStroryBoard.instantiateInitialViewController()
        window?.addSubview(ViewController.activeTextField)

        window?.makeKeyAndVisible()
        return true
    }
    
    override var canResignFirstResponder: Bool {
        return true
    }
    override func resignFirstResponder() -> Bool {
        return true
    }
    override var canBecomeFirstResponder: Bool {
       return true
    }
    
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        let touch = touches.first!
        let location = touch.location(in: self.inputView)
        event_touch_begin(ViewController.nativeApp, Float(location.x), Float(location.y))
    }
    
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        let touch = touches.first!
        let location = touch.location(in: self.inputView)
        event_touch_move(ViewController.nativeApp, Float(location.x), Float(location.y))
    }
    
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        let touch = touches.first!
        let location = touch.location(in: self.inputView)
        event_touch_end(ViewController.nativeApp, Float(location.x), Float(location.y))
    }
}

