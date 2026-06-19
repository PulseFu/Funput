# Funput Crates

Rust workspace chứa **toàn bộ logic lõi** của Funput — bộ gõ tiếng Việt đa nền tảng. Các crate ở đây **không phụ thuộc UI** và **không phụ thuộc platform hook** (CGEventTap, Fcitx5, Windows hook). Platform shell trong `platforms/` gọi vào các crate này qua FFI hoặc link trực tiếp.

## Mục tiêu thiết kế

- **Một engine duy nhất** cho macOS, Linux, Windows
- **Pure logic** tách khỏi inject / keyboard hook
- **Testable** — chạy unit test không cần OS IME hay Accessibility
- **Nhẹ** — rule-based Telex/VNI, không runtime nặng

## Danh sách crate

| Crate | Vai trò | Doc |
|-------|---------|-----|
| [`funput-core`](./funput-core/README.md) | Logic chuyển đổi Telex/VNI, validation, Unicode | Chi tiết ↓ |
| [`funput-engine`](./funput-engine/README.md) | Session, buffer, pipeline key → `ImeResult` | Chi tiết ↓ |
| [`funput-ffi`](./funput-ffi/README.md) | C ABI cho Swift / native bridge | Chi tiết ↓ |
| [`funput-cli`](./funput-cli/README.md) | Binary dev/test engine từ terminal | Chi tiết ↓ |

## Luồng phụ thuộc

```
funput-core
     ↓
funput-engine
     ↓
funput-ffi ──→ platforms/macos, platforms/windows
     ↓
funput-cli (dev tool)
```

| Crate | Phụ thuộc |
|-------|-----------|
| `funput-core` | Không phụ thuộc crate Funput khác |
| `funput-engine` | `funput-core` |
| `funput-ffi` | `funput-engine` |
| `funput-cli` | `funput-engine` |

## Luồng dữ liệu (tổng quan)

```
Key event (platform)
       ↓
  funput-ffi::ime_key()          ← platforms gọi qua C ABI
       ↓
  funput-engine                  ← buffer, session, word boundary
       ↓
  funput-core                    ← Telex/VNI transform + validation
       ↓
  ImeResult { action, backspace, chars }
       ↓
  Platform inject layer          ← Backspace / Selection / AX-sync (ngoài crates/)
```

## Ranh giới trách nhiệm

| Thuộc `crates/` | **Không** thuộc `crates/` |
|-----------------|---------------------------|
| Telex, VNI, validation | CGEventTap, Fcitx5, Windows hook |
| Buffer & session state | Inject text vào app (Backspace, Unicode events) |
| `ImeResult` struct | Menu bar, settings UI |
| C ABI export | Per-app auto-detect (AX role, bundle ID) |
| Unit/integration tests logic | Dictionary lớn (phase 2, nếu có) |

## Workspace members (dự kiến)

```toml
[workspace]
members = [
    "crates/funput-core",
    "crates/funput-engine",
    "crates/funput-ffi",
    "crates/funput-cli",
]
```

Linux Fcitx5 engine (`platforms/linux/fcitx5-funput/`) link trực tiếp `funput-engine` — có thể nằm ngoài hoặc trong workspace tùy cách tổ chức build.

## Thứ tự triển khai

1. `funput-core` — Telex converter + tests
2. `funput-engine` — session + `ImeResult`
3. `funput-cli` — test end-to-end từ terminal
4. `funput-ffi` — C ABI khi bắt đầu `platforms/macos` hoặc `platforms/windows`

## Release (macOS)

Funput là một **input method**, không sandbox được nên **không phát hành qua Mac App
Store**. Phân phối bằng **DMG** đã ký *Developer ID* + *notarize* (tải trực tiếp / GitHub
Releases) — giống mọi bộ gõ bên thứ ba.

Script: [`platforms/macos/scripts/release.sh`](platforms/macos/scripts/release.sh) —
build universal (arm64 + x86_64) → ký Developer ID → notarize → staple → đóng DMG. DMG
kèm `Install Funput.command` tự copy vào `~/Library/Input Methods` và đăng ký input source.

### Cài đặt một lần
1. Tạo certificate **Developer ID Application** (Xcode → Settings → Accounts → Manage
   Certificates → + → *Developer ID Application*).
2. Tạo *notarytool* credential profile (cần [app-specific
   password](https://support.apple.com/en-us/102654)):
   ```sh
   xcrun notarytool store-credentials funput \
     --apple-id <email> --team-id RSARFZ5CD3 --password <app-specific-password>
   ```

### Chạy
```sh
cd platforms/macos
DRY_RUN=1 ./scripts/release.sh   # ad-hoc sign, bỏ notarize — thử pipeline (chưa cần cert)
./scripts/release.sh             # release thật: Developer ID + notarize + staple
```
Output: `platforms/macos/build/release/Funput-<version>.dmg` kèm SHA-256 để dán vào
GitHub Release.

### Tự động hóa — GitHub Actions

[`.github/workflows/release-macos.yml`](.github/workflows/release-macos.yml) chạy đúng
`release.sh` trên runner `macos-latest`: push tag `v*` → build + ký + notarize + tạo
GitHub Release kèm DMG. `workflow_dispatch` chạy cả pipeline nhưng **không** publish (để
thử secrets).

**Secrets cần thêm** (repo → Settings → Secrets and variables → Actions):

| Secret | Giá trị |
|--------|---------|
| `DEVELOPER_ID_CERT_P12` | base64 của file `.p12` (Developer ID Application cert **kèm private key**) |
| `DEVELOPER_ID_CERT_PASSWORD` | mật khẩu đặt khi export `.p12` |
| `NOTARY_APPLE_ID` | Apple ID dùng notarize (vd `pcodedynamics@gmail.com`) |
| `NOTARY_PASSWORD` | [app-specific password](https://support.apple.com/en-us/102654) |
| `NOTARY_TEAM_ID` | *(tùy chọn)* mặc định `RSARFZ5CD3` |

Export cert sang base64 (chạy local một lần):
```sh
# Keychain Access → My Certificates → chuột phải "Developer ID Application: …" →
# Export → lưu Funput-DeveloperID.p12 (đặt mật khẩu), rồi:
base64 -i Funput-DeveloperID.p12 | pbcopy   # dán vào secret DEVELOPER_ID_CERT_P12
```

**Cắt một bản release:**
```sh
# Đảm bảo MARKETING_VERSION trong project khớp tag bạn định đẩy.
git tag v1.2026.1
git push origin v1.2026.1     # → Actions build, notarize, publish Release
```
Tag `v1.2026.1` đặt tên DMG thành `Funput-1.2026.1.dmg` (workflow truyền `VERSION` từ tag).
