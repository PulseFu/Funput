import InputMethodKit
import SwiftUI

enum WindowID {
    static let settings = "settings"
    static let onboarding = "onboarding"
}

/// Hosts the IMKServer so the bundle works as a system input method. The same
/// process also renders the SwiftUI menu bar / Settings scenes below.
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var server: IMKServer?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let connectionName = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String
            ?? "Funput_1_Connection"
        server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
        // Make "Funput" findable in Spotlight on no-admin installs (see type docs).
        LauncherInstaller.ensureInstalled()
    }

    /// Handle `funput://` URLs. The /Applications launcher stub opens
    /// `funput://settings` so users who find "Funput" in Spotlight reach Settings
    /// (the input method bundle itself isn't surfaced by Spotlight). Bumping the
    /// request is observed by the menu bar label, which opens the window — this
    /// works whether we were already running or were just cold-launched by the URL.
    func application(_ application: NSApplication, open urls: [URL]) {
        guard urls.contains(where: { $0.scheme == "funput" }) else { return }
        AppSettings.shared.openSettingsRequest &+= 1
        NSApp.activate(ignoringOtherApps: true)
    }
}

@main
struct FunputApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var settings = AppSettings.shared
    @State private var updater = UpdaterManager()

    init() {
        // Funput is a Vietnamese input method and its own UI is hardcoded Vietnamese,
        // so force the app's locale to Vietnamese. Without this, Sparkle's standard
        // update dialogs follow the *system* language and show English on the many
        // Macs that run in English — this makes them resolve to Sparkle's bundled
        // vi.lproj instead. Sparkle's UI is shown on demand (well after launch), so
        // setting this here is early enough; the value also persists for next launch.
        UserDefaults.standard.set(["vi"], forKey: "AppleLanguages")
    }

    var body: some Scene {
        MenuBarExtra(isInserted: menuBarBinding) {
            StatusMenu()
                .environment(settings)
                .environment(updater)
        } label: {
            MenuBarLabel()
                .environment(settings)
        }

        Window("Funput Settings", id: WindowID.settings) {
            SettingsView()
                .environment(settings)
                .environment(updater)
                .frame(minWidth: Theme.settingsMinWidth, minHeight: Theme.settingsMinHeight)
        }
        .windowResizability(.contentSize)
        .defaultSize(width: Theme.settingsMinWidth, height: Theme.settingsMinHeight)

        Window("Chào mừng đến Funput", id: WindowID.onboarding) {
            OnboardingView()
                .environment(settings)
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)
    }

    private var menuBarBinding: Binding<Bool> {
        Binding(get: { settings.showMenuBarIcon }, set: { settings.showMenuBarIcon = $0 })
    }
}

/// The menu bar icon. Renders at launch, so it also kicks off first-run onboarding.
private struct MenuBarLabel: View {
    @Environment(AppSettings.self) private var settings
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        // "VI" when composing Vietnamese, "EN" when passing through.
        Text(settings.vietnameseEnabled ? "VI" : "EN")
            .task {
                if !settings.hasCompletedOnboarding {
                    openWindow(id: WindowID.onboarding)
                }
            }
            // A funput://settings request (from the /Applications launcher) opens
            // Settings. `initial: true` also covers a cold launch where the URL
            // bumped the counter before this label first appeared.
            .onChange(of: settings.openSettingsRequest, initial: true) { _, count in
                guard count > 0 else { return }
                openWindow(id: WindowID.settings)
                NSApp.activate(ignoringOtherApps: true)
            }
    }
}
