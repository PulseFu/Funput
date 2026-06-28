<p align="right">
  <strong>Tiếng Việt</strong> · <a href="README.en.md">English</a>
</p>

<p align="center">
  <img src="assets/logo.png" width="256">
</p>

<pre align="center">
   ███████╗██╗   ██╗███╗   ██╗██████╗ ██╗   ██╗████████╗
   ██╔════╝██║   ██║████╗  ██║██╔══██╗██║   ██║╚══██╔══╝
█████╗  ██║   ██║██╔██╗ ██║██████╔╝██║   ██║   ██║
██╔══╝  ██║   ██║██║╚██╗██║██╔═══╝ ██║   ██║   ██║
██║     ╚██████╔╝██║ ╚████║██║     ╚██████╔╝   ██║
╚═╝      ╚═════╝ ╚═╝  ╚═══╝╚═╝      ╚═════╝    ╚═╝
</pre>

---

**Funput** là bộ gõ tiếng Việt mã nguồn mở, hiện đã phát hành trên macOS,
Windows và Linux, đang được phát triển cho Android và dự kiến hỗ trợ iOS. Mọi
nền tảng dùng chung một lõi xử lý để giữ hành vi gõ nhất quán.

## Bắt đầu

<p align="center">
  <a href="https://github.com/Funput/Funput/releases/latest">
    <img src="https://img.shields.io/badge/Tải_xuống-Bản_mới_nhất-22C55E?style=for-the-badge&logo=github&logoColor=white" alt="Tải phiên bản Funput mới nhất">
  </a>
  <a href="https://docs.funput.app/">
    <img src="https://img.shields.io/badge/Tài_liệu-Hướng_dẫn_cài_đặt-2563EB?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Hướng dẫn cài đặt Funput">
  </a>
  <a href="https://github.com/Funput/Funput/issues">
    <img src="https://img.shields.io/badge/Hỗ_trợ-Báo_lỗi-E11D48?style=for-the-badge&logo=github&logoColor=white" alt="Báo lỗi Funput">
  </a>
</p>

## Nền tảng hỗ trợ

<p align="center">
  <img src="https://img.shields.io/badge/macOS-Đã_phát_hành-22C55E?style=for-the-badge&logo=apple&logoColor=white" alt="macOS">
  <img src="https://img.shields.io/badge/Windows-Đã_phát_hành-22C55E?style=for-the-badge&logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/Linux-Đã_phát_hành-22C55E?style=for-the-badge&logo=linux&logoColor=white" alt="Linux">
  <br>
  <img src="https://img.shields.io/badge/Android-Đang_phát_triển-EAB308?style=for-the-badge&logo=android&logoColor=white" alt="Android">
  <img src="https://img.shields.io/badge/iOS-Đã_lên_kế_hoạch-555555?style=for-the-badge&logo=apple&logoColor=white" alt="iOS">
</p>

## Giao diện

<p align="center">
  <img src="assets/screenshot/screenshot-macos.png" width="900" alt="Giao diện cài đặt Funput trên macOS">
  <br>
  <sub>Tuỳ chỉnh phương thức gõ, kiểu đặt dấu và các tính năng thông minh trên macOS.</sub>
</p>

## Phản hồi gần như tức thời

Funput xử lý mỗi phím trong khoảng **1,5 micro giây** — tương đương khoảng
**650.000 phím mỗi giây**. Với tốc độ gõ thông thường chỉ vài phím mỗi giây, độ
trễ của engine gần như không thể cảm nhận và không trở thành nút thắt trong trải
nghiệm nhập liệu.

| Component / API | Phạm vi đo | Telex | VNI |
|---|---|---:|---:|
| [`funput-core::apply_checked`](crates/funput-core) | Lõi biến đổi Telex/VNI | 0,230 µs/phím | 0,204 µs/phím |
| [`funput-engine::Engine::process_char`](crates/funput-engine) | Pipeline đầy đủ, gồm boundary và English restore | 1,50 µs/phím | 1,53 µs/phím |
| [`funput-ffi::{funput_process_char, funput_buffer}`](crates/funput-ffi) | Engine qua C ABI và đọc composed buffer | 1,54 µs/phím | 1,53 µs/phím |

> Kết quả được đo với release build trên máy Apple M-series. Số liệu bao gồm phần
> xử lý của Funput, không bao gồm thời gian chuyển sự kiện bàn phím của hệ điều hành
> hoặc render của ứng dụng đích.

Xem [phương pháp đo, mã benchmark và cách tái lập](benchmarks/README.md).

## Trạng thái

Funput đang được phát triển tích cực. Tính năng và kiến trúc có thể tiếp tục thay đổi trong các phiên bản đầu.

Bug report, thảo luận và đóng góp đều được chào đón.

## License

[MIT](LICENSE) — © Funput
