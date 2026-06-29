import AppKit
import SwiftUI

struct AboutPane: View {
    // Optional so SwiftUI previews (which don't inject the manager) still render.
    @Environment(UpdaterManager.self) private var updater: UpdaterManager?

    var body: some View {
        VStack(spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(spacing: Theme.Spacing.md) {
                    logo
                        .frame(width: 92, height: 92)
                        .padding(Theme.Spacing.md)
                        .glassEffect(.regular, in: .circle)
                        .padding(.bottom, Theme.Spacing.xs)

                    VStack(spacing: Theme.Spacing.xs) {
                        Text("Funput")
                            .font(.largeTitle.bold())
                        Text("Bộ gõ tiếng Việt — miễn phí, mã nguồn mở.")
                            .font(.callout)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                    }

                    Text("Phiên bản \(appVersion)")
                        .font(.caption.weight(.medium))
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, Theme.Spacing.md)
                        .padding(.vertical, Theme.Spacing.xs)
                        .glassEffect(.regular, in: .capsule)

                    VStack(spacing: Theme.Spacing.sm) {
                        if let updater {
                            Button { updater.checkForUpdates() } label: {
                                Label("Kiểm tra cập nhật", systemImage: "arrow.triangle.2.circlepath")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.glassProminent)
                            .controlSize(.large)
                            .disabled(!updater.canCheckForUpdates)
                        }

                        HStack(spacing: Theme.Spacing.sm) {
                            linkButton(
                                "GitHub",
                                systemImage: "chevron.left.forwardslash.chevron.right",
                                url: "https://github.com/Funput/Funput"
                            )
                            linkButton("Website", systemImage: "globe", url: "https://funput.app/")
                        }
                    }
                    .padding(.top, Theme.Spacing.sm)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, Theme.Spacing.md)
            }
        }
    }

    /// The app icon (Funput logo); falls back to an SF Symbol if unavailable
    /// (e.g. in previews).
    @ViewBuilder private var logo: some View {
        if let icon = NSApplication.shared.applicationIconImage {
            Image(nsImage: icon)
                .resizable()
                .interpolation(.high)
                .scaledToFit()
        } else {
            Image(systemName: "character.bubble.fill")
                .resizable()
                .scaledToFit()
                .foregroundStyle(.tint)
                .padding(Theme.Spacing.sm)
        }
    }

    /// A secondary glass link button with a leading icon, sized to share its row.
    private func linkButton(_ title: String, systemImage: String, url: String) -> some View {
        Link(destination: URL(string: url)!) {
            Label(title, systemImage: systemImage)
                .frame(maxWidth: .infinity)
        }
        .buttonStyle(.glass)
        .controlSize(.large)
    }

    private var appVersion: String {
        Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
    }
}

#Preview {
    AboutPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
