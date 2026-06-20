# platforms/ui — Giao diện web dùng chung (Windows ↔ Linux)

Frontend **dùng chung** cho các shell desktop không-phải-macOS: cửa sổ Settings và
Onboarding của Funput. Viết **một lần** ở đây, các shell nhúng lại:

- **Windows** ([../windows](../windows)) — app Tauri trỏ `frontendDist` tới `./dist`.
- **Linux** (pha sau) — settings GUI của bản Fcitx5 dùng lại chính UI này.

macOS có giao diện **native riêng** (SwiftUI, xem `platforms/macos/Funput/DesignSystem`); thư
mục này là bản port design system đó sang web (màu, glass/acrylic, typography) để Windows/Linux
dùng chung.

## Trạng thái

MVP Windows chạy **tray-only**, không mở cửa sổ, nên hiện chỉ có `dist/index.html` **placeholder**
(đủ để `tauri build` có `frontendDist`). Settings/Onboarding thật là việc của pha sau:

- Khung đề xuất: bundler nhẹ (Vite) → output ra `dist/`.
- Cửa sổ Tauri bật nền **Acrylic/Mica** qua `window-vibrancy` để giống Liquid Glass của macOS.
