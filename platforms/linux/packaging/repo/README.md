# Kho phần mềm Funput — vận hành (maintainer)

> **Đối tượng: maintainer/người vận hành kho**, không phải người dùng cuối.
> Người dùng cài đặt tại **https://repo.funput.app** (trang `index.html`) hoặc theo
> mục cài đặt trong [`platforms/linux/README.md`](../../README.md). File này ghi cách
> *dựng* và *bảo trì* kho (khóa GPG, secrets, Pages, DNS).

Phân phối Phase 2: biến `.deb`/`.rpm` của bản release mới nhất thành **kho apt + dnf
có ký GPG**, host trên **GitHub Pages**, để người dùng `apt/dnf/zypper upgrade` thay vì
tải lại tay. Build bởi [`.github/workflows/publish-repo.yml`](../../../../.github/workflows/publish-repo.yml),
tự chạy mỗi khi có release chính thức (không phải prerelease).

> Đây là kho "bên thứ ba" (như Docker/VS Code): người dùng tin **khóa của bạn** và URL
> Pages. Khác với kênh *official* của distro (OBS/COPR/PPA) — cái đó để Phase 3.

## Thiết lập một lần

### 1. Tạo khóa ký GPG
Dùng **RSA 4096** cho tương thích rộng nhất (apt lẫn rpm đều nhận); uid **phải** là
`Funput <hello@funput.app>` (khớp `%_gpg_name` trong workflow):

```sh
gpg --batch --full-generate-key <<'EOF'
Key-Type: RSA
Key-Length: 4096
Name-Real: Funput
Name-Email: hello@funput.app
Expire-Date: 0
Passphrase: <đặt-mật-khẩu-mạnh>
%commit
EOF

# Xuất khóa BÍ MẬT (dán vào secret GPG_PRIVATE_KEY):
gpg --armor --export-secret-keys hello@funput.app
```

> Giữ khóa bí mật an toàn (vault/password manager). Mất khóa = phải phát hành khóa mới
> và mọi người phải import lại. Khóa công khai do CI tự xuất (`funput.asc`), không cần commit.

### 2. Đặt secrets của repo
Settings → Secrets and variables → Actions:
- `GPG_PRIVATE_KEY` — toàn bộ khối `-----BEGIN PGP PRIVATE KEY BLOCK----- … END …`.
- `GPG_PASSPHRASE` — mật khẩu của khóa trên.

### 3. Bật GitHub Pages + custom domain `repo.funput.app`
Kho phục vụ dưới thương hiệu funput.app qua subdomain riêng (tách hẳn site marketing —
**không** nhồi `.deb`/`.rpm` vào image web Angular):

1. Settings → Pages → **Source = GitHub Actions**. (Workflow tự deploy, không cần nhánh `gh-pages`.)
2. DNS: thêm bản ghi **CNAME `repo` → `<owner>.github.io`** (vd `Funput.github.io`). Để DNS phân giải
   xong rồi mới chạy workflow.
3. File `CNAME` (chứa `repo.funput.app`) do workflow tự ghi vào artifact Pages — không cần commit.

Đổi domain khác? Đặt variable `REPO_BASE_URL` = URL mới (vd `https://linux.funput.app/`, **giữ `/`
cuối**); workflow tự suy host cho `CNAME` từ đó. Không đặt thì mặc định `https://repo.funput.app/`.

### 4. Chạy
Tự chạy khi release `released`. Chạy tay: Actions → **Publish package repo** → Run workflow.
Xong, trang cài đặt nằm ở URL Pages (xem mục cuối `index.html.in` để biết các lệnh người dùng).

## Hoạt động thế nào (tóm tắt)
- **apt** (job `apt`): tải `.deb` của release → `apt-ftparchive` tạo `Packages`/`Release` →
  ký `InRelease` + `Release.gpg`. Kho **phẳng** (`deb … ./`).
- **dnf/zypper** (job `rpm`, container Fedora): `rpm --addsign` từng gói → `createrepo_c` →
  ký `repodata/repomd.xml`. Người dùng đặt `gpgcheck=1` + `repo_gpgcheck=1`.
- **deploy**: gộp `public/{deb,rpm}` + `funput.asc` + `index.html` + `funput.repo`, đẩy lên Pages.

Mỗi lần chạy **dựng lại toàn bộ** từ release hiện tại (stateless) — package manager chỉ cần
phiên bản mới nhất để chào upgrade, nên không cần giữ lịch sử gh-pages.

## Giới hạn / lưu ý
- Người dùng phải tin khóa của bạn (one-time `rpm --import` / `signed-by`). Để có dấu tin cậy
  của distro thì cần OBS/COPR/PPA (Phase 3).
- Kho chỉ chứa **bản mới nhất**; không phục vụ cài lại phiên bản cũ qua kho (vẫn tải được từ
  GitHub Releases).
- RPM ký gói + ký repomd; APT ký Release. Khóa hết hạn = `Expire-Date: 0` (vô hạn) để khỏi gãy
  kho; nếu đặt hạn, nhớ gia hạn và chạy lại workflow trước khi hết hạn.
