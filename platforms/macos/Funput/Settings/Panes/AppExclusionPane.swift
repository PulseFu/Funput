import AppKit
import SwiftUI
import UniformTypeIdentifiers

/// Manage the per-app exclusion list: apps where Funput stays out of the way and
/// keys pass straight through as English. Add via a file picker over `/Applications`
/// (the Settings window is frontmost, so "current app" wouldn't be meaningful here).
struct AppExclusionPane: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.md) {
                    SettingsRow(
                        title: "Ứng dụng bỏ qua",
                        subtitle: "Mặc định tiếng Anh khi vào các app này — vẫn bật lại tiếng Việt bằng phím tắt",
                        systemImage: "nosign"
                    ) {
                        Button(action: pickApp) {
                            Label("Thêm", systemImage: "plus")
                        }
                    }

                    if settings.excludedApps.isEmpty {
                        Divider()
                        Text("Chưa có app nào. Bấm “Thêm” để chọn app cần bỏ qua tiếng Việt.")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        ForEach(settings.excludedApps) { app in
                            Divider()
                            appRow(app)
                        }
                    }
                }
            }
        }
    }

    private func appRow(_ app: ExcludedApp) -> some View {
        HStack(spacing: Theme.Spacing.md) {
            Image(nsImage: icon(for: app.id))
                .resizable()
                .frame(width: 24, height: 24)
            VStack(alignment: .leading, spacing: 2) {
                Text(app.name).font(.body)
                Text(app.id).font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: Theme.Spacing.md)
            Button {
                settings.removeExcludedApp(app.id)
            } label: {
                Image(systemName: "trash")
            }
            .buttonStyle(.borderless)
            .foregroundStyle(.secondary)
            .help("Bỏ khỏi danh sách")
        }
        .padding(.vertical, Theme.Spacing.xs)
    }

    // MARK: - Actions

    private func pickApp() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.application]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.directoryURL = URL(fileURLWithPath: "/Applications")
        panel.prompt = "Thêm"
        panel.message = "Chọn app để bỏ qua tiếng Việt"

        guard panel.runModal() == .OK,
              let url = panel.url,
              let id = Bundle(url: url)?.bundleIdentifier
        else { return }

        let name = FileManager.default.displayName(atPath: url.path)
            .replacingOccurrences(of: ".app", with: "")
        settings.addExcludedApp(ExcludedApp(id: id, name: name))
    }

    private func icon(for bundleId: String) -> NSImage {
        if let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) {
            return NSWorkspace.shared.icon(forFile: url.path)
        }
        return NSImage(systemSymbolName: "app.dashed", accessibilityDescription: nil) ?? NSImage()
    }
}

#Preview {
    AppExclusionPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
