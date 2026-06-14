# funput-core

Crate **thuần logic** xử lý gõ tiếng Việt. Không biết keyboard hook, không biết macOS/Linux/Windows, không I/O, không config file.

## Ý nghĩa

`funput-core` trả lời câu hỏi: **“Chuỗi ký tự Latin này theo Telex/VNI nên biến thành gì?”**

Đây là nơi duy nhất chứa:

- Quy tắc VNI (`1` → sắc, `6` → `â`, `9` → `đ`, …) — **đã implement**
- Quy tắc Telex (`s` → sắc, `f` → huyền, `aa` → `â`, `dd` → `đ`, …) — **Phase 9**
- Validation âm tiết tiếng Việt trước khi transform
- Mapping Unicode (dấu thanh, mũ, móc, breve)

## Public API (frozen — Phase 8)

Chỉ bốn symbol public; mọi thứ khác là internal. Xem `cargo doc -p funput-core --no-deps`:

| Symbol | Mô tả |
|--------|--------|
| `InputMethod` | `Vni` hoặc `Telex` |
| `TransformKind` | `Pending`, `Applied`, `Reverted`, `Ignored` |
| `TransformResult` | `{ kind, text }` sau một keystroke |
| `apply(buffer, key, method)` | Transform một bước |

Breaking change cần semver và đồng bộ với `funput-engine`.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Transform syllable / buffer text | Giữ session theo thời gian (thuộc `funput-engine`) |
| Validate âm tiết hợp lệ | Đọc keycode phần cứng |
| Quy tắc đặt dấu (hoà, khoẻ, thuỷ) | Gửi Backspace / inject vào app |
| Revert (ví dụ `a11` → `a`) | Auto-restore tiếng Anh (thuộc `funput-engine`) |
| Unit + integration tests VNI | FFI, C ABI |

## Cấu trúc module (hiện tại)

```
funput-core/src/
├── lib.rs                 # Public API (frozen)
├── input_method/
│   ├── mod.rs
│   ├── vni.rs             # VNI digit mapping
│   └── telex.rs           # Telex stub (Phase 9)
├── composition/
│   ├── mod.rs
│   ├── transform.rs       # Pipeline: revert → validate → apply
│   └── revert.rs          # Double-key revert
├── validation/
│   └── syllable.rs        # parse_syllable, modifier validation
└── unicode/
    ├── mod.rs
    ├── marks.rs           # Tone / shape tables
    ├── vowels.rs          # Vowel cluster source of truth
    └── tone_position.rs   # Reposition rules (hoà, thuỷ, …)
```

## Input / Output

**Input:** buffer ký tự (Latin), kiểu gõ (Telex/VNI), key vừa nhập.

**Output:** [`TransformResult`] — engine so sánh buffer trước/sau để tính backspace và output.

Không trả `backspace` count — đó là nhiệm vụ `funput-engine`.

## Hiện thực

Xem [IMPLEMENTATION.md](./IMPLEMENTATION.md) — roadmap theo phase, **VNI trước, Telex sau**.

**Milestone Phase 8:** VNI pipeline hoàn chỉnh, API đóng băng — **sẵn sàng bắt `funput-engine`**.

## Ví dụ hành vi (VNI)

| Gõ (VNI) | Kết quả |
|----------|---------|
| `a1` | `á` |
| `d9` | `đ` |
| `uo7` | `ươ` |
| `hoa2` | `hoà` |
| `a11` | revert sắc |
| `vie5t` | `việt` |
| `truo7ng` | `trương` |
| `ngu7o7i2` | `người` |
| `text`+`1` | `text1` (Pending — engine restore) |

## Telex (Phase 9 — chưa implement)

| Gõ (Telex) | Kết quả (dự kiến) |
|------------|-------------------|
| `as` | `á` |
| `dd` | `đ` |
| `uow` | `ươ` |
| `hoaf` | `hoà` |
| `ass` | `as` (revert) |

Hiện tại `InputMethod::Telex` chỉ append key (`Pending`).

## Phụ thuộc

- **Không** phụ thuộc crate Funput khác
- Chỉ dùng crate Rust chuẩn
- **Không** `std::os`, **không** `libc`, **không** `serde`

## Ai gọi crate này?

Chỉ `funput-engine` (và tests). Platform code **không** import `funput-core` trực tiếp.

## Tests

```bash
cargo test -p funput-core
cargo test -p funput-core vni_full    # smoke: toàn bộ fixture const
cargo clippy -p funput-core -- -D warnings
cargo doc -p funput-core --no-deps
```

Fixture canonical: `tests/fixtures/vni_cases.rs` (Rust const). `tests/fixtures/vni.json.example` chỉ là tài liệu tham khảo, không load runtime.
