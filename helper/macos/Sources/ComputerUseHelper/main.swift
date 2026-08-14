import Foundation

struct ComputerUseHelperMain {
    static func main() {
        let args = Array(CommandLine.arguments.dropFirst())
        if args.isEmpty || args.first == "stdio" {
            runStdio()
            return
        }
        do {
            let request = try argvRequest(args)
            handle(request)
        } catch let error as HelperError {
            failure(id: "cli", error: error)
            exit(2)
        } catch {
            failure(id: "cli", error: .failed(error.localizedDescription))
            exit(2)
        }
    }

    static func runStdio() {
        while let line = readLine() {
            let trimmed = line.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.isEmpty { continue }
            do {
                handle(try jsonObject(trimmed))
            } catch let error as HelperError {
                failure(id: "unknown", error: error)
            } catch {
                failure(id: "unknown", error: .failed(error.localizedDescription))
            }
        }
    }

    static func handle(_ req: [String: Any]) {
        let id = (req["id"] as? String) ?? "0"
        let cmd = (req["cmd"] as? String) ?? ""
        do {
            let data = try dispatch(cmd: cmd, req: req)
            success(id: id, data: data)
        } catch let error as HelperError {
            failure(id: id, error: error)
        } catch {
            failure(id: id, error: .failed(error.localizedDescription))
        }
    }

    static func dispatch(cmd: String, req: [String: Any]) throws -> [String: Any] {
        let scale = optionalDouble(req, "scale", default: Input.lastScale)
        Input.lastScale = scale
        switch cmd {
        case "doctor":
            return Permissions.doctor()
        case "screenshot":
            let path = req["path"] as? String
            let windowId = (req["window_id"] as? NSNumber)?.uint32Value
            return try Screenshot.capture(path: path, windowId: windowId)
        case "get_screen_size":
            return Screenshot.screenSize()
        case "cursor":
            return Input.cursor(scale: scale)
        case "move":
            try Input.move(
                x: try requireDouble(req, "x"),
                y: try requireDouble(req, "y"),
                duration: optionalDouble(req, "duration", default: 0),
                scale: scale
            )
        case "click", "double_click":
            try Input.click(
                x: try requireDouble(req, "x"),
                y: try requireDouble(req, "y"),
                button: (req["button"] as? String) ?? "left",
                count: cmd == "double_click" ? 2 : Int(optionalDouble(req, "count", default: 1)),
                duration: optionalDouble(req, "duration", default: 0),
                scale: scale
            )
        case "scroll":
            try Input.scroll(
                deltaX: optionalDouble(req, "delta_x", default: 0),
                deltaY: optionalDouble(req, "delta_y", default: 0),
                x: req["x"] != nil ? try requireDouble(req, "x") : nil,
                y: req["y"] != nil ? try requireDouble(req, "y") : nil,
                scale: scale
            )
        case "type":
            try Input.typeText(try requireString(req, "text"), interval: optionalDouble(req, "interval", default: 0))
        case "key":
            let keys = parseKeys(req["key"] ?? req["keys"])
            try Input.hotkey(keys)
        case "hotkey":
            let keys = parseKeys(req["keys"] ?? req["key"])
            try Input.hotkey(keys)
        case "drag":
            try Input.drag(
                x: try requireDouble(req, "x"),
                y: try requireDouble(req, "y"),
                endX: try requireDouble(req, "end_x"),
                endY: try requireDouble(req, "end_y"),
                duration: optionalDouble(req, "duration", default: 0.2),
                scale: scale
            )
        case "wait":
            Input.wait(optionalDouble(req, "seconds", default: 1))
        case "list_windows":
            return try Windows.list()
        case "get_active_window":
            return try Windows.active()
        case "focus_app":
            try Windows.focusApp(try requireString(req, "app"))
        case "focus_window":
            let windowId = (req["window_id"] as? NSNumber)?.uint32Value
            try Windows.focusWindow(
                app: req["app"] as? String,
                title: req["title"] as? String,
                windowId: windowId
            )
        case "browser_open_profile":
            try Browser.openProfile(try requireString(req, "profile"))
        case "browser_open_url":
            try Browser.openURL(try requireString(req, "url"))
        case "browser_save_page":
            try Browser.savePage(path: try requireString(req, "path"))
        default:
            throw HelperError.unknownCommand(cmd)
        }
        return [:]
    }

    static func parseKeys(_ value: Any?) -> [String] {
        if let arr = value as? [String] { return arr }
        if let s = value as? String {
            return s.split { $0 == "+" || $0 == " " || $0 == "," }.map { String($0) }.filter { !$0.isEmpty }
        }
        return []
    }

    static func argvRequest(_ args: [String]) throws -> [String: Any] {
        if args.first == "--json", args.count >= 2 {
            return try jsonObject(args[1])
        }
        var req: [String: Any] = ["id": "cli", "cmd": args[0]]
        var i = 1
        while i < args.count {
            let a = args[i]
            if a == "--json" {
                req.merge(try jsonObject(args[i + 1])) { _, n in n }
                i += 2
                continue
            }
            if a.hasPrefix("--"), i + 1 < args.count {
                let key = String(a.dropFirst(2)).replacingOccurrences(of: "-", with: "_")
                req[key] = coerce(args[i + 1])
                i += 2
                continue
            }
            if req["cmd"] as? String == "click" || req["cmd"] as? String == "move" || req["cmd"] as? String == "double_click" {
                if req["x"] == nil { req["x"] = coerce(a); i += 1; continue }
                if req["y"] == nil { req["y"] = coerce(a); i += 1; continue }
            }
            i += 1
        }
        return req
    }

    static func coerce(_ raw: String) -> Any {
        if let n = Int(raw) { return n }
        if let n = Double(raw) { return n }
        if raw == "true" { return true }
        if raw == "false" { return false }
        return raw
    }
}

ComputerUseHelperMain.main()
