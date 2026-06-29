import SwiftUI

struct KeyboardPane: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        @Bindable var settings = settings

        VStack(alignment: .leading, spacing: Theme.Spacing.lg) {
            GlassCard {
                VStack(alignment: .leading, spacing: Theme.Spacing.md) {
                    SettingsRow(
                        title: "Phím chuyển Việt / Anh",
                        subtitle: "Nhấn để tạm tắt gõ tiếng Việt",
                        systemImage: "globe"
                    ) {
                        ShortcutCaps(caps: settings.toggleShortcut.keyCaps)
                    }
                    Divider()
                    Picker("Chọn phím", selection: $settings.toggleShortcut) {
                        ForEach(ToggleShortcut.allCases) { shortcut in
                            Text(shortcut.label).tag(shortcut)
                        }
                    }
                    .pickerStyle(.radioGroup)
                    .labelsHidden()
                }
            }

            GlassCard {
                SettingsRow(
                    title: "Phím lật từ vừa gõ",
                    subtitle: "Đổi từ đang gõ giữa tiếng Việt và chữ gốc (card ⇄ cải). Bấm rồi nhấn tổ hợp kèm ⌃/⌥/⌘.",
                    systemImage: "arrow.2.squarepath"
                ) {
                    ShortcutRecorder(combo: $settings.flipShortcut)
                }
            }
        }
    }
}

#Preview {
    KeyboardPane()
        .environment(AppSettings.shared)
        .padding(Theme.Spacing.xl)
        .frame(width: 520)
}
