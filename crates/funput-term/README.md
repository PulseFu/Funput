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
- **`Ctrl-\`**: bật/tắt tiếng Việt. **`Ctrl-^`**: xoay **Telex↔VNI** ngay trong phiên (đổi/tắt qua
  `FUNPUT_CYCLE_METHOD`). Trạng thái hiện ở **tiêu đề cửa sổ** dạng `Funput · VI · Telex` (OSC, tự bọc
  passthrough cho **tmux/screen**), và ở **màu con trỏ** (xanh khi VI, reset khi EN) — vẫn thấy được cả
  khi tiêu đề bị ẩn.
- Từ không hợp lệ tiếng Việt tự khôi phục ở ranh giới từ (`card ` → `card`).

**"Luôn bật":** chạy `funput-term install` để in (hoặc `--write` để ghi) đoạn alias vào shell rc, hoặc
cấu hình terminal emulator chạy `funput-term -- $SHELL` → mọi app trong cửa sổ đó đều gõ được.

```bash
funput-term install --alias claude --alias cursor   # in alias cho $SHELL
funput-term install --shell zsh --alias claude --write   # ghi vào ~/.zshrc (idempotent)
```

## Cấu hình

funput-term đọc file cấu hình chung của Funput: `dirs::config_dir()/Funput/settings.json` (Linux/Win
chia sẻ luôn preferences của IME hệ thống; macOS dùng file riêng tại path chuẩn). Áp dụng được:
`method`, `toneStyle`, `enabled`, `smartRestore`, `eagerRestore`, `spellCheck`, `autoCapitalize`,
`shortcuts` (gõ tắt). File thiếu key vẫn nạp được (mọi key có default).

**Thứ tự ưu tiên:** cờ CLI > biến môi trường > `settings.json` > mặc định.

- CLI: `-m, --method telex|vni`; chương trình truyền sau `--` (mặc định `$SHELL`).
- Env: `FUNPUT_METHOD`, `FUNPUT_TONE_STYLE`, `FUNPUT_ENABLED`, `FUNPUT_TOGGLE` (vd `ctrl-\`,
  `ctrl-space`), `FUNPUT_CYCLE_METHOD` (phím xoay Telex↔VNI; `off`/`none` để tắt),
  `FUNPUT_CURSOR_COLOR_VI`, `FUNPUT_CONFIG` (đường dẫn file khác).

## Hành vi & phạm vi

| | |
|--|--|
| Terminal emulator | **Tất cả** (chỉ cần TTY) |
| Hệ điều hành | macOS, Linux (Unix PTY). Windows (ConPTY): đang làm |
| App nhập theo dòng (shell, Claude, Cursor, REPL) | Soạn đầy đủ |
| App full-screen (vim, less, htop) | **Tự tắt** soạn (phát hiện alt-screen) để không phá UI |
| Backspace giữa lúc soạn | `engine.on_backspace()` — sửa rồi soạn tiếp (`Phua` ⌫ `s` → `Phú`) |
| Enter / Tab khi đang soạn | Đi qua engine trước → English-restore kịp chạy (`text`+Enter gửi `text`, không phải `tẽt`) |
| Bracketed paste (`ESC[200~ … ESC[201~`) | Nội dung dán forward **thô** (không soạn từng ký tự); marker giữ nguyên cho app con |
| Escape / mũi tên / phím tắt / UTF-8 | Forward **thô**, flush composition |

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
├── main.rs    # clap CLI: [run] -m telex|vni [-- command] (default $SHELL); subcommand `install`
├── config.rs  # THUẦN: đọc settings.json → TermConfig; overlay env; apply_to(engine); ưu tiên CLI>env>file
├── install.rs # THUẦN: snippet(shell, aliases) (bash/zsh/fish, idempotent) + ghi vào rc file
├── app.rs     # forward_input (seam THUẦN, có test) + run() orchestration (spawn PTY, threads, indicators)
├── input.rs   # THUẦN: Classifier byte → ByteKind (Printable/Control/Escape/Utf8/Toggle/CycleMethod/Paste)
├── inject.rs  # THUẦN: result_bytes(char, &ImeResult) → bytes (None→phím; Send/Restore→DEL×bs + UTF-8)
├── output.rs  # forward_output + AltScreenScanner (ESC[?1049h/l, chịu được chunk bị cắt)
├── term.rs    # RawModeGuard (RAII), Mux passthrough, set_title (OSC) + set_cursor_cue (OSC 12/112)
└── state.rs   # SharedState: enabled (toggle) + alt_screen (atomics)
```

`input` / `inject` / `forward_input` / `config` / `install` **thuần, không I/O thật** → unit-test bằng
pipe in-memory hoặc input dạng chuỗi.

### Quy tắc xử lý (trong `forward_input`)

- Khởi tạo: `config.apply_to(engine)` (method, tone, smart/eager restore, spell-check, auto-cap, gõ tắt).
- `Toggle` (mặc định `0x1c`) → `state.toggle()` + `engine.clear()` + cập nhật title & màu con trỏ.
- `CycleMethod` (mặc định `Ctrl-^` `0x1e`, đổi/tắt qua config) → `engine.set_method` xoay Telex↔VNI + `engine.clear()` + cập nhật indicator (`Status{enabled, method}`).
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
- Bracketed paste: `input.rs` thấy `ESC[200~` → `in_paste` → nội dung dán phân loại `Paste`
  (forward thô) tới `ESC[201~`. Chịu được marker bị cắt qua nhiều chunk; buffer tham số CSI có giới hạn.
- Title qua mux: `term.rs::detect_mux` đọc `$TMUX`/`$STY`/`$TERM`; `title_sequence` bọc DCS passthrough
  cho tmux (nhân đôi ESC) và screen.

## Quan hệ với phần còn lại

```
funput-core → funput-engine → funput-term   (Rust, link TRỰC TIẾP; KHÔNG qua funput-ffi)
```

`funput-term` giải đúng bài toán **terminal** — vốn là điểm mù của IME hệ thống (macOS IMKit,
Windows TSF, Linux Fcitx5). Nó hữu ích **ngay cả khi** đã cài IME hệ thống. Là binary độc lập, không
phải thư viện cho crate khác link.

## Phụ thuộc

`funput-core` · `funput-engine` · `portable-pty` (PTY/ConPTY) · `crossterm` (raw-mode/size) · `clap`
(CLI) · `serde`/`serde_json` (đọc settings.json) · `dirs` (config path) · `signal-hook` (resize, chỉ
unix). **Không** `funput-ffi`.

## Tests

```bash
cargo test   -p funput-term
cargo clippy -p funput-term --all-targets -- -D warnings
cargo run    -p funput-term -- cat       # gõ "as" → "á"
```

Unit test thuần (pipe in-memory): classifier (printable/control/escape/mũi tên/Alt/utf8/toggle/cycle-method),
`inject` (None/Send/Restore), `forward_input` (compose `as`→`a`+DEL+`á`, `Phua`⌫`s`→`Phú`, xoay
Telex→VNI giữa dòng `as`+Ctrl-^+`as`→`áas`,
`text`+Enter restore, `mas`+Enter giữ `má`, revert `mixx`→`mix`, toggle off, bracketed paste giữ
nguyên + soạn lại sau khi dán, config `enabled=false` không soạn, gõ tắt `vn`→`Việt Nam`), bracketed
paste (marker bị cắt, toggle/chữ trong paste là thô, CSI quá dài không phải marker), alt-screen
scanner (kể cả chunk bị cắt), `config` (parse camelCase, default khi thiếu key / JSON hỏng, env
override + ưu tiên, parse phím toggle), `install` (snippet bash/zsh/fish, idempotent, detect shell),
`title_sequence` & `cursor_cue` (none/tmux/screen), state.

## Còn làm (sau v1)

- **Windows (ConPTY):** hiện chỉ Unix PTY; stack `portable-pty` đã sẵn, cần tầng input/resize cho
  Windows (TT6).
- **Đổi kiểu đặt dấu lúc đang chạy:** đã có xoay Telex↔VNI (`Ctrl-^`); còn thiếu phím bật/tắt
  kiểu đặt dấu (traditional/modern) ngay trong phiên.
- **Per-app profile:** dùng `excludedApps` từ settings.json để tự bật/tắt theo lệnh được bọc.
