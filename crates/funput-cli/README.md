# funput-cli

Binary **công cụ phát triển** — chạy `funput-engine` trực tiếp từ terminal để test, debug, và CI mà không cần build platform shell hay cấp Accessibility.

## Ý nghĩa

`funput-cli` là cách nhanh nhất trả lời: **“Engine có transform đúng không?”**

Không thay thế bộ gõ thật trên OS. Không hook keyboard. Chỉ mô phỏng input → output để dev và automated test.

## Trách nhiệm

| Làm | Không làm |
|-----|-----------|
| Gọi `funput-engine` với chuỗi key / text | CGEventTap, inject vào app khác |
| In `ImeResult` (action, backspace, chars) | Production IME cho end user |
| Test nhanh Telex/VNI từ command line | Settings UI |
| Scriptable cho CI | FFI export |

## Use cases

### 1. Test transform đơn lẻ

```bash
# Dự kiến — API chưa implement
funput telex "as"
# → action=Send, backspace=1, output=á

funput telex "dd"
# → action=Send, backspace=1, output=đ
```

### 2. Test chuỗi key (pipeline)

```bash
funput simulate telex --keys "a,s,space"
# In từng bước ImeResult
```

### 3. CI

```bash
cargo test -p funput-core
cargo test -p funput-engine
funput test-vectors --file tests/fixtures/telex.json
```

### 4. Debug khi phát triển platform

Khi macOS inject sai, so sánh:

- Output `funput-cli` (engine đúng)
- vs hành vi thật trên app (inject layer sai)

→ Tách bug engine vs bug platform.

## Cấu trúc module (dự kiến)

```
funput-cli/src/
├── main.rs
├── commands/
│   ├── telex.rs          # Transform một chuỗi
│   ├── simulate.rs       # Mô phỏng chuỗi phím
│   └── test_vectors.rs   # Chạy fixture JSON
└── output.rs             # Format ImeResult cho terminal
```

## Phụ thuộc

```
funput-cli → funput-engine → funput-core
```

**Không** phụ thuộc `funput-ffi` — gọi engine Rust trực tiếp, tránh overhead FFI khi dev.

## Ai dùng?

| Đối tượng | Mục đích |
|-----------|----------|
| Contributor | Test local trước khi build macOS app |
| CI | Regression test Telex/VNI |
| Maintainer | Debug báo lỗi từ user (“gõ X ra Y”) |

End user **không** cần cài `funput-cli` — họ dùng app trong `platforms/`.

## Quan hệ với platform shell

```
funput-cli          →  funput-engine  (dev, trực tiếp)
platforms/macos     →  funput-ffi     → funput-engine  (production)
platforms/linux     →  funput-engine  (trực tiếp)
```

Cùng một engine — CLI chỉ là **cửa sổ debug**, không fork logic.

## Build & chạy

```bash
cargo run -p funput-cli -- telex "as"
cargo install --path crates/funput-cli   # optional
```
