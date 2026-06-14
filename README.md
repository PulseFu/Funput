# Funput Crates

Rust workspace chứa **toàn bộ logic lõi** của Funput — bộ gõ tiếng Việt đa nền tảng. Các crate ở đây **không phụ thuộc UI** và **không phụ thuộc platform hook** (CGEventTap, Fcitx5, Windows hook). Platform shell trong `platforms/` gọi vào các crate này qua FFI hoặc link trực tiếp.

## Mục tiêu thiết kế

- **Một engine duy nhất** cho macOS, Linux, Windows
- **Pure logic** tách khỏi inject / keyboard hook
- **Testable** — chạy unit test không cần OS IME hay Accessibility
- **Nhẹ** — rule-based Telex/VNI, không runtime nặng

## Danh sách crate

| Crate | Vai trò | Doc |
|-------|---------|-----|
| [`funput-core`](./funput-core/README.md) | Logic chuyển đổi Telex/VNI, validation, Unicode | Chi tiết ↓ |
| [`funput-engine`](./funput-engine/README.md) | Session, buffer, pipeline key → `ImeResult` | Chi tiết ↓ |
| [`funput-ffi`](./funput-ffi/README.md) | C ABI cho Swift / native bridge | Chi tiết ↓ |
| [`funput-cli`](./funput-cli/README.md) | Binary dev/test engine từ terminal | Chi tiết ↓ |

## Luồng phụ thuộc

```
funput-core
     ↓
funput-engine
     ↓
funput-ffi ──→ platforms/macos, platforms/windows
     ↓
funput-cli (dev tool)
```

| Crate | Phụ thuộc |
|-------|-----------|
| `funput-core` | Không phụ thuộc crate Funput khác |
| `funput-engine` | `funput-core` |
| `funput-ffi` | `funput-engine` |
| `funput-cli` | `funput-engine` |

## Luồng dữ liệu (tổng quan)

```
Key event (platform)
       ↓
  funput-ffi::ime_key()          ← platforms gọi qua C ABI
       ↓
  funput-engine                  ← buffer, session, word boundary
       ↓
  funput-core                    ← Telex/VNI transform + validation
       ↓
  ImeResult { action, backspace, chars }
       ↓
  Platform inject layer          ← Backspace / Selection / AX-sync (ngoài crates/)
```

## Ranh giới trách nhiệm

| Thuộc `crates/` | **Không** thuộc `crates/` |
|-----------------|---------------------------|
| Telex, VNI, validation | CGEventTap, Fcitx5, Windows hook |
| Buffer & session state | Inject text vào app (Backspace, Unicode events) |
| `ImeResult` struct | Menu bar, settings UI |
| C ABI export | Per-app auto-detect (AX role, bundle ID) |
| Unit/integration tests logic | Dictionary lớn (phase 2, nếu có) |

## Workspace members (dự kiến)

```toml
[workspace]
members = [
    "crates/funput-core",
    "crates/funput-engine",
    "crates/funput-ffi",
    "crates/funput-cli",
]
```

Linux Fcitx5 engine (`platforms/linux/fcitx5-funput/`) link trực tiếp `funput-engine` — có thể nằm ngoài hoặc trong workspace tùy cách tổ chức build.

## Thứ tự triển khai

1. `funput-core` — Telex converter + tests
2. `funput-engine` — session + `ImeResult`
3. `funput-cli` — test end-to-end từ terminal
4. `funput-ffi` — C ABI khi bắt đầu `platforms/macos` hoặc `platforms/windows`
