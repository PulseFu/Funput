# Funput cho Windows

Bộ gõ tiếng Việt cho Windows theo mô hình **keyboard hook + SendInput** (như UniKey/EVKey):
nghe phím toàn cục → đẩy vào `funput-engine` → xoá lùi + chèn lại text đã soạn. App chạy nền ở
**tray icon**; cửa sổ **Cài đặt / Onboarding** là **UI native Slint** (style Fluent, nền Mica),
không còn WebView2/Node. Đóng gói thành **một file `.exe` portable**.

> Crate này chỉ build trên **Windows** (dùng `slint` + crate `windows`). Một bảng `[workspace]`
> rỗng **tách** nó khỏi workspace ở repo gốc nên `cargo test --workspace` trên macOS/Linux vẫn
> xanh. Build từ trong thư mục này.

## Kiến trúc

```
Funput.exe — process nền nhẹ, không khởi tạo Slint
├─ hook.rs        WH_KEYBOARD_LL + WinEvent + tray, trên một thread message-loop
├─ keymap.rs      vkCode + modifier → funput_desktop::KeyEvent (ToUnicodeEx)
├─ inject.rs      InjectPlan → SendInput (VK_BACK ×n, KEYEVENTF_UNICODE), gắn INJECT_TAG
├─ shell.rs       state toàn cục: Engine + settings + per-app override (mutex)
├─ tray.rs        tray-icon + muda: VI/EN, VNI/Telex, Cài đặt…, Hướng dẫn, Kiểm tra cập nhật…, Thoát
├─ windows_ui.rs  spawn chính Funput.exe ở mode --settings/--onboarding; UI chạy process riêng
├─ commands.rs    glue: persist settings + autostart (auto-launch) + mở link + auto-update
├─ update.rs      auto-update: tải manifest JSON + verify chữ ký Ed25519 + tráo exe + relaunch
├─ dark_mode.rs   bật preferred-app-mode để context menu của tray theo dark/light hệ thống
└─ ui/*.slint     SettingsWindow + OnboardingWindow (style Fluent)
```

Logic thuần (quyết định inject + phân loại phím) nằm ở crate dùng-chung
[`funput-desktop`](../../crates/funput-desktop) — **có unit test chạy trên mọi nền**. Phần ở đây
chỉ là glue Windows + UI.

- **Tray ↔ UI:** tray spawn một process Funput con khi mở Settings/Onboarding. Process nền không gọi
  Slint; khi đóng cửa sổ, process con kết thúc và Windows thu hồi toàn bộ UI runtime/renderer.
- **Đồng bộ cấu hình:** process UI ghi `settings.json`; khi focus trở lại ứng dụng thường, process
  nền reload file trước phím kế tiếp và cập nhật engine/tray.
- **Chống đệ quy:** mọi event do `SendInput` tạo ra mang `dwExtraInfo = INJECT_TAG`; hook thấy tag
  thì bỏ qua.

## Build

Yêu cầu: Rust ≥ 1.92 (Slint 1.17), toolchain `x86_64-pc-windows-msvc`. Không cần Node/pnpm,
không cần WebView2.

```powershell
cd platforms\windows
cargo build --release          # → target\release\funput.exe (portable, một file)
```

Chạy: double-click `funput.exe` → icon "FU" hiện ở khay hệ thống, gõ được ngay ở mọi app. Lần đầu
mở **Onboarding**; tray có **"Cài đặt…"** (Settings) và **"Hướng dẫn"**. Cài đặt lưu ở
`%APPDATA%\Funput\settings.json` (giữ nguyên schema bản Tauri cũ — nâng cấp không mất cấu hình).

## Cửa sổ Settings / Onboarding

- UI **native Slint** (`ui/app.slint` + `ui/theme.slint`) — phong cách **liquid glass** kiểu macOS.
- Mỗi cửa sổ vẽ một **trường màu mờ trong cửa sổ** (3 quầng radial indigo/tím/teal trên nền
  gradient) rồi phủ **panel kính bán trong suốt** (translucent + viền sáng + đổ bóng) lên trên.
  Không dùng Mica/vibrancy thật (transparency của Slint kém ổn định) — cách này cho cảm giác kính
  mờ giống nhau trên mọi máy. **Tự đổi sáng/tối** theo `Palette.color-scheme` (cài đặt Windows).
- Mở **on-demand** trong process con; đóng cửa sổ kết thúc process đó, lần mở sau tạo process mới.
- Smart/eager restore điều khiển **engine thật**; mọi thay đổi persist ngay và áp vào engine.

## Đóng gói (installer)

Hiện build ra **`funput.exe` portable** (icon + manifest DPI/asInvoker nhúng qua `winresource`).
Bản Tauri trước lo luôn bundler; nếu cần MSI/NSIS, thêm bước riêng (`cargo-wix` hoặc script NSIS)
— **chưa** nằm trong crate này.

## Tự cập nhật (auto-update)

Giống bản macOS (Sparkle) hết mức có thể, **dùng chung cặp khóa EdDSA** với macOS:

- **Manifest** `funput-windows.json` (tương đương `appcast.xml`) đính kèm GitHub Release; app đọc ở
  URL cố định `releases/latest/download/funput-windows.json` (redirect tải thẳng — không đụng REST
  API nên không dính rate-limit).
- **Kiểm tra thủ công** (không tự poll nền): tray "Kiểm tra cập nhật…" hoặc Cài đặt → Giới thiệu.
- Khi có bản mới: tải `.exe` → **verify chữ ký Ed25519** bằng cùng public key macOS (`update.rs`) →
  `self-replace` exe đang chạy → relaunch (tray process nên **không cần logout** như macOS).
- **Ký** bằng `sign_update` của Sparkle trong CI (`secret SPARKLE_ED_PRIVATE_KEY`); xem job
  `windows-feed` trong [`.github/workflows/build-windows.yml`](../../.github/workflows/build-windows.yml).
- Test cục bộ: đặt `FUNPUT_UPDATE_FEED=<url manifest>` (chỉ có tác dụng ở **debug build**).

## Hạn chế đã biết

- **Không gõ được vào app chạy quyền Admin** (trừ khi `funput.exe` cũng chạy Admin) — bản chất của
  hook không-elevated.
- `ToUnicodeEx` trong LL hook có thể ảnh hưởng trạng thái dead-key ở bố cục bàn phím đặc biệt — ổn
  với layout US/Vietnamese thông dụng.
- Chưa ký Authenticode (có thể bị SmartScreen cảnh báo lần đầu).
- Emoji trong UI hiển thị tuỳ renderer (FemtoVG) — có thể thay bằng icon nếu cần.

## Verify (trên Windows)

Notepad/WordPad/trình duyệt:
- VNI `xin chaof` → `xin chào`; Telex `tieesng vieejt` → `tiếng việt`.
- Backspace sửa dấu: `Phuas` → ⌫ → `s` → `Phú`.
- English-restore: `text ` → `text `.
- Tray: left-click bật/tắt "Tiếng Việt" (icon đổi màu↔mono); right-click đổi VNI/Telex có hiệu lực
  từ chữ kế; "Thoát" để thoát.
- `Ctrl+` ``` bật/tắt nhanh (đổi được trong Settings).
- Settings: đổi kiểu gõ / phím tắt / smart-eager có hiệu lực ngay; **giữ qua restart**.
- Đổi Windows sang Dark/Light → cửa sổ đổi nền gradient + màu chữ theo; right-click tray ở Dark mode
  → context menu cũng nền tối.
- Bật "Khởi động cùng Windows" → có registry `HKCU\…\Run`.
- Auto-update: đặt `FUNPUT_UPDATE_FEED` trỏ manifest có version cao hơn (debug build) → tray/Settings
  "Kiểm tra cập nhật…" → tải + verify + tráo exe + relaunch; chữ ký sai thì báo lỗi, không tráo.
