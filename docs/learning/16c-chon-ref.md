# Bước 16c — Chọn branch / remote / tag hiển thị trong History

## Trạng thái
- `cargo test`: 104/104 pass (thêm 6 test: liệt kê branch/remote/tag kể cả tag annotated và bỏ `origin/HEAD`, HEAD detached đứng đầu, repo rỗng; lịch sử theo ref được chọn, bỏ qua ref không tồn tại hoặc trông như option, chọn HEAD detached). `cargo clippy`: sạch. `pnpm test`: 15/15 (thêm 4 test cho `currentBranchRefs`). `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-22). Đã commit.

## Mục tiêu
- Trên tab History có nút **"All refs ▾"**. Bấm vào mở bảng chọn:
  - Ô lọc theo tên.
  - Ba lựa chọn nhanh: **All** · **Current branch** (branch đang checkout + upstream của nó) · **None**.
  - Ba nhóm checkbox: **Local branches**, **Remote branches**, **Tags**; mỗi nhóm có "all / none". Màu giống nhãn trong History.
- Nút hiện `3 of 12 refs` (viền xanh) khi không xem tất cả.
- **Lần đầu** mở History của một repo: chỉ chọn **branch hiện tại + upstream** (yêu cầu của bạn sau khi thử lần đầu). Từ đó về sau, lựa chọn được **nhớ theo từng repo**.
- "All" nghĩa là *mọi ref, kể cả branch tạo sau này*. Nếu bạn tick lại hết từng cái, app tự quay về "All".
- Bảng đóng khi bấm ra ngoài hoặc nhấn Esc.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Liệt kê ref | `git for-each-ref --format=%(refname)%1f%(HEAD)%1f%(upstream) refs/heads refs/remotes refs/tags` |
| HEAD có detached không | `git symbolic-ref --quiet HEAD` (lỗi = detached) |
| Lịch sử của các ref được chọn | `git log … --end-of-options refs/heads/main refs/tags/v1 … --` |
| Lịch sử tất cả (như 16a) | `git log … --branches --remotes --tags HEAD --` |

### Ghi chú
- **`%1f` chứ không phải `%x1f`**: `for-each-ref` viết ký tự theo mã hex là `%xx`, khác `git log` (`%x1f`). Tôi thử trên máy trước khi viết parser; `%x1f` cho ra nguyên văn chữ "%x1f".
- **`%(HEAD)`** in `*` cho branch đang checkout; **`%(upstream)`** in tên đầy đủ của branch mà nó theo dõi (`refs/remotes/origin/main`), rỗng nếu không có.
- **Tag annotated** (`git tag -a`) là object riêng trỏ tới commit; `git log refs/tags/v2.0` vẫn tự lần tới commit đó.

### An toàn khi nhận danh sách ref từ frontend
Danh sách ref được chọn đến từ frontend, và còn được lưu lại giữa các lần mở app. Rust kiểm tra hai lớp:
1. **Chỉ giữ ref đang tồn tại** (`existing_refs`): so với kết quả `list_refs`. Branch đã bị xóa từ lần trước bị bỏ qua (nếu không git sẽ báo lỗi), và giá trị lạ như `--all` hay `-n1` không bao giờ tới được git.
2. **`--end-of-options`** (git ≥ 2.24): mọi thứ phía sau đều là tên revision, kể cả khi bắt đầu bằng `-`. Đây là lớp thứ hai; chỉ cần lớp 1 thôi đã đủ.

Không còn ref nào hợp lệ → trả danh sách rỗng ngay, không gọi git. History hiện "No branches or tags selected".

### Mặc định lần đầu: branch hiện tại
- `localStorage` của repo chưa có khóa `moongit.history.refs.<repoId>` → **chưa từng chọn**. Khác với giá trị `null` đã lưu, nghĩa là bạn đã bấm "All".
- Khi chưa từng chọn, HistoryView **tải danh sách ref trước**, tính `currentBranchRefs(refs)` (branch có `current` + `upstream` của nó nếu tồn tại), gán làm lựa chọn. `watch` sẽ lưu lại và tải lịch sử. Không có lần nào tải "tất cả" rồi mới thu hẹp, nên màn hình không bị nháy.
- Upstream lấy từ `for-each-ref` (`%(upstream)`), không lấy từ status mà RepoView truyền xuống. Nếu lấy từ status, tab History mở trước khi status tải xong sẽ chọn thiếu `origin/main` và lưu luôn cái thiếu đó.
- Repo chưa có commit: không có branch nào để chọn → chưa quyết định, lần sau mở lại sẽ tính lại.
- `currentBranchRefs` nằm trong `src/components/refSelection.ts`, dùng chung cho nút **Current branch** và cho mặc định, có 4 test Vitest.

### Dùng chung code phân loại ref
`git/refs.rs` có `ref_info()` (tên đầy đủ → loại + tên ngắn, bỏ `origin/HEAD`). `git/log.rs` trước đây có một bản gần giống (`classify`); giờ `classify` gọi lại `ref_info`, nên quy tắc chỉ nằm ở một chỗ.

## Luồng dữ liệu

```
HistoryView mount / statusVersion đổi → reloadAll()
  ├─ listRefs(repoId) → invoke("list_refs") → git::refs::list_refs → for-each-ref → refs
  └─ reload() → getLog(repoId, 0, n, refSelection)
                 └─ Rust read_log(…, Some(names)) → existing_refs → git log --end-of-options <names> --
RefFilter: tick/bỏ tick → choose(names) → refSelection = mảng mới (hoặc null = All)
  └─ HistoryView watch(refSelection) → lưu localStorage "moongit.history.refs.<repoId>" → reload()
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/refs.rs` (mới) | `list_refs`, `ref_info`, `is_detached` + 3 test |
| `src-tauri/src/git/log.rs` | tham số `refs`, `existing_refs`, `--end-of-options`; `classify` dùng `ref_info` + 3 test |
| `src-tauri/src/commands/log.rs` | `get_log` nhận `refs`; command mới `list_refs` |
| `src-tauri/src/models.rs` | `RefInfo` |
| `src/types/log.ts`, `src/api/log.ts` | `RefInfo`, `listRefs`, `getLog(…, refs)` |
| `src/components/RefFilter.vue` (mới) | nút + bảng chọn |
| `src/views/HistoryView.vue` | danh sách ref, lựa chọn (nhớ theo repo), tải lại khi đổi |
| `src/components/refSelection.ts` (mới) + `.test.ts` | `currentBranchRefs` + 4 test |

## Khái niệm mới

### Rust
- **`HashSet`**: tập hợp không trùng, tra cứu nhanh, như `HashSet` của Java. `known.contains(name)` thay cho việc duyệt cả danh sách.
- **`Option<&[String]>` và `as_deref()`**: command nhận `Option<Vec<String>>` (có quyền sở hữu). `refs.as_deref()` cho ra `Option<&[String]>`, tức chỉ **mượn** phần bên trong để truyền xuống, không copy.
- **Khai báo biến trước để mượn sau**: `selected` được tạo **trước** `args`, vì `args` chứa các `&str` mượn từ `selected`. Nếu khai báo sau, Rust báo lỗi "borrowed value does not live long enough". Đây là luật lifetime giữ cho tham chiếu không trỏ vào dữ liệu đã bị hủy.

### Vue
- **Drop-down tự viết**: `position: absolute` bên trong một khung `position: relative`. Bấm ra ngoài thì đóng: lắng nghe `mousedown` trên `document`, kiểm tra `root.contains(event.target)`. Listener chỉ được gắn khi bảng đang mở (`watch(open)`) và luôn được gỡ khi unmount.
- **Checkbox không dùng `v-model`**: `:checked="chosen.has(name)"` + `@change="toggle(info)"`. Trạng thái thật là một danh sách tên chung (hoặc null = all), không phải một biến boolean cho mỗi ô.
- **Giá trị đặc biệt `null` = "tất cả"**: khác với "mảng chứa mọi tên hiện có", vì "tất cả" còn gồm cả branch tạo sau này.

## Cách thử

App dev đã được khởi động lại. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo **chưa từng đổi lựa chọn** (ví dụ `branchy` nếu bạn chưa bấm gì ở bước trước; hoặc Add Repository một repo mới) → History | Nút hiện `1 of 4 refs` (hoặc `2 of …` nếu có remote): chỉ lịch sử branch hiện tại |
| 2 | Bấm nút → tick thêm `topic` | Commit `t1` xuất hiện; đóng/mở lại app vẫn giữ |
| 3 | **Current branch** | Chỉ còn lịch sử của `main` (vẫn có f1, f2, h1 vì đã được merge vào main) |
| 4 | **None** | "No branches or tags selected" |
| 5 | **All**, rồi chuyển repo khác và quay lại | Vẫn là All: đã chọn thì không quay về mặc định nữa; mỗi repo giữ riêng |
| 6 | Gõ `fea` vào ô lọc | Chỉ còn `feature`; "all/none" của nhóm chỉ tác động lên kết quả lọc |
| 7 | Chọn chỉ `topic`; trong terminal `git branch -D topic`; quay lại app | Không lỗi; History trống ("No commits to show"), nút hiện `0 of 3 refs` |
| 8 | Repo có remote (`moon-git` nếu đã push) → Current branch | `main` + `origin/main` được tick |
| 9 | Bấm ra ngoài bảng / nhấn Esc | Bảng đóng |
