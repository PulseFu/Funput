# funput-engine

Crate **điều phối** — nhận sự kiện phím theo thời gian, giữ buffer, gọi `funput-core`, trả kết quả cho platform inject.

## Ý nghĩa

`funput-core` trả lời “chuỗi này transform thành gì”.  
`funput-engine` trả lời **“sau key này, platform cần làm gì?”**

Đây là **single source of truth** cho trạng thái gõ: buffer đang composition, kiểu gõ đang chọn, bật/tắt, ranh giới từ.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Session / buffer theo input context | Hook keyboard (CGEventTap, …) |
| Gọi `funput-core` khi có key mới | Inject Backspace / Unicode vào app |
| Tính `backspace` count (buffer cũ vs mới) | UI settings, menu bar |
| Trả `ImeResult` cho platform | Logic Telex/VNI chi tiết |
| Word boundary (`ime_clear` khi Space, Enter) | C ABI export (thuộc `funput-ffi`) |
| Bật/tắt engine, đổi Telex/VNI | Auto-restore tiếng Anh (có thể ở đây hoặc module riêng trong engine) |
| Shortcut / gõ tắt (phase 2) | Đọc file config (platform hoặc crate riêng sau) |

## `ImeResult` — contract với platform

Struct trung tâm mà mọi platform shell consume:

```rust
pub enum Action {
    None,    // Pass key through — không transform
    Send,    // Transform — platform phải inject
    Restore, // Hoàn nguyên buffer (ví dụ ESC)
}

pub struct ImeResult {
    pub action: Action,
    pub backspace: usize,    // Số ký tự cần xóa trong app
    pub output: String,      // Chuỗi inject sau khi xóa
}
```

`ImeResult` là kiểu Rust-native. `funput-ffi` mới marshal sang struct `#[repr(C)]`
(`backspace: u8`, `chars: [u32; 32]`, `count: u8`) ở biên FFI — giới hạn 32 ký tự /
`u8` và chính sách tràn nằm ở đó, không ở engine.

Platform đọc `ImeResult` rồi quyết định **cách inject** (Backspace, Selection, AX-sync) — logic inject **không** nằm trong crate này.

## Luồng xử lý một phím

```
1. Platform gọi engine.process_key(keycode, modifiers)
2. Engine cập nhật buffer
3. Engine gọi funput-core transform
4. Engine so sánh buffer trước / sau → tính backspace + output
5. Trả ImeResult
```

### Ví dụ: Telex `a` → `s` → `á`

| Bước | Key | Action | Backspace | Output |
|------|-----|--------|-----------|--------|
| 1 | `a` | `None` | 0 | — (chờ thêm key) |
| 2 | `s` | `Send` | 1 | `á` |

Platform nhận bước 2: xóa 1 ký tự, inject `á`, nuốt key `s`.

## Cấu trúc module (hiện tại — E0)

```
funput-engine/src/
├── lib.rs                # Engine, re-exports
├── result.rs             # Action, ImeResult
├── session.rs            # enabled, method, buffer
├── pipeline.rs           # stub — E1
└── diff.rs               # stub — E1
```

Integration tests (`tests/telex_steps.rs`, …) — E1+.

## Phụ thuộc

```
funput-engine → funput-core
```

## Ai gọi crate này?

| Consumer | Cách gọi |
|----------|----------|
| `funput-ffi` | Wrap API C cho Swift / native |
| `funput-cli` | Test trực tiếp từ terminal |
| `platforms/linux/fcitx5-funput` | Link Rust trực tiếp (không qua FFI) |

Platform macOS/Windows **không** import trực tiếp — đi qua `funput-ffi`.

## Hiện thực

Xem [IMPLEMENTATION.md](./IMPLEMENTATION.md) — roadmap theo phase E0–E4.

## Tests

```bash
cargo test -p funput-engine
cargo clippy -p funput-engine -- -D warnings
```

**E0:** `process_char` stub trả `Action::None`; pipeline/diff implement ở E1.
