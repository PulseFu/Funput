# platforms/ui/public — tài nguyên tĩnh

Mọi file ở đây được Vite phục vụ ở **gốc** (`/`) và copy nguyên vào `dist/` khi build —
Tauri nhúng theo.

## Logo

Copy logo Funput vào đây với đúng tên:

```
platforms/ui/public/logo.png
```

Màn hình **Giới thiệu (About)** sẽ tự dùng `/logo.png`. Khuyến nghị:

- Định dạng **PNG**, nền **trong suốt**.
- **Vuông**, tối thiểu **256×256** (nên **512×512** cho sắc nét trên màn Retina).

Chưa có file thì About tự hiển thị badge **"FU"** thay thế.
