import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

enum Windows {
    static func list() throws -> [String: Any] {
        let options = CGWindowListOption.optionOnScreenOnly.union(.excludeDesktopElements)
        let infos = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []
        var windows: [[String: Any]] = []
        for info in infos {
            let layer = info[kCGWindowLayer as String] as? Int ?? 0
            if layer != 0 { continue }
            let bounds = info[kCGWindowBounds as String] as? [String: Any] ?? [:]
            let width = (bounds["Width"] as? NSNumber)?.intValue ?? 0
            let height = (bounds["Height"] as? NSNumber)?.intValue ?? 0
            if width < 80 || height < 80 { continue }
            let windowId = (info[kCGWindowNumber as String] as? NSNumber)?.uint32Value ?? 0
            windows.append([
                "app": info[kCGWindowOwnerName as String] as? String ?? "",
                "title": info[kCGWindowName as String] as? String ?? "",
                "window_id": windowId,
                "pid": info[kCGWindowOwnerPID as String] as? Int ?? 0,
                "bounds": [
                    "x": (bounds["X"] as? NSNumber)?.doubleValue ?? 0,
                    "y": (bounds["Y"] as? NSNumber)?.doubleValue ?? 0,
                    "width": width,
                    "height": height,
                ],
            ])
        }
        return ["windows": windows]
    }

    static func active() throws -> [String: Any] {
        let app = NSWorkspace.shared.frontmostApplication
        return [
            "app": app?.localizedName ?? "",
            "pid": app?.processIdentifier ?? 0,
            "bundle_id": app?.bundleIdentifier ?? "",
        ]
    }

    static func focusApp(_ name: String) throws {
        try Permissions.requireInput()
        let apps = NSWorkspace.shared.runningApplications
        let target = apps.first { app in
            let loc = app.localizedName ?? ""
            let bid = app.bundleIdentifier ?? ""
            return loc.caseInsensitiveCompare(name) == .orderedSame
                || bid.caseInsensitiveCompare(name) == .orderedSame
                || loc.localizedCaseInsensitiveContains(name)
        }
        guard let app = target else {
            throw HelperError.failed("app not running: \(name)")
        }
        let ok = app.activate(options: [.activateIgnoringOtherApps])
        if !ok {
            throw HelperError.failed("could not activate \(name)")
        }
        usleep(250_000)
    }

    static func focusWindow(app: String?, title: String?, windowId: UInt32?) throws {
        try Permissions.requireInput()
        if let app, !app.isEmpty {
            try focusApp(app)
        }
        if let title, !title.isEmpty {
            try raiseWindow(matching: title)
        }
        if windowId != nil {
            // AX raise by title is the reliable path; window id is used for screenshots.
            _ = windowId
        }
    }

    private static func raiseWindow(matching title: String) throws {
        let apps = NSWorkspace.shared.runningApplications
        for app in apps where app.activationPolicy == .regular {
            let el = AXUIElementCreateApplication(app.processIdentifier)
            var windowsRef: CFTypeRef?
            let status = AXUIElementCopyAttributeValue(el, kAXWindowsAttribute as CFString, &windowsRef)
            guard status == .success, let windows = windowsRef as? [AXUIElement] else { continue }
            for window in windows {
                var titleRef: CFTypeRef?
                AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &titleRef)
                let windowTitle = titleRef as? String ?? ""
                if windowTitle.localizedCaseInsensitiveContains(title) {
                    AXUIElementPerformAction(window, kAXRaiseAction as CFString)
                    app.activate(options: [.activateIgnoringOtherApps])
                    return
                }
            }
        }
        throw HelperError.failed("window not found: \(title)")
    }
}
