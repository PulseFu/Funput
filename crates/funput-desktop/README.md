# funput-desktop

Logic **thuần, không phụ thuộc OS** cho các shell kiểu **"hook + inject"** (chặn phím toàn cục rồi
tự gõ chữ ra — như Windows `WH_KEYBOARD_LL`). Crate này nằm giữa hai phần phụ thuộc nền tảng:

```
[host đọc phím thô] → funput-desktop::classify → engine → funput-desktop::plan_inject → [host inject]
        OS-specific              pure                        pure                  OS-specific
```

Mọi quyết định **"phím này nghĩa là gì"** và **"cần xoá/gõ gì"** sống ở đây nên unit-test được mà
không cần API hệ điều hành nào. (Cùng mô hình với `result_bytes` của `funput-term`, nhưng sinh ra một
*plan trung tính* thay vì byte terminal.)

> Chỉ shell **hook + inject** dùng crate này — hiện là **Windows** (`platforms/windows/src-tauri`).
> macOS (IMKit marked text) và Linux (Fcitx5 preedit) đi mô hình khác, **không** dùng.

## Hai nửa của crate

**1. Đầu vào — `classify(&KeyEvent) -> KeyKind`:** một phím nên làm gì khi đang bật tiếng Việt.

```rust
pub struct Mods { pub ctrl: bool, pub alt: bool, pub win: bool, pub shift: bool }
// is_shortcut() = ctrl || alt || win  (Shift KHÔNG tính — vẫn là gõ thường)

pub struct KeyEvent {
    pub mods: Mods,
    pub ch: Option<char>,   // ký tự phím tạo ra (Windows: từ ToUnicodeEx), nếu có
    pub is_backspace: bool,
    pub is_navigation: bool, // mũi tên, Home/End, PageUp/Down, Esc, Delete, F-keys, Enter, Tab
}

pub enum KeyKind {
    Compose(char), // nạp ký tự cho engine (kể cả space/dấu câu — engine tự quyết ranh giới từ)
    Backspace,     // gọi engine.on_backspace() và áp kết quả
    Flush,         // commit/clear composition rồi để phím đi qua (navigation / phím tắt hệ thống)
    PassThrough,   // phím vô nghĩa (không ký tự, không navigation) — bỏ qua, giữ nguyên composition
}
```

Thứ tự quyết định trong `classify`: phím tắt (ctrl/alt/win) → `Flush`; Backspace → `Backspace`;
navigation → `Flush`; có `ch` → `Compose(ch)`; còn lại → `PassThrough`. Toggle VI/EN do host xử lý
**trước** `classify` (tổ hợp toggle cấu hình được và tuỳ host).

**2. Đầu ra — `plan_inject(&ImeResult) -> InjectPlan`:** dịch kết quả engine thành thao tác inject.

```rust
pub struct InjectPlan {
    pub backspaces: usize,  // số ký tự cần xoá lùi
    pub units: Vec<u16>,    // UTF-16 code units để gõ sau khi xoá
}
// is_noop() = backspaces == 0 && units.is_empty()
```

- `Action::None` → plan rỗng (`is_noop`): để phím tới app nguyên vẹn.
- `Action::Send` / `Action::Restore` → xoá `backspace` ký tự rồi gõ `output`; host nuốt phím gốc.

`units` là **UTF-16** vì đó là thứ Windows `SendInput` (`KEYEVENTF_UNICODE`) nhận; chữ Việt NFC nằm
trong BMP (mỗi ký tự một unit), surrogate pair vẫn đúng cho text khác.

## Host nối dây thế nào (ví dụ Windows)

```
WH_KEYBOARD_LL callback
  ├─ tổ hợp toggle? → bật/tắt VI/EN, nuốt phím        (trước classify)
  └─ dựng KeyEvent (mods, ToUnicodeEx → ch, backspace/navigation)
       match classify(&ev):
         Compose(c)  → plan_inject(engine.process_char(c)) → SendInput(plan); nuốt phím
         Backspace   → engine.on_backspace(); để Backspace vật lý đi qua (app tự xoá)
         Flush       → engine.clear(); để phím đi qua
         PassThrough → để phím đi qua
```

`InjectPlan` chỉ **mô tả** việc cần làm; *cách* gửi (`SendInput` với `INJECT_TAG` chống đệ quy) là
việc của host trong `platforms/windows`.

## Phạm vi & ranh giới

| funput-desktop | Host (platforms/windows) |
|----------------|--------------------------|
| `classify` (phím → ý nghĩa) | Đọc phím thô (LL hook), map keycode → `KeyEvent` |
| `plan_inject` (`ImeResult` → `InjectPlan`) | Gửi Backspace/Unicode (`SendInput`), chống đệ quy |
| Pure, OS-neutral, unit-test được | Toggle hotkey, per-app VI/EN, tray, settings |

## Phụ thuộc & ai gọi

- Phụ thuộc: chỉ `funput-engine` (dùng `Action`, `ImeResult`). Không OS API, không `serde`.
- Consumer: `platforms/windows/src-tauri` (hook + SendInput). Mở cho bất kỳ host hook+inject nào khác.

## Tests

```bash
cargo test   -p funput-desktop
cargo clippy -p funput-desktop -- -D warnings
```

Test trong `src/lib.rs`: `classify` (printable/shortcut/shift/backspace/navigation/no-char) và
`plan_inject` (None no-op, Send xoá+gõ UTF-16, Restore cả từ).
