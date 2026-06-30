import AppKit

// Funput launcher — a tiny Spotlight-findable stub installed in /Applications.
//
// The real input method lives in /Library/Input Methods as an LSUIElement IMK
// agent, which macOS does not surface in Spotlight's app results. So a user who
// types "Funput" into Spotlight after installing would otherwise find nothing and
// not know how to open Funput's Settings. This stub exists only to be findable:
// launching it asks the input method to show its Settings window via the
// funput:// URL scheme (handled in FunputApp.swift), then exits. It has no UI.
//
// LaunchServices resolves funput:// to the input method bundle regardless of its
// /Library/Input Methods location, launching it if needed or delivering the URL
// to the already-running process.
let url = URL(string: "funput://settings")!
NSWorkspace.shared.open(url)
