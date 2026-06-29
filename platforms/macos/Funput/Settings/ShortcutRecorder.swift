import AppKit
import SwiftUI

/// Records a keyboard shortcut in the app's Liquid Glass style: tap the pill, then
/// press the combo you want. Binds to an optional `KeyCombo` (`nil` = off). Requires
/// a ⌃/⌥/⌘ modifier; Esc cancels.
struct ShortcutRecorder: View {
    @Binding var combo: KeyCombo?

    @State private var recording = false
    @State private var monitor: Any?

    var body: some View {
        HStack(spacing: Theme.Spacing.sm) {
            Button(action: toggleRecording) {
                pill
            }
            .buttonStyle(.plain)

            if combo != nil, !recording {
                Button { combo = nil } label: {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 15))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("Tắt phím tắt")
            }
        }
        .animation(.easeInOut(duration: 0.15), value: recording)
        .onDisappear(perform: stopRecording)
    }

    /// The tappable pill: prompt when empty, the recorded caps when set, a tinted
    /// "listening" state while recording.
    @ViewBuilder private var pill: some View {
        if recording {
            glassPill(
                Label("Nhấn phím…", systemImage: "record.circle"),
                glass: .regular.tint(.accentColor).interactive()
            )
        } else if let combo {
            glassPill(ShortcutCaps(caps: combo.keyCaps), glass: .regular.interactive())
        } else {
            glassPill(
                Label("Đặt phím", systemImage: "keyboard"),
                glass: .regular.interactive()
            )
        }
    }

    private func glassPill(_ content: some View, glass: Glass) -> some View {
        content
            .font(.callout.weight(.medium))
            .foregroundStyle(.primary)
            .padding(.horizontal, Theme.Spacing.md)
            .padding(.vertical, Theme.Spacing.sm)
            .glassEffect(glass, in: .capsule)
            .contentShape(.capsule)
    }

    // MARK: - Recording

    private func toggleRecording() {
        recording ? stopRecording() : startRecording()
    }

    private func startRecording() {
        recording = true
        // Swallow keys (return nil) while recording so they don't leak to the UI.
        monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            if event.keyCode == 53 { // Esc cancels
                stopRecording()
            } else if let captured = KeyCombo.from(event) {
                combo = captured
                stopRecording()
            }
            return nil
        }
    }

    private func stopRecording() {
        recording = false
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
    }
}
