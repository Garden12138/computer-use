import AppKit
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

enum Screenshot {
    static func capture(path: String?, windowId: UInt32?) throws -> [String: Any] {
        try Permissions.requireScreen()
        let scale = Double(NSScreen.main?.backingScaleFactor ?? 1.0)
        let image: CGImage
        if let windowId {
            let opts = CGWindowImageOption.boundsIgnoreFraming.union(.bestResolution)
            guard let captured = CGWindowListCreateImage(
                .null,
                .optionIncludingWindow,
                windowId,
                opts
            ) else {
                throw HelperError.failed("could not capture window \(windowId)")
            }
            image = captured
        } else {
            guard let captured = CGDisplayCreateImage(CGMainDisplayID()) else {
                throw HelperError.failed("could not capture main display")
            }
            image = captured
        }
        let outPath = path ?? FileManager.default.temporaryDirectory
            .appendingPathComponent("computer-use-\(Int(Date().timeIntervalSince1970 * 1000)).png")
            .path
        try writePNG(image, to: outPath)
        let display = NSScreen.main?.frame ?? .zero
        var payload: [String: Any] = [
            "path": outPath,
            "width": image.width,
            "height": image.height,
            "scale": scale,
            "display_id": CGMainDisplayID(),
            "display_width_points": display.width,
            "display_height_points": display.height,
        ]
        if let windowId {
            payload["window_id"] = windowId
            if let bounds = windowBounds(windowId) {
                payload["origin_x"] = bounds.origin.x
                payload["origin_y"] = bounds.origin.y
                payload["bounds_width"] = bounds.size.width
                payload["bounds_height"] = bounds.size.height
            }
        }
        return payload
    }

    private static func windowBounds(_ windowId: UInt32) -> CGRect? {
        let options = CGWindowListOption.optionIncludingWindow.union(.excludeDesktopElements)
        let infos = CGWindowListCopyWindowInfo(options, windowId) as? [[String: Any]] ?? []
        guard let info = infos.first(where: {
            ($0[kCGWindowNumber as String] as? NSNumber)?.uint32Value == windowId
        }) else {
            return nil
        }
        let bounds = info[kCGWindowBounds as String] as? [String: Any] ?? [:]
        return CGRect(
            x: (bounds["X"] as? NSNumber)?.doubleValue ?? 0,
            y: (bounds["Y"] as? NSNumber)?.doubleValue ?? 0,
            width: (bounds["Width"] as? NSNumber)?.doubleValue ?? 0,
            height: (bounds["Height"] as? NSNumber)?.doubleValue ?? 0
        )
    }

    static func screenSize() -> [String: Any] {
        let scale = Double(NSScreen.main?.backingScaleFactor ?? 1.0)
        let frame = NSScreen.main?.frame ?? .zero
        return [
            "width_points": frame.width,
            "height_points": frame.height,
            "width": frame.width * scale,
            "height": frame.height * scale,
            "scale": scale,
        ]
    }

    private static func writePNG(_ image: CGImage, to path: String) throws {
        let url = URL(fileURLWithPath: path)
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        guard let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else {
            throw HelperError.failed("could not open \(path)")
        }
        CGImageDestinationAddImage(dest, image, nil)
        if !CGImageDestinationFinalize(dest) {
            throw HelperError.failed("could not write PNG \(path)")
        }
    }
}
