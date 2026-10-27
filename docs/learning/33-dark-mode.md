# Bước 33 — Dark mode (View → Theme: System / Light / Dark)

*Bạn yêu cầu (2026-10-27); chỗ đặt sẵn ở menu từ bước 31a.*

## Trạng thái
- `pnpm build`: OK. Rust chỉ thêm một quyền (`core:app:allow-set-app-theme`), app dev đã build lại.
- Giao diện: bạn đã thử, OK. Đã commit.

## Thay đổi
**View → Theme** có ba lựa chọn (dấu ✓ ở lựa chọn hiện tại):

| Lựa chọn | Ý nghĩa |
|---|---|
| System (mặc định) | Theo macOS: macOS đổi sáng/tối (kể cả tự đổi theo giờ) thì app đổi theo |
| Light | Luôn sáng |
| Dark | Luôn tối |

- Áp dụng **ngay** cho **mọi cửa sổ**: cửa sổ chính, cửa sổ diff, Conflict Solver (cả editor Monaco), kể cả **thanh tiêu đề** và menu chuột phải native.
- Lựa chọn **được nhớ** cho lần mở app sau.

## Cách làm: để macOS làm phần khó
- Từ bước 1, `styles.css` đã có **hai bộ màu** (biến CSS `--bg`, `--text`, `--accent`…): bộ sáng, và bộ tối trong `@media (prefers-color-scheme: dark)`. Mọi màu trong app đều đi qua các biến này (đã kiểm tra: không còn màu viết cứng). `monaco.ts` cũng nghe cùng điều kiện đó để chọn theme `vs` / `vs-dark`.
- Điều kiện `prefers-color-scheme` của khung web đi theo **giao diện của cửa sổ macOS**. Vì vậy chỉ cần đổi giao diện của **cả app** bằng `setTheme` của Tauri (`@tauri-apps/api/app`):
  - `setTheme("dark")` / `setTheme("light")`: ép một chế độ;
  - `setTheme(null)`: theo hệ thống.
- Kết quả: CSS, Monaco, thanh tiêu đề, menu native, cửa sổ mở sau… **tự đổi theo**, không phải sửa từng chỗ. Không cần thêm thuộc tính `data-theme` hay bộ màu thứ ba.

```
View → Theme → Dark → chooseTheme("dark")      (appMenu.ts)
   ├─ đặt ✓ cho đúng mục (check item tự đảo khi bấm, nên đặt lại cả ba)
   └─ applyThemeChoice("dark")                  (theme.ts)
        ├─ localStorage "moongit.theme" = "dark"
        └─ setTheme("dark") → macOS đổi giao diện app
              └─ khung web: prefers-color-scheme = dark
                    ├─ styles.css dùng bộ màu tối
                    └─ monaco.ts: onThemeChange → vs-dark
Lúc mở app: App.vue → applyThemeChoice(readThemeChoice())
```

## File

| File | Việc |
|---|---|
| `src/theme.ts` (mới) | `readThemeChoice`, `applyThemeChoice` |
| `src/appMenu.ts` | Theme ▸ System / Light / Dark (hành xử như nút radio) |
| `src/App.vue` | Áp dụng lựa chọn đã lưu khi mở app |
| `src-tauri/capabilities/default.json` | Quyền `core:app:allow-set-app-theme` (không nằm trong quyền mặc định) |

## Khái niệm mới
- **`prefers-color-scheme`**: điều kiện CSS "hệ thống đang sáng hay tối". Trong app Tauri, "hệ thống" là giao diện của cửa sổ, nên app tự đổi được.
- **Biến CSS (`--bg`, `var(--bg)`)**: định nghĩa màu một chỗ, dùng khắp nơi; đổi theme = đổi giá trị biến, không phải sửa từng component. Giống hằng số dùng chung, nhưng đổi được lúc chạy.
- **`as const` cho mảng cặp** (`[["system", "System"], …] as const`): TS hiểu `"system"` là đúng kiểu `ThemeChoice`, không phải `string` chung chung.

## Lưu ý
- Lúc mở app với Light/Dark đã chọn, cửa sổ có thể hiện theo macOS trong chớp mắt rồi mới đổi (lựa chọn được đọc khi giao diện đã chạy).
- Ảnh màn hình chào là cảnh đêm, viền mờ dần, nên hợp cả hai chế độ.

## Cách thử

App dev đã build lại. Bấm ⇧⌘R nếu menu chưa có Light/Dark bật.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | View → Theme → Dark | Cả app tối ngay, ✓ ở Dark; thanh tiêu đề cũng tối |
| 2 | Mở diff một file (double-click) | Cửa sổ diff và editor Monaco cũng tối |
| 3 | View → Theme → Light (khi macOS đang tối) | App sáng, macOS vẫn tối |
| 4 | View → Theme → System | Theo macOS; đổi macOS sáng/tối → app đổi theo |
| 5 | Chọn Dark, thoát app, mở lại | Vẫn tối, ✓ ở Dark |
| 6 | Chuột phải trong sidebar | Menu native cùng màu với app |
