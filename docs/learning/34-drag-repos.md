# Bước 34 — Kéo-thả repo vào / ra khỏi nhóm

*Bạn yêu cầu (2026-10-28). CLAUDE.md mục 3: "di chuyển repo giữa các nhóm (kéo-thả làm sau cũng được)".*

## Trạng thái
- Chỉ đổi frontend (lệnh `move_repo` của Rust đã có từ bước 4). `pnpm test`: 38/38. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK (sau khi sửa `dragDropEnabled`). Đã commit.

## Cách dùng
Kéo một **repo** trong danh sách Repositories rồi thả:

| Thả lên | Kết quả |
|---|---|
| Một **nhóm** (dòng folder) | Repo vào nhóm đó |
| Một **repo** khác | Repo vào cùng nhóm với repo đó |
| **Khoảng trống** trong danh sách (dưới cùng) | Repo ra cấp ngoài cùng (không thuộc nhóm nào) |

- Khi kéo, repo đang kéo **mờ đi**. Chỗ sẽ thả được tô **viền + nền xanh nhạt**: dòng nhóm, hoặc cả khung danh sách khi thả ra ngoài cùng.
- Thả vào chỗ repo **đang ở sẵn**: không có gì xảy ra, không tô gì.
- **Giữ chuột trên một nhóm đang gập** khoảng 0,7 giây: nhóm tự mở ra, để thả vào nhóm con bên trong.
- Thả xong, nhóm đích (và các nhóm cha) tự mở để thấy repo.
- Menu chuột phải **Move To** vẫn còn, như trước.
- Chỉ kéo được **repo**. Kéo cả **nhóm** sang nhóm khác chưa làm (cần thêm phần Rust: kiểm tra không thả nhóm vào chính nhóm con của nó).

## Cách làm: kéo-thả có sẵn của trình duyệt (HTML5)
Không thêm thư viện. Ba sự kiện:

```
dragstart (repo bắt đầu bị kéo)   → ghi nhớ repo đang kéo
dragover  (chuột đi qua một dòng) → "thả ở đây được": event.preventDefault()  → tô chỗ đó
drop      (nhả chuột)              → store.moveRepo(repo, nhóm) → invoke("move_repo")   (bước 4)
dragend   (kết thúc, kể cả hủy)    → xóa trạng thái
```

- **Mặc định trình duyệt không cho thả.** Phải gọi `preventDefault()` trong `dragover` thì mới thả được: hơi ngược đời, nhưng là quy ước của HTML5.
- Mỗi dòng xử lý xong thì `stopPropagation()`: không để khung danh sách phía sau (nghĩa là "cấp ngoài cùng") giành lấy.
- Dữ liệu kéo có kiểu riêng `application/x-moongit-repo`: kéo chữ hay file vào sidebar sẽ không bị hiểu nhầm là repo.
- Trạng thái kéo dùng chung giữa `RepoItem`, `GroupItem` (cây đệ quy) và `AppSidebar`, nên để trong một module riêng (`repoDrag.ts`) với vài `ref`, thay vì truyền props / emit qua từng tầng cây.

## Sửa sau khi bạn thử: thả ra không có gì xảy ra
- **Nguyên nhân**: Tauri mặc định tự nhận các thứ được kéo thả vào cửa sổ (để app biết khi có **file từ Finder** được thả vào). Lớp đó nằm trên khung web, nên "nuốt" mất sự kiện `drop` của trang. Kéo vẫn thấy tô màu (`dragover` vẫn tới), nhưng `drop` thì không bao giờ tới.
- **Sửa**: `"dragDropEnabled": false` cho cửa sổ chính trong `src-tauri/tauri.conf.json`. Tài liệu của Tauri chỉ ghi là cần trên Windows, nhưng thực tế trên macOS cũng bị.
- Hệ quả: cửa sổ chính không nhận file / thư mục kéo từ Finder (app chưa dùng tính năng này). Nếu sau này muốn "kéo thư mục repo từ Finder vào để thêm", sẽ làm bằng cách khác.

## File

| File | Việc |
|---|---|
| `src/components/repoDrag.ts` (mới) | Trạng thái kéo (`draggedRepo`, `dropTarget`), `startRepoDrag`, `overDropPlace`, `dropOnPlace`, mở nhóm gập khi giữ chuột |
| `src/components/RepoItem.vue` | `draggable="true"`; là chỗ thả (cùng nhóm) |
| `src/components/GroupItem.vue` | Dòng nhóm là chỗ thả; giữ chuột thì mở nhóm gập |
| `src/components/AppSidebar.vue` | Khoảng trống = cấp ngoài cùng |
| `src/stores/repos.ts` | Xuất `revealGroup` |
| `src/styles.css` | `.dragging`, `.drop-target`, `.drop-top`; biến `--accent-soft` |
| `src-tauri/tauri.conf.json` | `dragDropEnabled: false` cho cửa sổ chính |

## Khái niệm mới
- **HTML5 drag and drop**: `draggable`, `dragstart` / `dragover` / `drop` / `dragend`, `event.dataTransfer` (`setData`, `effectAllowed`, `dropEffect`).
- **Module có `ref` dùng chung**: một file `.ts` export vài `ref` là đủ làm "trạng thái chung" cho vài component, không cần cả một Pinia store (vì trạng thái này chỉ sống trong lúc kéo).
- **Hằng màu thay cho `color-mix()`**: CSS mới `color-mix()` không có trên WebKit của macOS 12 (≈ Safari 15.6), nên màu xanh nhạt được viết sẵn thành biến `--accent-soft` cho cả hai chế độ sáng / tối.

## Cách thử

Bấm ⇧⌘R. Cần ít nhất một nhóm (chuột phải → New Group…).

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Kéo một repo ở ngoài lên một nhóm | Dòng nhóm được tô; thả → repo nằm trong nhóm, nhóm mở ra |
| 2 | Kéo repo đó xuống khoảng trống dưới cùng | Cả khung được tô viền; thả → repo ra ngoài cùng |
| 3 | Kéo repo lên một repo ở nhóm khác | Thả → repo vào nhóm đó |
| 4 | Kéo repo lên chính nhóm nó đang ở | Không tô gì, thả không có gì xảy ra |
| 5 | Giữ chuột trên một nhóm đang gập | Sau ~0,7 giây nhóm mở ra |
| 6 | Kéo rồi nhấn Esc | Hủy, không đổi gì |
| 7 | Dark mode | Màu tô vẫn rõ |
