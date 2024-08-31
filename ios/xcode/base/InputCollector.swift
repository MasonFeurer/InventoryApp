import Foundation
import UIKit

class InputCollector: UIView, UIKeyInput {
    func insertText(_ text: String) {
        var bytes = text.cString(using: String.Encoding.utf8)!
        event_text_input(ViewController.nativeApp, &bytes, Int32(bytes.count) - 1)
    }
    func deleteBackward() {
        event_key_typed_backspace(ViewController.nativeApp)
    }
    var hasText: Bool {
       return true
    }
    override var canBecomeFirstResponder: Bool {
       return true
    }
}
