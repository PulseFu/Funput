// Word-boundary test shared by the Linux input-method shells (Fcitx5, IBus).
// A boundary ends the composing word and triggers commit / English-restore.

#ifndef FUNPUT_BOUNDARY_H
#define FUNPUT_BOUNDARY_H

#include <cstdint>

namespace funput {

// ASCII whitespace or punctuation; digits excluded (VNI uses them as tone
// modifiers). Mirrors funput_core's rule and the macOS shell.
inline bool isBoundary(char32_t s) {
    if (s > 0x7F) return false;
    if (s == U' ' || s == U'\t' || s == U'\n' || s == U'\r') return true;
    const uint32_t v = static_cast<uint32_t>(s);
    return (v >= 0x21 && v <= 0x2F) || (v >= 0x3A && v <= 0x40) ||
           (v >= 0x5B && v <= 0x60) || (v >= 0x7B && v <= 0x7E);
}

} // namespace funput

#endif // FUNPUT_BOUNDARY_H
