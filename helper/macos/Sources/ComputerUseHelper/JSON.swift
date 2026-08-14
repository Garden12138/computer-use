import CoreGraphics
import Foundation

enum HelperError: Error {
    case invalidJSON
    case unknownCommand(String)
    case missing(String)
    case permission(String)
    case failed(String)

    var code: String {
        switch self {
        case .invalidJSON: return "invalid_json"
        case .unknownCommand: return "unknown_command"
        case .missing: return "invalid_argument"
        case .permission: return "permission_denied"
        case .failed: return "failed"
        }
    }

    var message: String {
        switch self {
        case .invalidJSON: return "request is not a JSON object"
        case .unknownCommand(let cmd): return "unknown command: \(cmd)"
        case .missing(let name): return "missing argument: \(name)"
        case .permission(let msg): return msg
        case .failed(let msg): return msg
        }
    }
}

func jsonObject(_ raw: String) throws -> [String: Any] {
    guard let data = raw.data(using: .utf8),
          let obj = try JSONSerialization.jsonObject(with: data) as? [String: Any]
    else { throw HelperError.invalidJSON }
    return obj
}

func emit(_ obj: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: obj, options: [])
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data("\n".utf8))
}

func success(id: String, data: [String: Any] = [:]) {
    emit(["id": id, "ok": true, "data": data])
}

func failure(id: String, error: HelperError) {
    emit([
        "id": id,
        "ok": false,
        "error": ["code": error.code, "message": error.message],
    ])
}

func requireDouble(_ req: [String: Any], _ key: String) throws -> Double {
    if let n = req[key] as? Double { return n }
    if let n = req[key] as? Int { return Double(n) }
    if let n = req[key] as? NSNumber { return n.doubleValue }
    throw HelperError.missing(key)
}

func optionalDouble(_ req: [String: Any], _ key: String, default value: Double) -> Double {
    if let n = req[key] as? Double { return n }
    if let n = req[key] as? Int { return Double(n) }
    if let n = req[key] as? NSNumber { return n.doubleValue }
    return value
}

func requireString(_ req: [String: Any], _ key: String) throws -> String {
    guard let s = req[key] as? String else { throw HelperError.missing(key) }
    return s
}

func screenshotToCG(x: Double, y: Double, scale: Double) -> CGPoint {
    let s = scale > 0 ? scale : 1
    return CGPoint(x: x / s, y: y / s)
}
