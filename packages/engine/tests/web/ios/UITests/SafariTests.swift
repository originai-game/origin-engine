// Drives Safari in the iOS Simulator through the steps tests/web/ios-sim.ts
// judges: the exported page opened at OG_URL, then real touches (tap, double
// tap, swipes that move Safari's toolbar, a pinch), both orientations, and
// backgrounding and returning. Before each step it posts a mark to the
// harness (OG_CTL/mark), so the page's own event log lines up with the
// steps, and its screenshots go to OG_CTL/shot (the Simulator's screen, as
// XCTest captures it). The harness's page reports to the same server.
import XCTest

final class SafariTests: XCTestCase {
    private let env = ProcessInfo.processInfo.environment

    private func send(_ path: String, _ body: Data = Data()) {
        guard let ctl = env["OG_CTL"], let url = URL(string: ctl + path) else { return }
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.httpBody = body
        let done = expectation(description: path)
        URLSession.shared.dataTask(with: req) { _, _, _ in done.fulfill() }.resume()
        wait(for: [done], timeout: 10)
    }

    private func mark(_ step: String, _ detail: String = "") {
        send("/mark?step=\(step)", Data(detail.utf8))
    }

    private func shot(_ name: String) {
        send("/shot?name=\(name)", XCUIScreen.main.screenshot().pngRepresentation)
    }

    /// Waits until the page has reported `key` to the harness.
    private func page(_ key: String, timeout: TimeInterval) -> Bool {
        guard let ctl = env["OG_CTL"], let url = URL(string: "\(ctl)/has?key=\(key)") else { return false }
        let end = Date().addingTimeInterval(timeout)
        while Date() < end {
            var ok = false
            let done = expectation(description: "has \(key)")
            URLSession.shared.dataTask(with: url) { _, r, _ in
                ok = (r as? HTTPURLResponse)?.statusCode == 200
                done.fulfill()
            }.resume()
            wait(for: [done], timeout: 10)
            if ok { return true }
            Thread.sleep(forTimeInterval: 0.25)
        }
        return false
    }

    /// The web view's buttons and their frames in screen points (where Safari puts the page's controls).
    private func buttons(_ web: XCUIElement) -> String {
        web.buttons.allElementsBoundByIndex.map { b in
            let f = b.frame
            return "\(b.label)|\(Int(f.minX)),\(Int(f.minY)),\(Int(f.width)),\(Int(f.height))|\(b.isHittable)"
        }.joined(separator: "\n")
    }

    func testShell() throws {
        let url = try XCTUnwrap(URL(string: env["OG_URL"] ?? ""), "OG_URL is not set")
        XCUIDevice.shared.orientation = .portrait
        let safari = XCUIApplication(bundleIdentifier: "com.apple.mobilesafari")
        safari.launch()
        XCUIDevice.shared.system.open(url)
        XCTAssertTrue(safari.wait(for: .runningForeground, timeout: 30))
        XCTAssertTrue(page("ready", timeout: 90), "the page never reported ready")
        Thread.sleep(forTimeInterval: 1.5)
        let web = safari.webViews.firstMatch
        let screen = XCUIScreen.main.screenshot().image.size
        mark("portrait", "window \(Int(safari.windows.firstMatch.frame.width))x\(Int(safari.windows.firstMatch.frame.height)) web \(web.frame) screen \(screen)\n\(buttons(web))")
        shot("portrait")

        mark("tap")
        web.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.62)).tap()
        Thread.sleep(forTimeInterval: 0.5)
        mark("double-tap")
        web.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.62)).doubleTap()
        Thread.sleep(forTimeInterval: 1)
        shot("after-double-tap")
        mark("swipe-up")
        web.swipeUp()
        Thread.sleep(forTimeInterval: 1.5)
        shot("after-swipe-up")
        mark("swipe-down")
        web.swipeDown()
        Thread.sleep(forTimeInterval: 1.5)
        mark("pinch-out")
        web.pinch(withScale: 2.0, velocity: 1.0)
        Thread.sleep(forTimeInterval: 1.5)
        shot("after-pinch")
        mark("pinch-in")
        web.pinch(withScale: 0.5, velocity: -1.0)
        Thread.sleep(forTimeInterval: 1.5)

        mark("landscape")
        XCUIDevice.shared.orientation = .landscapeLeft
        Thread.sleep(forTimeInterval: 2.5)
        mark("landscape-settled", buttons(web))
        shot("landscape")
        mark("landscape-tap")
        web.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)).tap()
        Thread.sleep(forTimeInterval: 0.5)
        mark("portrait-again")
        XCUIDevice.shared.orientation = .portrait
        Thread.sleep(forTimeInterval: 2.5)
        shot("portrait-again")

        mark("background")
        XCUIDevice.shared.press(.home)
        Thread.sleep(forTimeInterval: 3)
        mark("return")
        safari.activate()
        Thread.sleep(forTimeInterval: 2.5)
        shot("returned")
        mark("done")
    }
}
