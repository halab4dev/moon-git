# Bước 30 — "Today" / "Yesterday" trong cột thời gian của History

*Bạn yêu cầu (2026-10-26).*

## Trạng thái
- Chỉ đổi frontend. `pnpm test`: 35/35 (thêm 5 test cho `formatCommitTime`). `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit.

## Thay đổi
Cột thời gian của danh sách commit (tab History):

| Commit lúc | Trước | Giờ |
|---|---|---|
| hôm nay 14:32 | `26/10/2026, 14:32` | `Today, 14:32` |
| hôm qua 09:10 | `25/10/2026, 09:10` | `Yesterday, 09:10` |
| cũ hơn | `24/10/2026, 18:00` | không đổi |

- "Hôm nay / hôm qua" tính theo **ngày lịch** ở múi giờ của máy, không phải "trong vòng 24 giờ". Commit 23:50 tối qua, xem lúc 10:00 sáng nay, là "Yesterday".
- Thứ tự ngày/tháng và giờ 12h/24h vẫn theo cài đặt của macOS (`Intl.DateTimeFormat` với locale mặc định), như trước.
- Chỉ áp dụng cho danh sách History, như bạn yêu cầu. Chi tiết commit và danh sách stash vẫn hiện ngày đầy đủ.
- Nếu để app mở qua nửa đêm, nhãn chỉ cập nhật khi danh sách tải lại (Refresh, hoặc sau một thao tác git).

## File

| File | Việc |
|---|---|
| `src/components/commitTime.ts` (mới) | `formatCommitTime(seconds, now)` |
| `src/components/commitTime.test.ts` (mới) | 5 test: hôm nay, hôm qua (kể cả < 24h), cũ hơn, qua tháng / qua năm, giờ tương lai |
| `src/components/CommitList.vue` | dùng `formatCommitTime` thay cho format cũ |

## Khái niệm
- **`new Date(năm, tháng, ngày - 1)`**: JS tự xử lý ngày 0 hay số âm. `new Date(2026, 0, 0)` là 31/12/2025. Nhờ vậy "hôm qua" luôn đúng qua đầu tháng, đầu năm, và cả ngày đổi giờ mùa hè (ngày đó dài 23 hoặc 25 giờ, nên "trừ 24 giờ" sẽ sai).
- **Tham số `now` có giá trị mặc định** (`now: Date = new Date()`): code thật không cần truyền, còn test truyền một thời điểm cố định nên kết quả không phụ thuộc lúc chạy test. Giống việc inject `Clock` trong Java (`java.time.Clock`).
- Tháng trong `Date` của JS **bắt đầu từ 0** (tháng 10 là `9`). Test dùng hàm `at(y, m, d, h, min)` nhận tháng 1–12 cho dễ đọc.

## Cách thử
Bấm ⌘R, mở tab History.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Commit mới hôm nay | Cột thời gian: `Today, HH:MM` |
| 2 | Commit hôm qua (nếu có) | `Yesterday, HH:MM` |
| 3 | Commit cũ hơn | Ngày đầy đủ như trước |
| 4 | Chọn dòng (nền xanh) | Chữ vẫn trắng, đọc được |
