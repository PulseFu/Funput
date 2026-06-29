import AppKit

/// A user-recorded keyboard shortcut: a key plus its ⌃/⌥/⌘/⇧ modifiers.
///
/// Matching is by `keyCode` + the exact masked modifier set (layout-independent).
/// `label` is the display string captured when recorded, so we don't need a
/// keyCode→name table for every layout.
struct KeyCombo: Codable, Equatable {
    let keyCode: UInt16
    /// Raw value of the masked `NSEvent.ModifierFlags` (⌃⌥⌘⇧ only).
    let modifiers: UInt
    let label: String

    /// The modifier keys that make a combo a real shortcut (Shift alone doesn't —
    /// it's part of normal typing).
    private static let commandModifiers: NSEvent.ModifierFlags = [.control, .option, .command]
    private static let allModifiers: NSEvent.ModifierFlags = [.control, .option, .command, .shift]

    /// Build a combo from a key-down event, or `nil` when it isn't a usable shortcut
    /// (no ⌃/⌥/⌘ held — a bare key would fire while typing).
    static func from(_ event: NSEvent) -> KeyCombo? {
        let mods = event.modifierFlags.intersection(allModifiers)
        guard !mods.intersection(commandModifiers).isEmpty else { return nil }
        return KeyCombo(keyCode: event.keyCode, modifiers: mods.rawValue, label: label(for: event))
    }

    /// True when `event` is exactly this combo.
    func matches(_ event: NSEvent) -> Bool {
        event.keyCode == keyCode
            && event.modifierFlags.intersection(Self.allModifiers).rawValue == modifiers
    }

    /// Keycaps for display, e.g. `["⌃", "⇧", "Z"]`.
    var keyCaps: [String] {
        let flags = NSEvent.ModifierFlags(rawValue: modifiers)
        var caps: [String] = []
        if flags.contains(.control) { caps.append("⌃") }
        if flags.contains(.option) { caps.append("⌥") }
        if flags.contains(.shift) { caps.append("⇧") }
        if flags.contains(.command) { caps.append("⌘") }
        caps.append(label)
        return caps
    }

    /// Human-readable label for the pressed key, preferring the produced character.
    private static func label(for event: NSEvent) -> String {
        if let chars = event.charactersIgnoringModifiers,
           let first = chars.first,
           first.isLetter || first.isNumber || first.isPunctuation || first.isSymbol {
            return chars.uppercased()
        }
        return specialKeyNames[event.keyCode] ?? "•"
    }

    /// Display names for common non-printing keys (fallback when there's no character).
    private static let specialKeyNames: [UInt16: String] = [
        49: "Space", 36: "↩", 48: "⇥", 51: "⌫", 53: "⎋",
        123: "←", 124: "→", 125: "↓", 126: "↑",
    ]
}
