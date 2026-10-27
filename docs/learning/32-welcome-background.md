# Bước 32 — Ảnh nền (có chuyển động) cho màn hình chào

*Bạn yêu cầu (2026-10-27), ảnh do bạn cung cấp (`background.svg`).*

## Trạng thái
- Chỉ đổi frontend. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK. Đã commit.

## Thay đổi
- Khi chưa chọn repo nào (lúc mới mở app), vùng bên phải trước đây chỉ có chữ "MoonGit" và một dòng hướng dẫn. Giờ là **ảnh của bạn** (căn cứ trên mặt trăng, Trái Đất, các con đường như nhánh git), **căn giữa**, to nhất có thể trong cửa sổ (rộng tối đa 1400px, cao tối đa 85%), tự thu nhỏ khi cửa sổ nhỏ (giữ tỉ lệ 2:1).
- Bên dưới vẫn giữ **một dòng gợi ý nhỏ, mờ**: "Select a repository on the left, or click + to add one." Muốn bỏ hẳn thì xóa một dòng trong `WelcomeView.vue`.
- Ảnh tự **mờ dần ở các cạnh** (có sẵn trong file SVG), nên hòa vào nền cửa sổ cả ở chế độ sáng lẫn tối.
- Ảnh không kéo-thả được và không bôi đen được (để không lộ ra là ảnh khi lỡ kéo chuột).

## Làm ảnh "sống" hơn (bạn yêu cầu thêm)
Giữ nguyên mọi hình trong ảnh gốc, chỉ thêm chuyển động nhẹ và vài chi tiết:

| Chi tiết | Hiệu ứng |
|---|---|
| Sao | Lấp lánh theo 4 nhịp lệch nhau; thêm vài ngôi sao |
| Hai ngôi sao 4 cánh | Co lại, xoay nhẹ, sáng lại |
| Sao băng | Vụt qua bầu trời mỗi 9 giây |
| Dải Ngân Hà | Dải sáng mờ, nhòe chéo bầu trời (mới, tĩnh) |
| Trái Đất | Quầng sáng "thở" (to ra, nhỏ lại); thêm viền khí quyển mảnh |
| Ăng-ten chảo | Đèn đỉnh nhấp nháy; **sóng tín hiệu** phát về phía Trái Đất |
| Hai con đường (như hai nhánh git) | Vạch vàng **chạy dọc đường** như commit đang di chuyển |
| Các nút đèn trên đường | Phập phồng, lệch nhịp nhau |
| Xe tự hành | Đèn ăng-ten nhấp nháy, đèn pha chập chờn |

- Chỉ dùng **CSS animation bên trong file SVG** (`<style>` + `@keyframes`), không có script. Nhờ vậy ảnh vẫn hiện bằng thẻ `<img>` như cũ và vẫn chuyển động.
- Ai bật **Reduce motion** trong macOS (System Settings → Accessibility → Display) sẽ thấy ảnh đứng yên (`@media (prefers-reduced-motion: reduce)`).
- Chuyển động chỉ chạy khi màn hình chào đang hiện; chọn repo là ảnh biến mất.

## File

| File | Việc |
|---|---|
| `background.svg` → `src/assets/background.svg` | Chuyển vào thư mục tài nguyên của frontend |
| `src/assets/background.svg` | Thêm `<style>` animation, sao, sao băng, Ngân Hà, sóng tín hiệu |
| `src/views/WelcomeView.vue` | Hiện ảnh (to hơn) + dòng gợi ý |

Đã kiểm tra nội dung file SVG: chỉ có hình vẽ và gradient, không có script, không tải gì từ bên ngoài.

## Khái niệm mới
- **`import background from "../assets/background.svg"`**: với Vite, import một file ảnh không trả về nội dung mà trả về **đường dẫn (URL)** tới ảnh. Khi build, Vite chép file vào `dist/assets/` với tên có mã băm (`background-DmH5JP7h.svg`) để trình duyệt không dùng bản cũ trong cache. Ảnh rất nhỏ (< 4 KB) thì Vite nhúng thẳng vào code.
- **CSS animation trong SVG**: `@keyframes` mô tả các trạng thái (0%, 50%, 100%), `animation: tên thời-gian kiểu-chuyển infinite` chạy lặp. `animation-delay` âm để các ngôi sao không nhấp nháy cùng lúc.
- **`stroke-dasharray` + animate `stroke-dashoffset`**: nét đứt "trượt" dọc đường cong, tạo cảm giác có thứ đang chạy trên đường mà không phải tính vị trí.
- **`transform-box: fill-box; transform-origin: center`**: để `scale`/`rotate` xoay quanh tâm của chính hình đó (mặc định trong SVG là gốc tọa độ của cả ảnh).
- **`prefers-reduced-motion`**: tôn trọng cài đặt trợ năng của hệ điều hành.
- **`object-fit: contain`**: thu ảnh vừa khung mà không méo, giống `ImageView.ScaleType.FIT_CENTER` bên Android.

## Cách thử
App không nhớ repo đã chọn giữa các lần mở, nên lúc mới mở (hoặc sau ⇧⌘R Reload Interface) chưa có repo nào được chọn: đó chính là màn hình chào.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | ⇧⌘R (hoặc mở lại app) | Ảnh căn giữa, dòng gợi ý mờ bên dưới |
| 2 | Thu nhỏ / phóng to cửa sổ | Ảnh co giãn theo, không méo |
| 3 | Đổi macOS sang Dark mode | Viền ảnh hòa vào nền tối |
| 4 | Chọn một repo | Màn hình repo như bình thường |
