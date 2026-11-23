// The SwiftUI side of scripts/macos-fullscreen.sh: one window whose style is picked at build time
// (-D HIDDEN or -D PLAIN; none is the default title bar). It prints "full-screen: true|false"
// when the window enters or leaves full screen, as agents/recipes/full_screen.rs does.
import SwiftUI

@main
struct Reference: App {
    init() {
        for (name, state) in [
            (NSWindow.didEnterFullScreenNotification, true),
            (NSWindow.didExitFullScreenNotification, false),
        ] {
            NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { _ in
                FileHandle.standardError.write(Data("full-screen: \(state)\n".utf8))
            }
        }
    }

    var body: some Scene {
        let group = WindowGroup("Full screen") {
            Text("In a window").frame(width: 520, height: 320)
        }
        #if HIDDEN
        group.windowStyle(.hiddenTitleBar)
        #elseif PLAIN
        group.windowStyle(.plain)
        #else
        group
        #endif
    }
}
