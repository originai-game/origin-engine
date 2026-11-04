// The target application a UI test bundle needs; the test drives Safari.
import SwiftUI

@main
struct HostApp: App {
    var body: some Scene {
        WindowGroup { Text("Origin Engine Safari compatibility host") }
    }
}
