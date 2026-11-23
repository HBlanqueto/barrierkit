// Moves the pointer to x y and presses and releases the left button, as a person would.
// Usage: click x y   (needs the Accessibility permission of the terminal that runs it)
import CoreGraphics
import Foundation

let args = CommandLine.arguments
guard args.count == 3, let x = Double(args[1]), let y = Double(args[2]) else {
    FileHandle.standardError.write(Data("usage: click x y\n".utf8))
    exit(2)
}
let point = CGPoint(x: x, y: y)
let source = CGEventSource(stateID: .hidSystemState)
func post(_ type: CGEventType) {
    CGEvent(mouseEventSource: source, mouseType: type, mouseCursorPosition: point, mouseButton: .left)?
        .post(tap: .cghidEventTap)
}
post(.mouseMoved)
usleep(400_000)
post(.leftMouseDown)
usleep(150_000)
post(.leftMouseUp)
