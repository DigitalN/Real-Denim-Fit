#!/usr/bin/env swift
// Draws the README's screenshots, in light and dark, from the example profile next to this
// script. Run it by hand on a Mac, from the repo root, after a change to the screens or the
// catalogue that the pictures should show:
//
//   swift docs/screenshots/take-screenshots.swift
//
// It needs only Xcode's command line tools and Rust, nothing installed. It:
//
//   1. copies example-profile.json to a temporary folder, so the profile itself is never
//      written to (the preview saves what's on screen as it goes);
//   2. builds the browser preview (src-tauri/examples/preview.rs) and starts it on port 1430,
//      with RDF_STATE_FILE pointing at that copy;
//   3. opens the screens in a WebKit view with no title bar, the same engine as the app's
//      window on macOS, and saves only the window's contents, at 2×;
//   4. stops the preview and deletes the copy.
//
// The window is invisible (1% opaque, and clicks go through it) but on screen and in front,
// because WebKit draws checkboxes and radio buttons faded in a window that's off screen or in
// the background. So it takes the keyboard focus for the half minute it runs. The page ignores
// the pointer, so nothing shows as hovered wherever it is.
//
// The app has no theme switch of its own: dark mode comes only from prefers-color-scheme in
// ui/styles.css, so each picture is taken once with the view's appearance set to Aqua (light)
// and once with DarkAqua (dark). The results screen fills the app's default window,
// 1100 × 780 pt (tauri.conf.json). Each close-up frames one part of a screen in a narrower
// window, so its text stays readable at the size GitHub shows it.
//
// Each picture is saved here as <name>-light.png and <name>-dark.png, and README.md shows
// them. The body in example-profile.json is made up; the jeans come from the catalogue as it
// is in this checkout. Look at every picture before committing them.

import AppKit
import WebKit

let here = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let root = here.deletingLastPathComponent().deletingLastPathComponent()
let port = 1430
let window = NSSize(width: 1100, height: 780)
/// Page background kept around a close-up.
let pad: CGFloat = 8

struct Shot {
    let name: String
    /// Run in the page to open the screen. It may await, for example to ask Rust for a match.
    let open: String
    /// Window width; nil for the default window, saved whole.
    let width: CGFloat?
    /// Returns the elements a close-up frames; the picture is the box around all of them.
    let frame: String?
}

let shots = [
    Shot(name: "results", open: "location.hash = '#/results'", width: nil, frame: nil),
    // The 12″ leg screen: the guide's 7th body screen (ui/guide/steps.json, "body-leg-12in").
    Shot(
        name: "guide", open: "location.hash = '#/guide/6'", width: 900,
        frame: "[...document.querySelectorAll('#app .narrow > :not(.actions)')]"),
    Shot(
        name: "fit", open: "location.hash = '#/fit'", width: 1000,
        frame: "[...document.querySelectorAll('.fit-layout > .card')]"),
    // The closest match whose after-wash measurements are estimated from the brand's
    // shrinkage, so the picture shows as-sold and after-wash side by side.
    Shot(
        name: "product",
        open: """
            const r = await invoke('match_catalogue', { profile: profileForRust(), filters: filtersForRust() });
            const m = r.rows.map((row) => row.best).find((b) => b.basis === 'from-shrinkage');
            location.hash = `#/product/${encodeURIComponent(m.product_id)}/${encodeURIComponent(m.size)}`;
            """,
        width: 900,
        frame: "[...document.querySelectorAll('#app > h1, #app > .card')].slice(0, 4)"),
    Shot(
        name: "pair", open: "location.hash = '#/pair/' + encodeURIComponent(S.pairs[0].id)", width: 900,
        frame: "(e => [e.previousElementSibling, e])(document.getElementById('ease-card'))"),
]

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(("take-screenshots: " + message + "\n").data(using: .utf8)!)
    exit(1)
}

func run(_ tool: String, _ args: [String]) {
    let p = Process()
    p.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    p.arguments = [tool] + args
    p.currentDirectoryURL = root
    do { try p.run() } catch { fail("couldn't run \(tool): \(error)") }
    p.waitUntilExit()
    if p.terminationStatus != 0 { fail("\(tool) \(args.joined(separator: " ")) failed") }
}

// 1. A copy of the example profile, for the preview to read and write.
let scratch = FileManager.default.temporaryDirectory.appendingPathComponent("rdf-screenshots-\(UUID().uuidString)")
let stateCopy = scratch.appendingPathComponent("state.json")
do {
    try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
    try FileManager.default.copyItem(at: here.appendingPathComponent("example-profile.json"), to: stateCopy)
} catch {
    fail("couldn't copy the example profile: \(error)")
}

// 2. The preview, on the copy.
print("Building the preview…")
run("cargo", ["build", "-q", "-p", "real-denim-fit", "--example", "preview"])
let server = Process()
server.executableURL = root.appendingPathComponent("target/debug/examples/preview")
server.arguments = [String(port)]
server.currentDirectoryURL = root
var env = ProcessInfo.processInfo.environment
env["RDF_STATE_FILE"] = stateCopy.path
server.environment = env
do { try server.run() } catch { fail("couldn't start the preview: \(error)") }

func cleanUp() {
    if server.isRunning { server.terminate() }
    try? FileManager.default.removeItem(at: scratch)
}

// Waits until the preview answers, then loads the app.
let base = URL(string: "http://127.0.0.1:\(port)/")!
for attempt in 0... {
    if !server.isRunning { cleanUp(); fail("the preview stopped. Is port \(port) already in use?") }
    if (try? Data(contentsOf: base)) != nil { break }
    if attempt > 100 { cleanUp(); fail("the preview didn't answer on port \(port)") }
    Thread.sleep(forTimeInterval: 0.1)
}

// 3. The pictures.

/// A borderless window can't become the key window unless it says so.
final class KeyWindow: NSWindow {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
}

final class Shooter: NSObject, WKNavigationDelegate {
    let win: NSWindow
    let view: WKWebView
    var jobs: [(Shot, String)] = ["light", "dark"].flatMap { theme in shots.map { ($0, theme) } }

    override init() {
        let rect = NSRect(origin: .zero, size: window)
        win = KeyWindow(contentRect: rect, styleMask: .borderless, backing: .buffered, defer: false)
        win.setFrameOrigin(.zero)
        win.alphaValue = 0.01  // invisible, but on screen (see the top of this file)
        win.ignoresMouseEvents = true
        view = WKWebView(frame: rect, configuration: WKWebViewConfiguration())
        super.init()
        view.navigationDelegate = self
        win.contentView = view
        toFront()
        view.load(URLRequest(url: base))
    }

    func toFront() {
        NSApp.activate(ignoringOtherApps: true)  // activate() alone is more easily turned down
        win.makeKeyAndOrderFront(nil)
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        // The window ignores clicks, but WebKit still follows the pointer, so a row under
        // wherever it happens to be would show as hovered. Nothing on the page reacts to it.
        view.evaluateJavaScript("document.documentElement.style.pointerEvents = 'none'") { _, _ in
            self.after(2.0, self.next)
        }
    }

    func after(_ seconds: TimeInterval, _ fn: @escaping () -> Void) {
        DispatchQueue.main.asyncAfter(deadline: .now() + seconds, execute: fn)
    }

    func next() {
        guard let (shot, theme) = jobs.first else { return finish() }
        // A close-up's window is tall enough for the whole part it frames.
        win.setContentSize(shot.width.map { NSSize(width: $0, height: 2000) } ?? window)
        let look = NSAppearance(named: theme == "dark" ? .darkAqua : .aqua)
        win.appearance = look
        view.appearance = look
        toFront()
        view.callAsyncJavaScript(shot.open, arguments: [:], in: nil, in: .page) { result in
            if case .failure(let error) = result { print("Couldn't open \(shot.name): \(error)") }
            self.after(1.5, self.measure)
        }
    }

    func measure() {
        guard let frame = jobs.first?.0.frame else { return shoot(nil) }
        let js = """
            window.scrollTo(0, 0);
            const r = (\(frame)).filter(Boolean).map((e) => e.getBoundingClientRect());
            if (!r.length) return null;
            const x = Math.min(...r.map((b) => b.left)) - \(pad), y = Math.min(...r.map((b) => b.top)) - \(pad);
            return [x, y, Math.max(...r.map((b) => b.right)) + \(pad) - x, Math.max(...r.map((b) => b.bottom)) + \(pad) - y];
            """
        view.callAsyncJavaScript(js, arguments: [:], in: nil, in: .page) { result in
            guard case .success(let value) = result, let box = value as? [Double], box.count == 4 else {
                print("Nothing to frame for \(self.jobs[0].0.name)")
                return self.skip()
            }
            self.shoot(NSRect(x: box[0], y: box[1], width: box[2], height: box[3]))
        }
    }

    /// Asks again for the front for up to 5 seconds, since macOS can turn a request down.
    func shoot(_ rect: NSRect?, tries: Int = 25) {
        if !(NSApp.isActive && win.isKeyWindow) && tries > 0 {
            toFront()
            return after(0.2) { self.shoot(rect, tries: tries - 1) }
        }
        if !(NSApp.isActive && win.isKeyWindow) {
            print("The window wasn't in front for \(jobs[0].0.name), so its checkboxes may look faded.")
            print("Run the script again and leave the keyboard alone while it runs.")
        }
        let config = WKSnapshotConfiguration()
        let area = rect ?? NSRect(origin: .zero, size: window)
        config.rect = area
        // 2× whatever the screen is: the width asked for is in points at the window's scale.
        config.snapshotWidth = NSNumber(value: Double(area.width * 2 / win.backingScaleFactor))
        // No focus ring or caret in the pictures. This comes last, because a window coming to
        // the front gives the focus back to the field that had it.
        view.evaluateJavaScript("document.activeElement && document.activeElement.blur()") { _, _ in
            self.after(0.3) { self.snapshot(config) }
        }
    }

    func snapshot(_ config: WKSnapshotConfiguration) {
        view.takeSnapshot(with: config) { image, error in
            guard let image, let cg = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
                print("Couldn't take \(self.jobs[0].0.name): \(String(describing: error))")
                return self.skip()
            }
            let (shot, theme) = self.jobs[0]
            let file = here.appendingPathComponent("\(shot.name)-\(theme).png")
            let png = NSBitmapImageRep(cgImage: cg).representation(using: .png, properties: [:])
            do { try png?.write(to: file) } catch { print("Couldn't save \(file.path): \(error)") }
            print("Saved docs/screenshots/\(file.lastPathComponent) (\(cg.width) × \(cg.height))")
            self.skip()
        }
    }

    func skip() {
        jobs.removeFirst()
        next()
    }

    func finish() {
        cleanUp()
        exit(0)
    }
}

let app = NSApplication.shared
// No Dock icon, but it can be the active app, so its window draws as it does in use.
app.setActivationPolicy(.accessory)
let shooter = Shooter()
app.run()
