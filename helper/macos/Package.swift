// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ComputerUseHelper",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "computer-use-helper",
            path: "Sources/ComputerUseHelper",
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("ApplicationServices"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("ScreenCaptureKit"),
            ]
        ),
    ]
)
