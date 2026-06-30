import AppKit

/// Keeps the Spotlight-findable launcher stub (see Launcher/main.swift) present in
/// an indexed Applications folder, so users can find "Funput" in Spotlight however
/// they installed.
///
/// The input method lives in (~)/Library/Input Methods, which Spotlight does not
/// surface as an app. The .pkg installer drops the launcher into /Applications, but
/// the no-admin .app.zip cannot — so on first launch we copy the launcher embedded
/// in our Resources (by the Xcode "Embed Launcher" build phase) into ~/Applications
/// when none is installed. ~/Applications is Spotlight-indexed and writable without
/// admin rights; the launcher there opens funput://settings back to this process.
enum LauncherInstaller {
    private static let bundleName = "Funput.app"

    /// Copy the embedded launcher into ~/Applications if no launcher is installed
    /// yet. Best-effort and off the main thread — discoverability is a nicety and
    /// must never delay or fail input-method startup.
    static func ensureInstalled() {
        DispatchQueue.global(qos: .utility).async { installIfNeeded() }
    }

    private static func installIfNeeded() {
        let fileManager = FileManager.default

        // Already installed system-wide (.pkg) or per-user? Nothing to do.
        let systemWide = URL(fileURLWithPath: "/Applications/\(bundleName)")
        let userApps = userApplicationsURL.appendingPathComponent(bundleName)
        guard !fileManager.fileExists(atPath: systemWide.path),
              !fileManager.fileExists(atPath: userApps.path)
        else { return }

        // The launcher we ship inside Resources/Funput.app.
        guard let embedded = Bundle.main.url(forResource: "Funput", withExtension: "app") else { return }

        do {
            try fileManager.createDirectory(at: userApplicationsURL, withIntermediateDirectories: true)
            try fileManager.copyItem(at: embedded, to: userApps)
            register(userApps)
        } catch {
            // Leave discoverability unimproved rather than disrupt the input method.
        }
    }

    /// `~/Applications`, falling back to the literal path if the lookup fails.
    private static var userApplicationsURL: URL {
        FileManager.default.urls(for: .applicationDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Applications")
    }

    /// Nudge LaunchServices + Spotlight so the freshly copied launcher is found
    /// without waiting for the next automatic index pass. Best-effort.
    private static func register(_ url: URL) {
        let lsregister = "/System/Library/Frameworks/CoreServices.framework"
            + "/Frameworks/LaunchServices.framework/Support/lsregister"
        run(lsregister, ["-f", url.path])
        run("/usr/bin/mdimport", [url.path])
    }

    private static func run(_ launchPath: String, _ arguments: [String]) {
        guard FileManager.default.isExecutableFile(atPath: launchPath) else { return }
        let process = Process()
        process.executableURL = URL(fileURLWithPath: launchPath)
        process.arguments = arguments
        try? process.run()
    }
}
