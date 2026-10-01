import CoreLocation
import Foundation

struct DeviceFix: Sendable {
    let latitude: Double
    let longitude: Double
    let horizontalAccuracyMeters: Double
    let measuredAt: Date
    let reducedAccuracy: Bool
    let simulated: Bool?
    let accessoryProduced: Bool?
}

// One manager per request prevents a late reply from a cancelled manager being
// accepted by a newer request. Core Location does not expose a GNSS-only switch.
@MainActor
final class DeviceLocation: NSObject, @preconcurrency CLLocationManagerDelegate {
    private var manager: CLLocationManager?
    private var deadline: Task<Void, Never>?
    private var completion: ((Result<DeviceFix, Error>) -> Void)?
    private var permissionRequested = false
    private var fixRequested = false
    private var sceneIsActive = true
    var isAwaitingPermission: Bool { manager?.authorizationStatus == .notDetermined }

    func request(precise: Bool, completion: @escaping (Result<DeviceFix, Error>) -> Void) {
        cancel()
        self.completion = completion
        permissionRequested = false
        fixRequested = false
        sceneIsActive = true
        // Global locationServicesEnabled() can synchronously consult the OS.
        // Let authorization/delegate callbacks report availability so this
        // foreground UI path never waits on that synchronous system query.
        let provider = CLLocationManager()
        manager = provider
        provider.delegate = self
        provider.desiredAccuracy = precise ? kCLLocationAccuracyBest : kCLLocationAccuracyHundredMeters
        provider.pausesLocationUpdatesAutomatically = true
        // No always authorization, background mode, continuous update, or network fallback.
        deadline = Task { [weak self, weak provider] in
            do { try await Task.sleep(for: .seconds(30)) } catch { return }
            guard let self, let provider, self.manager === provider else { return }
            self.finish(.failure(ClientError.message("No usable location arrived within 30 seconds. Try outdoors once, or enter coordinates manually.")))
        }
        beginIfAuthorized(provider)
    }

    func cancel() {
        deadline?.cancel()
        deadline = nil
        manager?.stopUpdatingLocation()
        manager?.delegate = nil
        manager = nil
        completion = nil
    }

    func pauseForPermissionPrompt() { sceneIsActive = false }

    func resume() {
        sceneIsActive = true
        if let manager { beginIfAuthorized(manager) }
    }

    private func finish(_ result: Result<DeviceFix, Error>) {
        let callback = completion
        cancel()
        callback?(result)
    }

    private func beginIfAuthorized(_ provider: CLLocationManager) {
        guard manager === provider else { return }
        switch provider.authorizationStatus {
        case .notDetermined:
            if !permissionRequested { permissionRequested = true; provider.requestWhenInUseAuthorization() }
        case .authorizedAlways, .authorizedWhenInUse:
            if sceneIsActive && !fixRequested { fixRequested = true; provider.requestLocation() }
        case .denied, .restricted:
            finish(.failure(ClientError.message("Location permission or Location Services are unavailable. Enter coordinates manually, or check Settings.")))
        @unknown default:
            finish(.failure(ClientError.message("This device reported an unsupported location permission state. Enter coordinates manually.")))
        }
    }

    func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) { beginIfAuthorized(manager) }

    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard self.manager === manager, sceneIsActive else { return }
        let now = Date()
        let valid = locations.filter {
            CLLocationCoordinate2DIsValid($0.coordinate) &&
            $0.horizontalAccuracy.isFinite && $0.horizontalAccuracy >= 0 &&
            now.timeIntervalSince($0.timestamp) >= -5 && now.timeIntervalSince($0.timestamp) <= 60
        }
        guard let fix = valid.min(by: { $0.horizontalAccuracy < $1.horizontalAccuracy }) else {
            finish(.failure(ClientError.message("The device supplied only an invalid or old fix. No coordinates were filled; try once again or enter them manually.")))
            return
        }
        finish(.success(DeviceFix(
            latitude: fix.coordinate.latitude,
            longitude: fix.coordinate.longitude,
            horizontalAccuracyMeters: fix.horizontalAccuracy,
            measuredAt: fix.timestamp,
            reducedAccuracy: manager.accuracyAuthorization == .reducedAccuracy,
            simulated: fix.sourceInformation?.isSimulatedBySoftware,
            accessoryProduced: fix.sourceInformation?.isProducedByAccessory
        )))
    }

    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        guard self.manager === manager else { return }
        // requestLocation is one-shot: its failure ends that attempt. Do not
        // keep a spinner waiting for another fix from a completed request.
        if let locationError = error as? CLError, locationError.code == .locationUnknown {
            finish(.failure(ClientError.message("Apple could not obtain a usable fix for this attempt. Try outdoors once, or enter coordinates manually.")))
            return
        }
        finish(.failure(ClientError.message("This device could not supply a location. Check Location Services or enter coordinates manually.")))
    }
}
