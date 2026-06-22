# funput-cli

Binary **dev-tool** (`funput`) — chạy `funput-engine` thẳng từ terminal để test, debug, và CI mà
**không** cần build platform shell hay cấp quyền Accessibility.

Trả lời nhanh đúng một câu: **"Engine có transform đúng không?"** Không phải IME thật — không hook
bàn phím, không inject vào app khác. Nó nạp một chuỗi qua engine rồi **mô phỏng vai trò platform**
(áp từng `ImeResult` vào một app-text model) để in ra text người dùng sẽ thấy.

## Dùng

```bash
# qua cargo (dev)
cargo run -p funput-cli -- run "a1 b2"                # → á b2       (VNI mặc định)
cargo run -p funput-cli -- run -m telex "xins chaof"  # → xín chào   (Telex)
# hoặc cài binary tên `funput`
cargo install --path crates/funput-cli
funput run "xin1 chao2"          # → xín chào
funput run -m telex "card "      # → card    (English-restore khi gặp dấu cách)
funput run -m telex "card"       # → cảd     (chưa tới boundary → chưa restore)
funput run --steps "a1"          # bảng từng phím
funput repl -m telex --steps     # REPL: gõ một dòng + Enter; :q hoặc Ctrl-D để thoát
```

CLI:

```
funput run  [-m telex|vni] [--steps] <INPUT>   # transform → in app-text (hoặc bảng --steps)
funput repl [-m telex|vni] [--steps]           # REPL đọc từng dòng
```

- `INPUT` là **chuỗi literal**; dấu cách và dấu câu là **ranh giới từ**. English-restore chỉ kích
  hoạt tại boundary (Telex `"card "` → `card`; `"card"` → `cảd` vì chưa boundary).
- `-m, --method` mặc định `vni` (CLI luôn set method tường minh qua `Engine::set_method`).
- Mặc định in **chỉ app-text** (dễ pipe/diff); `--steps` in bảng từng phím:

```
$ funput run --steps "a1"
#   key   action  bs  output   buffer
1   a     None    0   -        a
2   1     Send    1   á        á
→ á
```

REPL **line-based** (không raw-mode → không thêm dep): banner in ra **stderr** để stdout sạch cho
pipe (`printf 'a1\nd9\n:q\n' | funput repl`).

## Mô phỏng platform (`sim.rs` — trái tim, thuần, có test)

`simulate(method, input) -> Simulation { app_text, steps }` làm **đúng** việc một platform shell làm:
áp từng `ImeResult` vào app-text.

```rust
match result.action {
    Action::None              => app_text.push(key),            // app nhận phím
    Action::Send | Restore    => { /* pop `backspace` ký tự */ app_text.push_str(&output) },
}
```

`Restore` gộp chung với `Send` để forward-compatible (ESC ở engine sau này). Mỗi `Step` ghi
`{ key, action, backspace, output, buffer }` cho chế độ `--steps`. `simulate` thuần I/O-free → unit
test trực tiếp; `main`/`repl` chỉ lo I/O.

## Cấu trúc module

```
src/
├── main.rs    # clap parse → dispatch run | repl
├── cli.rs     # Cli, Command{Run, Repl}, CommonOpts (method + steps), MethodArg
├── sim.rs     # Method, Step, Simulation, simulate()  ← logic thuần, có test
├── render.rs  # steps_table(&Simulation) -> String  (bảng --steps)
└── repl.rs    # vòng lặp đọc dòng tương tác
```

## Phụ thuộc & ai dùng

- `funput-cli → funput-engine → funput-core`, thêm `clap`. **Không** `funput-ffi` — gọi engine Rust
  **trực tiếp**, tránh overhead FFI khi dev.
- Contributor (test local trước khi build app), CI (regression Telex/VNI), maintainer (tái hiện báo
  lỗi "gõ X ra Y"). **End user không cần** — họ dùng app trong `platforms/`.

Cùng một engine với mọi platform → CLI chỉ là **cửa sổ debug**, không fork logic. Khi platform inject
sai, so sánh output `funput-cli` (engine đúng) với hành vi app thật để tách bug engine vs bug
inject.

## Tests

```bash
cargo test   -p funput-cli
cargo clippy -p funput-cli --all-targets -- -D warnings
```

Unit test ở `sim.rs`: Telex/VNI cơ bản + đa từ (`xins chaof`→`xín chào`), English-restore ở boundary
(`card `→`card`, `mas `→`má `), và `--steps` ghi đúng từng phím. Đối chiếu app-text với
`funput-engine/tests/fixtures/step_cases.rs` để chắc CLI khớp engine.

## Còn làm

- **Chưa expose `ToneStyle`**: CLI chỉ có `-m method`, nên đang chạy theo kiểu đặt dấu mặc định của
  engine (`Traditional`). Thêm cờ `--tone-style traditional|modern` để test được kiểu mới.
- **REPL per-keystroke** (raw-mode, cần `crossterm`) — hiện chỉ line-based.
