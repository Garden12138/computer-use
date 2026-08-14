import AppKit
import CoreGraphics
import Foundation

enum Input {
    static var lastScale: Double = Double(NSScreen.main?.backingScaleFactor ?? 1.0)

    static func move(x: Double, y: Double, duration: Double, scale: Double) throws {
        try Permissions.requireInput()
        let end = screenshotToCG(x: x, y: y, scale: scale)
        animateMove(to: end, duration: duration)
    }

    static func click(x: Double, y: Double, button: String, count: Int, duration: Double, scale: Double) throws {
        try Permissions.requireInput()
        let point = screenshotToCG(x: x, y: y, scale: scale)
        animateMove(to: point, duration: duration)
        let cgButton: CGMouseButton = button == "right" ? .right : (button == "middle" ? .center : .left)
        let downType: CGEventType = button == "right" ? .rightMouseDown : (button == "middle" ? .otherMouseDown : .leftMouseDown)
        let upType: CGEventType = button == "right" ? .rightMouseUp : (button == "middle" ? .otherMouseUp : .leftMouseUp)
        for i in 1 ... max(count, 1) {
            postMouse(downType, button: cgButton, at: point, clickCount: i)
            usleep(40_000)
            postMouse(upType, button: cgButton, at: point, clickCount: i)
            if i < count { usleep(80_000) }
        }
    }

    static func scroll(deltaX: Double, deltaY: Double, x: Double?, y: Double?, scale: Double) throws {
        try Permissions.requireInput()
        if let x, let y {
            animateMove(to: screenshotToCG(x: x, y: y, scale: scale), duration: 0.05)
        }
        // CGEvent scroll units are roughly lines; pixel deltas from the pacer
        // are mapped into wheel ticks.
        let wheelY = Int32((deltaY / 40.0).rounded())
        let wheelX = Int32((deltaX / 40.0).rounded())
        guard let event = CGEvent(
            scrollWheelEvent2Source: nil,
            units: .pixel,
            wheelCount: 2,
            wheel1: Int32(deltaY.rounded()),
            wheel2: Int32(deltaX.rounded()),
            wheel3: 0
        ) else {
            throw HelperError.failed("could not create scroll event")
        }
        _ = wheelY
        _ = wheelX
        event.post(tap: .cghidEventTap)
    }

    static func typeText(_ text: String, interval: Double) throws {
        try Permissions.requireInput()
        for ch in text {
            if ch == "\n" {
                try key(["enter"], down: true)
                try key(["enter"], down: false)
            } else {
                var utf16 = Array(String(ch).utf16)
                guard let down = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true) else {
                    throw HelperError.failed("keyboard event")
                }
                down.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: &utf16)
                down.post(tap: .cghidEventTap)
                guard let up = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: false) else {
                    throw HelperError.failed("keyboard event")
                }
                up.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: &utf16)
                up.post(tap: .cghidEventTap)
            }
            if interval > 0 { usleep(UInt32(interval * 1_000_000)) }
        }
    }

    static func hotkey(_ keys: [String]) throws {
        try Permissions.requireInput()
        try key(keys, down: true)
        usleep(30_000)
        try key(keys, down: false)
    }

    static func drag(x: Double, y: Double, endX: Double, endY: Double, duration: Double, scale: Double) throws {
        try Permissions.requireInput()
        let start = screenshotToCG(x: x, y: y, scale: scale)
        let end = screenshotToCG(x: endX, y: endY, scale: scale)
        animateMove(to: start, duration: min(duration, 0.12))
        postMouse(.leftMouseDown, button: .left, at: start, clickCount: 1)
        animateMove(to: end, duration: duration, drag: true)
        postMouse(.leftMouseUp, button: .left, at: end, clickCount: 1)
    }

    static func wait(_ seconds: Double) {
        if seconds > 0 { usleep(UInt32(seconds * 1_000_000)) }
    }

    private static func key(_ names: [String], down: Bool) throws {
        let mods = names.map { $0.lowercased() }
        var flags: CGEventFlags = []
        var keyCode: CGKeyCode?
        for name in mods {
            switch name {
            case "cmd", "command", "meta": flags.insert(.maskCommand)
            case "shift": flags.insert(.maskShift)
            case "alt", "option": flags.insert(.maskAlternate)
            case "ctrl", "control": flags.insert(.maskControl)
            case "fn": flags.insert(.maskSecondaryFn)
            default:
                keyCode = virtualKey(name)
            }
        }
        let code = keyCode ?? 0
        guard let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down) else {
            throw HelperError.failed("keyboard event")
        }
        event.flags = flags
        event.post(tap: .cghidEventTap)
    }

    private static func virtualKey(_ name: String) -> CGKeyCode {
        switch name.lowercased() {
        case "enter", "return": return 36
        case "tab": return 48
        case "space": return 49
        case "delete", "backspace": return 51
        case "escape", "esc": return 53
        case "home": return 115
        case "end": return 119
        case "pageup", "page_up": return 116
        case "pagedown", "page_down": return 121
        case "left": return 123
        case "right": return 124
        case "down": return 125
        case "up": return 126
        case "a": return 0
        case "s": return 1
        case "d": return 2
        case "f": return 3
        case "h": return 4
        case "g": return 5
        case "z": return 6
        case "x": return 7
        case "c": return 8
        case "v": return 9
        case "b": return 11
        case "q": return 12
        case "w": return 13
        case "e": return 14
        case "r": return 15
        case "y": return 16
        case "t": return 17
        case "1": return 18
        case "2": return 19
        case "3": return 20
        case "4": return 21
        case "6": return 22
        case "5": return 23
        case "9": return 25
        case "7": return 26
        case "8": return 28
        case "0": return 29
        case "l": return 37
        case "n": return 45
        case "m": return 46
        case "p": return 35
        case "o": return 31
        case "i": return 34
        case "k": return 40
        case "j": return 38
        default: return 0
        }
    }

    private static func animateMove(to end: CGPoint, duration: Double, drag: Bool = false) {
        let start = currentMouse()
        let steps = max(1, Int((max(duration, 0) * 90).rounded()))
        if duration <= 0 || steps == 1 {
            postMouse(drag ? .leftMouseDragged : .mouseMoved, button: .left, at: end, clickCount: 0)
            return
        }
        for i in 1 ... steps {
            let t = Double(i) / Double(steps)
            let eased = t * t * (3 - 2 * t)
            let p = CGPoint(
                x: start.x + (end.x - start.x) * eased,
                y: start.y + (end.y - start.y) * eased
            )
            postMouse(drag ? .leftMouseDragged : .mouseMoved, button: .left, at: p, clickCount: 0)
            usleep(UInt32((duration / Double(steps)) * 1_000_000))
        }
    }

    private static func currentMouse() -> CGPoint {
        CGEvent(source: nil)?.location ?? .zero
    }

    static func cursor(scale: Double) -> [String: Any] {
        let point = currentMouse()
        let s = scale > 0 ? scale : 1
        return [
            "x": point.x * s,
            "y": point.y * s,
            "x_points": point.x,
            "y_points": point.y,
            "scale": s,
        ]
    }

    private static func postMouse(_ type: CGEventType, button: CGMouseButton, at point: CGPoint, clickCount: Int) {
        guard let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: button) else {
            return
        }
        if clickCount > 0 {
            event.setIntegerValueField(.mouseEventClickState, value: Int64(clickCount))
        }
        event.post(tap: .cghidEventTap)
    }
}
