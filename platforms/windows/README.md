# Funput cho Windows (MVP)

Bộ gõ tiếng Việt cho Windows theo mô hình **keyboard hook + SendInput** (như UniKey/EVKey):
nghe phím toàn cục → đẩy vào `funput-engine` → xoá lùi + chèn lại text đã soạn. Giao diện MVP là
**tray icon** (không cửa sổ). Đóng gói thành **một file `.exe` portable**.

> Crate này chỉ build trên **Windows** (dùng `tauri` + crate `windows`). Nó **bị `exclude`** khỏi
> workspace ở repo gốc nên `cargo test --workspace` trên macOS/Linux vẫn xanh. Build từ trong thư
> mục này.

## Kiến trúc

```
Funput.exe (Tauri 2, Rust) — khởi động ẩn, không cửa sổ
├─ hook.rs   WH_KEYBOARD_LL + message loop (thread riêng)
├─ keymap.rs vkCode + modifier → funput_desktop::KeyEvent (ToUnicodeEx)
├─ inject.rs InjectPlan → SendInput (VK_BACK ×n, KEYEVENTF_UNICODE), gắn INJECT_TAG
├─ shell.rs  state toàn cục: Engine + enabled + method (mutex)
└─ tray.rs   tray menu: Tiếng Việt | VNI/Telex | Thoát
```

Logic thuần (quyết định inject + phân loại phím) nằm ở crate dùng-chung
[`funput-desktop`](../../crates/funput-desktop) — **có unit test chạy trên mọi nền**. Phần ở đây
chỉ là glue Windows.

- **Toggle VI/EN:** `Ctrl + ` ``` (backtick) — hoặc menu tray. (Cấu hình được: pha sau.)
- **Chống đệ quy:** mọi event do `SendInput` tạo ra mang `dwExtraInfo = INJECT_TAG`; hook thấy tag
  thì bỏ qua.

## Build

Yêu cầu: Rust (toolchain `x86_64-pc-windows-msvc`), [Tauri CLI](https://tauri.app), WebView2
runtime (có sẵn trên Win11 / hầu hết Win10).

```powershell
# 1) Build giao diện web dùng chung trước (dist/ bị gitignore):
pnpm -C ..\..\ui install
pnpm -C ..\..\ui build         # → platforms/ui/dist (Tauri nhúng qua frontendDist)

# 2) Build app:
cd platforms\windows\src-tauri
cargo build --release          # → target\release\funput.exe (portable, một file)
# hoặc qua Tauri CLI:
cargo tauri build
```

`tauri.conf.json` đặt `bundle.active = false` → **chỉ ra `funput.exe`**, không sinh MSI/NSIS.

Chạy: double-click `funput.exe` → icon "FU" hiện ở khay hệ thống, gõ được ngay ở mọi app. Lần đầu
mở **Onboarding**; tray có **"Cài đặt…"** (Settings) và **"Hướng dẫn"**. Cài đặt lưu ở
`%APPDATA%\Funput\settings.json`; bật "Khởi động cùng Windows" trong Settings để tự chạy.

## Cửa sổ Settings / Onboarding

- UI là Svelte (port design system macOS) ở [`platforms/ui`](../ui), nhúng qua `frontendDist`.
- Mở **on-demand** (tray hoặc lần đầu) và **destroy khi đóng** → WebView2 chỉ tốn RAM khi mở.
- Nền **Acrylic** (`window-vibrancy`) cho cảm giác kính mờ như Liquid Glass.
- Smart/eager restore điều khiển **engine thật** (setter mới trong `funput-engine`).

## Hạn chế đã biết

- **Không gõ được vào app chạy quyền Admin** (trừ khi `funput.exe` cũng chạy Admin) — bản chất của
  hook không-elevated.
- **English-restore** kích hoạt ở space/dấu câu; **chưa** ở Enter/Tab (để Enter/Tab hoạt động đúng
  trong app). Sẽ tinh chỉnh sau.
- `ToUnicodeEx` trong LL hook có thể ảnh hưởng trạng thái dead-key ở bố cục bàn phím đặc biệt — ổn
  với layout US/Vietnamese thông dụng.
- Chưa ký Authenticode (có thể bị SmartScreen cảnh báo lần đầu).

## Verify (trên Windows)

Notepad/WordPad/trình duyệt:
- VNI `xin chaof` → `xin chào`; Telex `tieesng vieejt` → `tiếng việt`.
- Backspace sửa dấu: `Phuas` → ⌫ → `s` → `Phú`.
- English-restore: `text ` → `text `.
- Tray: tắt "Tiếng Việt" → ra Latin thường; đổi VNI/Telex có hiệu lực từ chữ kế.
- `Ctrl+` ``` bật/tắt nhanh (đổi được trong Settings).
- Settings: đổi kiểu gõ / phím tắt / smart-eager có hiệu lực ngay; **giữ qua restart**.
- Smart off → `card ` giữ `cảd`; eager off → chỉ đổi lại ở dấu cách.
- Bật "Khởi động cùng Windows" → có registry Run key; đóng cửa sổ → RAM nền về thấp.
