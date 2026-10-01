import Foundation

enum EngineOperation: UInt32, Sendable {
    case inventory = 0
    case lookup = 1
    case clock = 2
    case calculate = 3
}

struct RustEngine: Sendable {
    enum Failure: LocalizedError {
        case unsupportedABI, requestTooLarge, unavailable, invalidResponse
        var errorDescription: String? {
            switch self {
            case .unsupportedABI: "The bundled engine interface is incompatible."
            case .requestTooLarge: "The request is too large for the bundled engine."
            case .unavailable: "The bundled engine could not return a result."
            case .invalidResponse: "The bundled engine returned an unreadable result."
            }
        }
    }

    func execute(_ operation: EngineOperation, request: Data = Data()) throws -> Data {
        guard salah_native_abi_version() == 1 else { throw Failure.unsupportedABI }
        guard request.count <= 8192 else { throw Failure.requestTooLarge }
        let response = request.withUnsafeBytes { bytes in
            salah_execute(operation.rawValue, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count)
        }
        guard let response else { throw Failure.unavailable }
        defer { salah_response_free(response) }
        let count = salah_response_length(response)
        guard count > 0, count <= 1_048_576,
              let pointer = salah_response_data(response) else { throw Failure.invalidResponse }
        return Data(bytes: pointer, count: count) // Copy before freeing Rust ownership.
    }
}
