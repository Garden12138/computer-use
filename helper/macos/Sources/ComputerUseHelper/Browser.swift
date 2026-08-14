import AppKit
import ApplicationServices
import Foundation

enum Browser {
    static func openProfile(_ directory: String) throws {
        let chrome = URL(fileURLWithPath: "/Applications/Google Chrome.app")
        let config = NSWorkspace.OpenConfiguration()
        config.arguments = ["--profile-directory=\(directory)", "--no-first-run", "--no-default-browser-check"]
        config.activates = true
        let sem = DispatchSemaphore(value: 0)
        var thrown: Error?
        NSWorkspace.shared.openApplication(at: chrome, configuration: config) { _, error in
            thrown = error
            sem.signal()
        }
        sem.wait()
        if let thrown {
            throw HelperError.failed(thrown.localizedDescription)
        }
        usleep(800_000)
    }

    static func openURL(_ url: String) throws {
        try Windows.focusApp("Google Chrome")
        try Input.hotkey(["cmd", "l"])
        usleep(200_000)
        try Input.hotkey(["cmd", "a"])
        usleep(80_000)
        try Input.typeText(url, interval: 0.02)
        usleep(120_000)
        try Input.hotkey(["enter"])
    }

    static func savePage(path: String) throws {
        try Windows.focusApp("Google Chrome")
        try Input.hotkey(["cmd", "s"])
        usleep(1_200_000)
        try chooseWebpageComplete()
        let dest = URL(fileURLWithPath: path)
        let directory = dest.deletingLastPathComponent().path
        let filename = dest.deletingPathExtension().lastPathComponent
        try Input.hotkey(["cmd", "shift", "g"])
        usleep(400_000)
        try Input.typeText(directory, interval: 0.01)
        try Input.hotkey(["enter"])
        usleep(400_000)
        try Input.hotkey(["cmd", "a"])
        try Input.typeText(filename, interval: 0.02)
        usleep(200_000)
        try Input.hotkey(["enter"])
        usleep(400_000)
        try Input.hotkey(["enter"])
    }

    private static func chooseWebpageComplete() throws {
        let sys = AXUIElementCreateSystemWide()
        var focused: CFTypeRef?
        AXUIElementCopyAttributeValue(sys, kAXFocusedApplicationAttribute as CFString, &focused)
        guard let app = focused else { return }
        let appEl = app as! AXUIElement
        var windowsRef: CFTypeRef?
        AXUIElementCopyAttributeValue(appEl, kAXWindowsAttribute as CFString, &windowsRef)
        guard let windows = windowsRef as? [AXUIElement], let window = windows.first else { return }
        var sheetRef: CFTypeRef?
        AXUIElementCopyAttributeValue(window, "AXSheets" as CFString, &sheetRef)
        let sheets = sheetRef as? [AXUIElement] ?? []
        guard let sheet = sheets.first else { return }
        clickPopup(on: sheet, labels: ["网页，全部", "Webpage, Complete", "Web Page, Complete"])
    }

    private static func clickPopup(on sheet: AXUIElement, labels: [String]) {
        var childrenRef: CFTypeRef?
        AXUIElementCopyAttributeValue(sheet, kAXChildrenAttribute as CFString, &childrenRef)
        guard let children = childrenRef as? [AXUIElement] else { return }
        for child in children {
            var roleRef: CFTypeRef?
            AXUIElementCopyAttributeValue(child, kAXRoleAttribute as CFString, &roleRef)
            let role = roleRef as? String ?? ""
            if role == kAXPopUpButtonRole as String {
                AXUIElementPerformAction(child, kAXPressAction as CFString)
                usleep(300_000)
                var menuRef: CFTypeRef?
                AXUIElementCopyAttributeValue(child, kAXChildrenAttribute as CFString, &menuRef)
                let menus = menuRef as? [AXUIElement] ?? []
                for menu in menus {
                    var itemsRef: CFTypeRef?
                    AXUIElementCopyAttributeValue(menu, kAXChildrenAttribute as CFString, &itemsRef)
                    let items = itemsRef as? [AXUIElement] ?? []
                    for item in items {
                        var titleRef: CFTypeRef?
                        AXUIElementCopyAttributeValue(item, kAXTitleAttribute as CFString, &titleRef)
                        let title = titleRef as? String ?? ""
                        if labels.contains(title) {
                            AXUIElementPerformAction(item, kAXPressAction as CFString)
                            return
                        }
                    }
                }
            }
            clickPopup(on: child, labels: labels)
        }
    }
}
