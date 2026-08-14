import ApplicationServices
import CoreGraphics
import Foundation
import ScreenCaptureKit

enum Permissions {
    static let bundleId = "dev.computeruse.helper"

    static func accessibilityTrusted() -> Bool {
        AXIsProcessTrusted()
    }

    static func screenRecordingTrusted() -> Bool {
        if CGPreflightScreenCaptureAccess() {
            return true
        }
        // Manual toggle in System Settings often does not flip Preflight until
        // this process asks TCC itself.
        CGRequestScreenCaptureAccess()
        if CGPreflightScreenCaptureAccess() {
            return true
        }
        return canCaptureDisplay()
    }

    static func canCaptureDisplay() -> Bool {
        if CGDisplayCreateImage(CGMainDisplayID()) != nil {
            return true
        }
        let sem = DispatchSemaphore(value: 0)
        var ok = false
        SCShareableContent.getExcludingDesktopWindows(false, onScreenWindowsOnly: true) { content, error in
            ok = error == nil && !(content?.displays.isEmpty ?? true)
            sem.signal()
        }
        _ = sem.wait(timeout: .now() + .seconds(3))
        return ok
    }

    static func doctor() -> [String: Any] {
        let ax = accessibilityTrusted()
        let screen = screenRecordingTrusted()
        var hints: [String] = []
        if !ax {
            hints.append(
                "Grant Accessibility to ComputerUseHelper: System Settings → Privacy & Security → Accessibility. Then quit Terminal and rerun."
            )
        }
        if !screen {
            hints.append(
                "Grant Screen Recording to ComputerUseHelper AND to Terminal (or python), then fully quit Terminal and rerun. TCC follows the parent process that spawned the helper."
            )
        }
        return [
            "accessibility": ax,
            "screen_recording": screen,
            "preflight": CGPreflightScreenCaptureAccess(),
            "bundle_id": bundleId,
            "executable": CommandLine.arguments[0],
            "hints": hints,
            "ready": ax && screen,
        ]
    }

    static func requireInput() throws {
        if !accessibilityTrusted() {
            throw HelperError.permission(
                "Accessibility permission is required. Run `computer-use doctor` and grant ComputerUseHelper."
            )
        }
    }

    static func requireScreen() throws {
        if !screenRecordingTrusted() {
            throw HelperError.permission(
                "Screen Recording permission is required. Run `computer-use doctor` and grant ComputerUseHelper."
            )
        }
    }
}
