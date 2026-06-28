<p align="right">
  <a href="README.md">Tiếng Việt</a> · <strong>English</strong>
</p>

<p align="center">
  <img src="assets/logo.png" width="256" alt="Funput logo">
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

**Funput** is an open-source Vietnamese input method released for macOS,
Windows, and Linux. Android support is in development, with iOS planned next.
Every platform shares the same processing core to provide consistent typing
behavior across operating systems.

## Get started

<p align="center">
  <a href="https://github.com/Funput/Funput/releases/latest">
    <img src="https://img.shields.io/badge/Download-Latest_release-22C55E?style=for-the-badge&logo=github&logoColor=white" alt="Download the latest Funput release">
  </a>
  <a href="https://docs.funput.app/">
    <img src="https://img.shields.io/badge/Documentation-Installation_guide-2563EB?style=for-the-badge&logo=readthedocs&logoColor=white" alt="Read the Funput installation guide">
  </a>
  <a href="https://github.com/Funput/Funput/issues">
    <img src="https://img.shields.io/badge/Support-Report_an_issue-E11D48?style=for-the-badge&logo=github&logoColor=white" alt="Report a Funput issue">
  </a>
</p>

## Supported platforms

<p align="center">
  <img src="https://img.shields.io/badge/macOS-Released-22C55E?style=for-the-badge&logo=apple&logoColor=white" alt="macOS: released">
  <img src="https://img.shields.io/badge/Windows-Released-22C55E?style=for-the-badge&logo=windows&logoColor=white" alt="Windows: released">
  <img src="https://img.shields.io/badge/Linux-Released-22C55E?style=for-the-badge&logo=linux&logoColor=white" alt="Linux: released">
  <br>
  <img src="https://img.shields.io/badge/Android-In_development-EAB308?style=for-the-badge&logo=android&logoColor=white" alt="Android: in development">
  <img src="https://img.shields.io/badge/iOS-Planned-555555?style=for-the-badge&logo=apple&logoColor=white" alt="iOS: planned">
</p>

## Interface

<p align="center">
  <img src="assets/screenshot/screenshot-macos.png" width="900" alt="Funput settings interface on macOS">
  <br>
  <sub>Configure the input method, tone placement, and smart typing features on macOS.</sub>
</p>

## Near-instant response

Funput processes each keystroke in approximately **1.5 microseconds** — roughly
**650,000 keystrokes per second**. At normal typing speeds of only a few
keystrokes per second, engine latency is effectively imperceptible and does not
become a bottleneck in the input experience.

| Component / API | Measured scope | Telex | VNI |
|---|---|---:|---:|
| [`funput-core::apply_checked`](crates/funput-core) | Telex/VNI transformation core | 0.230 µs/key | 0.204 µs/key |
| [`funput-engine::Engine::process_char`](crates/funput-engine) | Full pipeline, including boundaries and English restore | 1.50 µs/key | 1.53 µs/key |
| [`funput-ffi::{funput_process_char, funput_buffer}`](crates/funput-ffi) | Engine through the C ABI, including composed-buffer reads | 1.54 µs/key | 1.53 µs/key |

> Results were measured from a release build on an Apple M-series machine. They
> cover Funput's own processing and exclude operating-system keyboard event
> delivery and rendering by the host application.

See the [benchmark methodology, source code, and reproduction instructions](benchmarks/README.md).

## Project status

Funput is under active development. Features and architecture may continue to
evolve during the project's early releases.

Bug reports, discussions, and contributions are welcome.

## License

[MIT](LICENSE) — © Funput
