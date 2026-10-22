# Bước 19b — Đổi tên, xóa branch, đặt / bỏ upstream

## Trạng thái
- `cargo test`: 129/129 pass. Thêm 6 test:
  - đổi tên (kể cả branch hiện tại, upstream đi theo; tên trùng/sai; không đổi tên remote branch);
  - xóa branch đã merge; branch chưa merge cần `force`;
  - không xóa được branch đang checkout;
  - xóa branch trên server (cả `origin/x` ở máy cũng mất);
  - tách tên remote có `/`;
  - đặt / bỏ upstream.
- `cargo clippy`: sạch. `pnpm test`: 17/17 (thêm 2 test cho việc lựa chọn ref đi theo tên mới). `pnpm build`: OK.
- Giao diện: bạn đã chấp nhận (2026-10-22). Đã commit.

## Mục tiêu
**Menu tên branch ở header** (`⎇ main ▾`), dưới phần checkout của bước 19a:
- **Rename "main"…**: đổi tên branch hiện tại.
- **Upstream of "main" ▸**: chọn remote branch để theo dõi (✓ ở cái đang theo dõi), hoặc **Stop Tracking**.
- **Delete Branch ▸**: các branch local, trừ branch đang checkout.
- **Delete Remote Branch ▸**: các branch trên server.

**History → chuột phải một commit** có thêm, cho các branch trỏ vào commit đó:
- **Rename x…**, **Delete x…**;
- **Delete origin/x on the Server…**.

**Mọi thao tác xóa đều hỏi trước** (CLAUDE.md):
- **Xóa branch local**: hỏi "Delete the branch …?".
  - Nếu git báo branch có commit **không nằm trong branch nào khác**, app hỏi thêm lần nữa, rõ hơn: "… Delete anyway?" (**Delete Anyway** / **Keep Branch**). Chỉ khi bạn đồng ý lần hai mới xóa cưỡng bức.
- **Xóa branch trên server**: hỏi, kèm cảnh báo "biến mất với mọi người dùng repo này".

## Lệnh git

| Việc | Lệnh |
|---|---|
| Đổi tên | `git branch --move --end-of-options <cũ> <mới>` |
| Xóa (an toàn) | `git branch --delete --end-of-options <tên>` |
| Xóa cưỡng bức (sau khi bạn xác nhận lần 2) | `git branch --delete --force --end-of-options <tên>` (= `git branch -D`) |
| Xóa trên server | `git push --progress --delete <remote> refs/heads/<tên>` |
| Theo dõi remote branch | `git branch --set-upstream-to=refs/remotes/origin/x --end-of-options <tên>` |
| Bỏ theo dõi | `git branch --unset-upstream --end-of-options <tên>` |

### Ghi chú
- **Xóa an toàn trước, cưỡng bức sau.**
  - `--delete` không có `--force` bị git từ chối khi branch chưa được merge vào đâu; lỗi được chuyển thành `ErrorKind::NotMerged`, và frontend dựa vào đó để hỏi lần hai.
  - Các commit sau khi xóa cưỡng bức chưa mất ngay: git còn giữ chúng một thời gian trong **reflog** (`git reflog`). Tuy vậy rất khó tìm lại, nên phải hỏi kỹ.
- **Không xóa được branch đang checkout**: app báo trước khi gọi git.
- **Đổi tên** không đổi tên branch trên server; upstream vẫn trỏ vào tên cũ trên server (có test). Muốn đổi cả trên server thì: push branch mới, rồi xóa branch cũ trên server.
- **Xóa trên server**:
  - Dùng tên ref đầy đủ `refs/heads/<tên>`, để một **tag** trùng tên trên server không bao giờ bị xóa nhầm.
  - git cũng xóa luôn `origin/<tên>` ở máy bạn.
  - Chạy qua `run_git_with_progress` như fetch/push (process group riêng, lỗi đăng nhập thành `AuthRequired`).
- **Tên remote có thể chứa `/`** (ví dụ `team/origin`): `origin/feature/x` được tách theo **tên remote dài nhất khớp** (`split_remote_name`), không đơn giản cắt ở dấu `/` đầu tiên.
- **Mọi tên đều được so với `list_refs`** trước khi chạy git, như các bước trước; sau đó còn có `--end-of-options`.

### Lựa chọn ref của History đi theo tên mới
Bước 16c lưu lựa chọn ref theo tên đầy đủ (`refs/heads/main`). Đổi `main` → `trunk` mà không sửa gì thì tên cũ trong lựa chọn không còn tồn tại, và History sẽ âm thầm thôi hiện branch đó. Vì vậy:
- Khi đổi tên thành công, `renameInSavedSelection` thay tên cũ bằng tên mới trong lựa chọn đã lưu.
- HistoryView đọc lại lựa chọn đã lưu mỗi lần status thay đổi (`reloadAll`). Nhờ vậy đổi tên từ menu header khi đang mở tab History cũng được cập nhật.
- Các hàm đọc/ghi lựa chọn chuyển từ HistoryView sang `refSelection.ts` để dùng chung.

## Luồng dữ liệu

```
Menu header → Delete Branch ▸ feature
  └─ confirmDeleteLocal → ask("Delete the branch …?")
       └─ deleteBranch(repoId, "refs/heads/feature", force=false) → invoke("delete_branch")
            └─ Rust git::branch::delete_local → git branch --delete …
                 ├─ OK → RepoStatus → onStatus → RepoView applyStatus
                 └─ "not fully merged" → ErrorKind::NotMerged
       └─ notMerged → ask("… Delete anyway?") → deleteBranch(…, force=true) → --delete --force
Menu header → Rename "main"… → NameDialog → renameBranch → git branch --move
  └─ renameInSavedSelection (localStorage) → onStatus → statusVersion++ → HistoryView reloadAll đọc lại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/branch.rs` | `rename`, `delete_local`, `delete_remote`, `split_remote_name`, `set_upstream`, `find_local` + 6 test |
| `src-tauri/src/commands/branch.rs` | `rename_branch`, `delete_branch`, `delete_remote_branch`, `set_branch_upstream` |
| `src-tauri/src/error.rs`, `src/types/error.ts` | `NotMerged` |
| `src/api/branch.ts` | 4 wrapper mới |
| `src/components/branchMenus.ts` | thêm Rename / Upstream / Delete vào hai menu; hộp thoại xác nhận 2 bước |
| `src/components/refSelection.ts` (+ test) | đọc/ghi lựa chọn đã lưu, `renameInSavedSelection` |
| `src/views/HistoryView.vue` | dùng các hàm trên; đọc lại lựa chọn khi status đổi |

## Khái niệm mới

### Rust
- **`filter_map`**: vừa lọc vừa biến đổi. Closure trả về `Some(x)` thì giữ `x`, trả `None` thì bỏ. Bên trong dùng `?` với `Option` (`strip_prefix(...)?`) để bỏ qua remote không khớp.
- **`max_by_key`**: lấy phần tử có khóa lớn nhất (ở đây là tên remote dài nhất), như `stream().max(Comparator.comparing(...))`.
- **Match nhánh `None if …`**: `None if branch.upstream.is_none() => {}` chỉ khớp khi không có upstream; `None` còn lại (có upstream) mới chạy lệnh bỏ.

### TS
- **Hai lần xác nhận nối tiếp**: `await ask(...)` lần một → gọi lệnh → bắt lỗi `notMerged` → `await ask(...)` lần hai. `async/await` cho phép viết luồng hỏi–đáp nhiều bước như code tuần tự.
- **Giả `localStorage` trong test**: Vitest chạy trong Node, không có `localStorage` của trình duyệt. Test gán một object nhỏ có `getItem` / `setItem` dựa trên `Map`, đủ cho code đang thử.

## Cách thử

App dev đang **tắt**, vì lần trước Claude Code đã dừng nó khi máy thiếu bộ nhớ. Hãy chạy:

```
! export PATH="$HOME/.cargo/bin:$PATH" DEVELOPER_DIR=/Library/Developer/CommandLineTools; pnpm tauri dev
```

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `branchy` → `⎇ main ▾` → **Delete Branch ▸ hotfix** | Hỏi → Delete → `hotfix` biến mất (đã merge nên không hỏi thêm) |
| 2 | **Delete Branch ▸ topic** | Hỏi → Delete → hỏi lần 2 "has commits that are not in any other branch … Delete anyway?" → **Keep Branch**: `topic` vẫn còn |
| 3 | Làm lại, chọn **Delete Anyway** | `topic` biến mất; commit `t1` không còn trong History |
| 4 | `⎇ main ▾` → **Rename "main"…** → `trunk` | Header `⎇ trunk`; History vẫn hiện branch đó (lựa chọn ref đã đi theo tên mới) |
| 5 | Đổi lại thành `main` từ **History → chuột phải** commit có nhãn → **Rename trunk…** | Như trên |
| 6 | Không có lựa chọn **Delete** cho branch đang checkout | Đúng: menu không liệt kê nó |
| 7 | Repo có remote mà bạn được phép xóa branch (repo thử trên GitHub) → push một branch `try-delete` (bước 11) → `⎇ ▾` → **Delete Remote Branch ▸ origin/try-delete** | Hỏi "Delete … on the server?" → sau đó branch biến mất trên GitHub và khỏi menu |
| 8 | `⎇ main ▾` → **Upstream of "main" ▸ Stop Tracking** | Header: `(no upstream)`; ↑/↓ biến mất |
| 9 | **Upstream of "main" ▸ origin/main** | Header lại `→ origin/main` |
