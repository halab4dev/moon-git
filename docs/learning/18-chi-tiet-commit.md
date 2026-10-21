# Bước 18 — Bấm vào commit: thông tin, danh sách file, diff

## Trạng thái
- `cargo test`: 117/117 pass (thêm 6 test: parse `--name-status -z`, đọc message + tác giả + committer + file (sửa, đổi tên, thêm), commit đầu tiên, merge so với cha thứ nhất, chỉ nhận commit id, diff một file của commit). `cargo clippy`: sạch. `pnpm test`: 15/15. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-22). Đã commit.

## Mục tiêu
- Tab History chia **trên/dưới** bằng thanh kéo (tỉ lệ được nhớ): danh sách commit ở trên, **chi tiết** commit đang chọn ở dưới.
- Phần chi tiết:
  - **Bên trái**:
    - **tiêu đề** (đậm) và **thân** message;
    - **Commit** (id đầy đủ, bôi đen để copy được);
    - **Parent(s)**: bấm để nhảy tới commit cha;
    - **Author** (tên, email, thời gian);
    - **Committer**: chỉ hiện khi khác Author, ví dụ sau rebase hoặc cherry-pick.
  - **Danh sách file** thay đổi, có chữ cái màu A/M/D/R/C/T giống tab Changes. File đầu tiên được chọn sẵn; dùng ↑/↓ để đổi file.
  - **Bên phải**: diff của file đang chọn (cha → commit), cùng Monaco và cùng tùy chọn Side by side / Ignore whitespace như tab Changes.
- **Merge commit** được so với **cha thứ nhất**, tức "merge này đã đưa gì vào branch", như SmartGit mặc định.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Thông tin commit | `git show --no-patch --no-show-signature --no-color --format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%cN%x1f%cE%x1f%ct%x1f%B <id>^{commit}` |
| File thay đổi | `git diff-tree -r -z -M --name-status --no-commit-id <cha> <id>` |
| File của commit đầu tiên | `git diff-tree -r -z -M --name-status --no-commit-id --root <id>` |
| Nội dung một file | `git cat-file blob <cha>:<đường dẫn cũ>` và `git cat-file blob <id>:<đường dẫn>` |

### Ghi chú
- **`^{commit}`**: bắt id phải là một **commit**. Nếu không có nó, id của một file (blob) sẽ khiến `git show` in nội dung file đó ra.
- **`diff-tree`** là lệnh "plumbing" (dành cho chương trình), **không** bị ảnh hưởng bởi cấu hình diff của bạn (`diff.external`, màu…), nên output luôn ổn định.
  - `-M` phát hiện đổi tên.
  - Định dạng `-z --name-status`: `M\0đường/dẫn\0` hoặc `R087\0tên-cũ\0tên-mới\0`. Số sau `R` là độ giống nhau (%).
- **Chỉ nhận commit id** (`check_commit_id`): 4–64 ký tự hex. Tên branch, `HEAD`, hay chuỗi bắt đầu bằng `-` bị từ chối trước khi chạy git (có test với `--all`, `main`, `HEAD`).
- **Commit không còn tồn tại** (ví dụ vừa bị amend hoặc rebase) → `NotFound` kèm thông báo dễ hiểu.
- Diff của commit dùng lại `read_blob` của bước 8: giới hạn 5 MB, phát hiện file nhị phân, file mới/xóa có một bên trống.

## Frontend
- **`DiffView.vue`** (mới, tách từ `DiffPanel`): phần **hiển thị** diff (thông báo nhị phân/quá lớn, Monaco, hai tùy chọn được nhớ). Header có một **slot** `controls` để component cha tự đặt nút của mình vào:
  - `DiffPanel` (tab Changes) đặt các nút Working Tree / Staged / Conflict.
  - `CommitDetails` không đặt gì.
- **`CommitDetails.vue`** (mới): tải chi tiết khi `commitId` đổi, tải diff khi đổi file. Mỗi loại có bộ đếm request riêng để bỏ câu trả lời đến muộn.
- **`useSplitRatio`** (composable mới, `src/composables/`): logic thanh kéo trên/dưới trước đây nằm trong RepoView; giờ RepoView và HistoryView cùng dùng.
- **HistoryView**:
  - `shownCommit` (commit đang xem chi tiết) tách khỏi `selected` (dòng đang chọn trong danh sách).
  - Bấm vào **Parent** → `showCommit(id)`: hiện chi tiết ngay, đồng thời tìm và chọn commit đó trong danh sách (tải thêm trang nếu cần).
  - Nếu cha không có trong danh sách (bị filter ẩn, hoặc không thuộc ref đang chọn), phần chi tiết vẫn hiện nó.
  - Đoạn "tải từng trang tới khi thấy commit" được tách khỏi `jump()` (bước 17) thành `revealCommit()`, dùng chung.

## Luồng dữ liệu

```
CommitList click → selected = id → (watch) shownCommit = id
CommitDetails watch(commitId)
  └─ getCommitDetails(repoId, id) → invoke("get_commit_details")
       └─ Rust git::show::commit_details → check_commit_id → git show … <id>^{commit}
            └─ changed_files → git diff-tree … <cha> <id> → parse_name_status
  └─ selectedPath = file đầu tiên
CommitDetails watch([details, selectedFile])
  └─ getCommitFileDiff(repoId, id, cha, path, origPath) → invoke("get_commit_file_diff")
       └─ Rust git::diff::read_commit_diff → read_blob(<cha>:<cũ>), read_blob(<id>:<mới>)
  └─ <DiffView> → <DiffEditor> (Monaco)
Bấm Parent → emit("selectCommit", cha) → HistoryView showCommit → revealCommit
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/show.rs` (mới) | `commit_details`, `parse_name_status`, `check_commit_id` + 5 test |
| `src-tauri/src/git/diff.rs` | `read_commit_diff` + 1 test |
| `src-tauri/src/commands/log.rs` | `get_commit_details`, `get_commit_file_diff` |
| `src-tauri/src/models.rs` | `CommitDetails` (dùng lại `FileChange` của status) |
| `src/types/log.ts`, `src/api/log.ts` | kiểu + wrapper |
| `src/components/DiffView.vue` (mới) | phần hiển thị diff dùng chung, có slot |
| `src/components/DiffPanel.vue` | dùng `DiffView`, chỉ còn logic chọn chế độ + tải |
| `src/components/CommitDetails.vue` (mới) | thông tin + danh sách file + diff |
| `src/composables/useSplitRatio.ts` (mới) | thanh kéo trên/dưới dùng chung |
| `src/views/RepoView.vue` | dùng `useSplitRatio` |
| `src/views/HistoryView.vue` | chia trên/dưới, `shownCommit`, `revealCommit`, `showCommit` |

## Khái niệm mới

### Rust
- **Slice pattern 9 phần tử** `let [id, parents, …, message] = fields[..] else { … }` (như bước 16a, dài hơn).
- **`(4..=64).contains(&len)`**: khoảng **bao gồm** cả 64 (`..=`), khác `4..64` (không gồm 64).
- **`format!("{commit}^{{commit}}")`**: trong `format!`, `{{` và `}}` là dấu ngoặc nhọn thật, không phải chỗ chèn biến.
- **`map_or(default, f)`**: `parent.map_or("(none)".to_string(), short_id)`. Có giá trị → gọi `f`, không có → dùng `default`. Như `Optional.map(f).orElse(default)` của Java.

### Vue
- **Slot** (`<slot name="controls">` trong con, `<template #controls>` trong cha): cha "cắm" giao diện của mình vào một chỗ trống của component con. Giống truyền một đoạn view vào layout. Style `scoped` của cha vẫn áp dụng cho phần cha cắm vào.
- **Composable** (`useSplitRatio`): một hàm thường tạo state reactive + hàm xử lý cho component dùng. Mỗi component gọi nó có **bản state riêng**. Giống một helper class mà mỗi màn hình `new` một instance. Quy ước tên bắt đầu bằng `use…`.
- **`useTemplateRef("tên")`** (Vue 3.5): lấy phần tử có `ref="tên"` trong template, dùng được bên trong composable. Kiểm tra kiểu (`vue-tsc`) cũng hiểu cách này, còn biến cùng tên với `ref="…"` thì bị báo "khai báo nhưng không dùng" khi được lấy ra từ composable.
- **Hai state cho hai ý nghĩa** (`selected` vs `shownCommit`): dòng đang chọn trong danh sách và commit đang xem chi tiết thường trùng nhau nhưng không phải lúc nào cũng vậy. Tách ra giúp xử lý được trường hợp commit cha không có trong danh sách.

## Cách thử

App dev đã được khởi động lại. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `moon-git` → History → bấm một commit | Phần dưới: tiêu đề + thân message, id, Parent, Author; danh sách file; diff của file đầu tiên bên phải |
| 2 | Bấm file khác, dùng ↑/↓ trong danh sách file | Diff đổi theo |
| 3 | Bấm commit đầu tiên của repo (dòng dưới cùng) | Không có Parent; mọi file là **A**; diff bên trái trống |
| 4 | Repo `branchy` → bấm "Merge branch 'feature'" | "Parents" có 2 id; danh sách "(compared with the first parent)" gồm f1, f2 |
| 5 | Bấm id cha thứ hai | Danh sách nhảy tới và chọn commit đó; chi tiết đổi theo |
| 6 | Filter Hide others với chữ chỉ khớp một commit → bấm Parent của nó | Chi tiết của cha vẫn hiện, dù cha không có trong danh sách |
| 7 | Kéo thanh ngang giữa danh sách và chi tiết; đóng/mở lại app | Tỉ lệ được nhớ (riêng với tab Changes) |
| 8 | Tab **Changes** | Diff hoạt động như trước (Working Tree / Staged, hai tùy chọn); tùy chọn dùng chung với History |
| 9 | Tạo một commit có file ảnh (binary) → xem trong History | "Binary file — no text to compare." |
| 10 | Bôi đen id commit hoặc email, ⌘C | Copy được |
