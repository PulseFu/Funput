# funput-ffi

Crate **C ABI boundary** — export API ổn định cho platform native (Swift trên macOS, C#/C++ trên Windows) gọi `funput-engine` mà không cần viết Rust ở phía UI.

## Ý nghĩa

Rust engine chạy trong `.dylib` / `.dll` / `.so`. Platform shell (Swift, C#) load library và gọi hàm C — đây là cầu nối giữa **Rust core** và **native hook/inject layer**.

Linux Fcitx5 có thể link `funput-engine` trực tiếp và **không cần** crate này.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Export `extern "C"` functions | Logic Telex/VNI |
| Chuyển `ImeResult` → struct C (`#[repr(C)]`) | CGEventTap, keyboard hook |
| Quản lý vòng đời init / free result | Settings UI |
| Thread-safe singleton engine (nếu cần) | Inject text vào app |
| `cbindgen` / header generation | Fcitx5 integration |

## API dự kiến (C ABI)

```c
// Initialize once at app startup
void ime_init(void);

// Process one keystroke
ImeResult* ime_key(uint16_t keycode, bool caps, bool ctrl);

// Configuration
void ime_method(uint8_t method);   // 0=Telex, 1=VNI
void ime_enabled(bool enabled);
void ime_clear(void);              // Word boundary

// Caller must free every result from ime_key
void ime_free(ImeResult* result);
```

```c
typedef struct {
    uint32_t chars[32];
    uint8_t  action;      // 0=None, 1=Send, 2=Restore
    uint8_t  backspace;
    uint8_t  count;
    uint8_t  _pad;
} ImeResult;
```

## Luồng trên macOS (ví dụ)

```
CGEventTap callback (Swift)
       ↓
RustBridge.processKey(keycode, flags)
       ↓
ime_key()                    ← funput-ffi
       ↓
funput-engine
       ↓
ImeResult* → Swift đọc backspace + chars
       ↓
Inject layer (Backspace / AX-sync)   ← ngoài funput-ffi
       ↓
ime_free(result)
```

## Memory ownership

| Bên | Trách nhiệm |
|-----|------------|
| Rust (`ime_key`) | Allocate `ImeResult` |
| Caller (Swift/C#) | Gọi `ime_free()` đúng một lần mỗi result |
| Rust (`ime_init`) | Init engine một lần, thread-safe |

Caller **phải** free — thường dùng `defer ime_free(ptr)` trong Swift.

## Cấu trúc module (dự kiến)

```
funput-ffi/src/
├── lib.rs                # extern "C" exports
├── types.rs              # #[repr(C)] ImeResult
└── include/
    └── funput.h          # Generated via cbindgen
```

## Build output

| Platform | Artifact |
|----------|----------|
| macOS | `libfunput_ffi.dylib` + `funput.h` |
| Windows | `funput_ffi.dll` + `.lib` |
| Linux | Không bắt buộc (Fcitx5 link trực tiếp engine) |

Script build trong `platforms/macos/scripts/build-rust.sh` compile crate này với target phù hợp.

## Phụ thuộc

```
funput-ffi → funput-engine → funput-core
```

## Ai gọi crate này?

| Consumer | Ghi chú |
|----------|---------|
| `platforms/macos/Funput/Bridge/` | Swift qua bridging header |
| `platforms/windows/` | P/Invoke hoặc C++ link |
| **Không** | `funput-cli`, Fcitx5 (link engine trực tiếp) |

## Tests

FFI layer nên có test round-trip:

```bash
cargo test -p funput-ffi
```

Kiểm tra: gọi `ime_key` qua C ABI → parse result → `ime_free` không leak.
