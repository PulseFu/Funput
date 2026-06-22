# funput-term

Gõ **tiếng Việt** trong app terminal (Claude Code, Cursor, shell, REPL…) — nơi IME hệ thống thường
lỗi vì terminal chạy raw-mode.

`funput-term` là **PTY wrapper trong suốt**: chạy app của bạn trong một pseudo-terminal, soạn tiếng
Việt từ luồng phím rồi đẩy text đã hoàn chỉnh vào app. **Không** quyền Accessibility, **không**
daemon, **không** hook hệ thống — chạy trong **mọi** terminal emulator (iTerm2, Terminal.app,
Alacritty, kitty, WezTerm, tmux, SSH…).

## Chạy

```bash
cargo run -p funput-term -- claude         # gõ tiếng Việt trong Claude CLI
cargo run -p funput-term -m telex -- bash  # chọn Telex (mặc định VNI)
funput-term                                # không tham số → bọc $SHELL
funput-term -- cursor
```

- VNI `xin1 chao2` hoặc Telex `xins chaof` → **xín chào**.
- **`Ctrl-\`**: bật/tắt tiếng Việt (trạng thái VI/EN hiện ở **tiêu đề cửa sổ** qua OSC).
- Từ không hợp lệ tiếng Việt tự khôi phục ở ranh giới từ (`card ` → `card`).

**"Luôn bật":** `alias claude='funput-term -- claude'`, hoặc cấu hình terminal emulator chạy
`funput-term -- $SHELL` → mọi app trong cửa sổ đó đều gõ được.

CLI: `-m, --method telex|vni` (mặc định `vni`); chương trình truyền sau `--` (mặc định `$SHELL`).

## Hành vi & phạm vi

| | |
|--|--|
| Terminal emulator | **Tất cả** (chỉ cần TTY) |
| Hệ điều hành | macOS, Linux (Unix PTY). Windows (ConPTY): đang làm |
| App nhập theo dòng (shell, Claude, Cursor, REPL) | Soạn đầy đủ |
| App full-screen (vim, less, htop) | **Tự tắt** soạn (phát hiện alt-screen) để không phá UI |
| Backspace giữa lúc soạn | `engine.on_backspace()` — sửa rồi soạn tiếp (`Phua` ⌫ `s` → `Phú`) |
| Enter / Tab khi đang soạn | Đi qua engine trước → English-restore kịp chạy (`text`+Enter gửi `text`, không phải `tẽt`) |
| Escape / mũi tên / phím tắt / paste / UTF-8 | Forward **thô**, flush composition |

## Cách hoạt động

Interposer trong suốt: chỉ **chặn phím chữ ASCII** để soạn, mọi thứ khác forward thô (giữ nguyên
escape/mouse/paste).

```
stdin  ─raw bytes─► input::Classifier ─► engine ─► inject::result_bytes ─► PTY ─► child
stdout ◄─────────── output: scan alt-screen ◄──────────────────────────── PTY ◄─ child
        main: spawn child trong PTY, chờ exit; thread SIGWINCH→resize; RawModeGuard (RAII)
```

Hai thread: stdin→pty (`forward_input`) và pty→stdout (`forward_output`). Engine sống gọn trong
thread input nên **không cần khoá**. Engine trả "xoá N ký tự + chèn chuỗi"; wrapper dịch thành bytes
(`DEL 0x7f × N` + UTF-8) đẩy vào stdin của child — đây chỉ là một "frontend terminal" cho **cùng**
engine của hệ Funput.

### Module

```
src/
├── main.rs    # clap CLI: -m telex|vni, [-- command]; default $SHELL; toggle Ctrl-\ (0x1c)
├── app.rs     # forward_input (seam THUẦN, có test) + run() orchestration (spawn PTY, threads)
├── input.rs   # THUẦN: Classifier byte → ByteKind (Printable/Control/Escape/Utf8/Toggle)
├── inject.rs  # THUẦN: result_bytes(char, &ImeResult) → bytes (None→phím; Send/Restore→DEL×bs + UTF-8)
├── output.rs  # forward_output + AltScreenScanner (ESC[?1049h/l, chịu được chunk bị cắt)
├── term.rs    # RawModeGuard (RAII), set_title (OSC)
└── state.rs   # SharedState: enabled (toggle) + alt_screen (atomics)
```

`input` / `inject` / `forward_input` **thuần, không I/O thật** → unit-test bằng pipe in-memory.

### Quy tắc xử lý (trong `forward_input`)

- `Toggle` (`0x1c`) → `state.toggle()` + `engine.clear()` + đổi title VI/EN.
- `Printable` khi đang soạn → `engine.process_char` → `result_bytes`.
- Backspace (`0x7f`/`0x08`) khi đang soạn → `engine.on_backspace()` + forward byte (app tự xoá ký tự).
- Tab/LF/CR (ranh giới từ) khi đang soạn → `engine.process_char(boundary)`: `None` → forward byte;
  ngược lại → `result_bytes` (English-restore chạy trước khi phím tới child).
- Còn lại (escape / utf8 / control khác / printable lúc đã tắt) → `engine.clear()` + forward thô.
- `input.rs` theo dõi state machine ESC → CSI/SS3 để mũi tên/Alt-combo không bị nhầm là chữ.

### Robustness

- `RawModeGuard` vào raw-mode **trước** khi spawn → không TTY thì fail nhanh, không bỏ rơi child;
  drop luôn khôi phục (kể cả panic).
- `portable_pty::openpty` + `CommandBuilder` kế thừa cwd + env. Child thoát → reader EOF → output
  thread dừng → exit đúng status code.
- Resize: Unix `SIGWINCH` (`signal-hook`) → `crossterm::size` → `master.resize`. Windows: TT6.
- Alt-screen: `output.rs` thấy `ESC[?1049h` → set `state.alt_screen` → input passthrough (vim/less
  không bị soạn).

## Quan hệ với phần còn lại

```
funput-core → funput-engine → funput-term   (Rust, link TRỰC TIẾP; KHÔNG qua funput-ffi)
```

`funput-term` giải đúng bài toán **terminal** — vốn là điểm mù của IME hệ thống (macOS IMKit,
Windows TSF, Linux Fcitx5). Nó hữu ích **ngay cả khi** đã cài IME hệ thống. Là binary độc lập, không
phải thư viện cho crate khác link.

## Phụ thuộc

`funput-core` · `funput-engine` · `portable-pty` (PTY/ConPTY) · `crossterm` (raw-mode/size) · `clap`
(CLI) · `signal-hook` (resize, chỉ unix). **Không** `funput-ffi`.

## Tests

```bash
cargo test   -p funput-term
cargo clippy -p funput-term --all-targets -- -D warnings
cargo run    -p funput-term -- cat       # gõ "as" → "á"
```

Unit test thuần (pipe in-memory): classifier (printable/control/escape/mũi tên/Alt/utf8/toggle),
`inject` (None/Send/Restore), `forward_input` (compose `as`→`a`+DEL+`á`, `Phua`⌫`s`→`Phú`,
`text`+Enter restore, `mas`+Enter giữ `má`, revert `mixx`→`mix`, toggle off), alt-screen scanner
(kể cả chunk bị cắt), state.

## Còn làm (sau v1)

- **Windows (ConPTY):** hiện chỉ Unix PTY; stack `portable-pty` đã sẵn, cần tầng input/resize cho
  Windows (TT6).
- **Bracketed paste:** khi dán (`ESC[200~ … ESC[201~`) nên pass-through thô thay vì soạn từng ký tự,
  tránh méo nội dung dán (phát hiện tương tự alt-screen).
- **Cấu hình bền:** phương thức + phím toggle đang cố định qua CLI; thêm env/file + đổi phương thức
  lúc đang chạy.
