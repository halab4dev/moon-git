# Bước 19a — Checkout và tạo branch

Bước 19 của CLAUDE.md (tạo, checkout, đổi tên, xóa, tracking) được chia làm hai:
- **19a (bước này):** checkout, tạo branch mới.
- **19b:** đổi tên, xóa local/remote (có hộp thoại xác nhận), đặt / bỏ upstream.

## Trạng thái
- `cargo test`: 123/123 pass. Thêm 6 test:
  - checkout branch local;
  - checkout branch remote → tạo branch theo dõi; lần sau chỉ switch;
  - từ chối khi đã có branch local trùng tên nhưng không theo dõi remote đó;
  - thay đổi chưa commit cản đường → dừng, không mất dữ liệu;
  - tạo từ HEAD / từ commit cũ / từ remote (có tracking);
  - tên không hợp lệ hoặc đã tồn tại.
- `cargo clippy`: sạch. `pnpm test`: 15/15. `pnpm build`: OK.
- Giao diện: bạn đã chấp nhận (2026-10-22). Đã commit.

## Mục tiêu
- **Tên branch ở đầu màn hình repo** (`⎇ main ▾`) giờ là một nút. Bấm vào mở menu:
  - mọi branch local, có dấu ✓ ở branch hiện tại; bấm một branch để **checkout**;
  - submenu **Remote Branches**: bấm `origin/feature` → tạo branch local `feature` **theo dõi** `origin/feature` rồi checkout. Nếu branch đó đã có thì chỉ checkout;
  - **New Branch…**: tạo branch tại commit hiện tại.
- **History → chuột phải một commit**:
  - **Create Branch Here…**: tạo branch tại commit đó;
  - **Check Out \<branch\>**: cho mỗi branch / remote branch đang trỏ vào commit đó.
- Hộp thoại **New Branch**: ô tên + checkbox **Check out the new branch** (mặc định bật). Lỗi (tên sai, đã tồn tại, thay đổi chưa commit cản đường…) hiện **ngay trong hộp thoại**, hộp thoại không đóng.
- **Không bao giờ ép buộc.** Nếu file đang sửa dở bị đè khi checkout, git từ chối, và app hiện thông báo của git ("commit your changes or stash them…"). Không có `--force`, không mất dữ liệu.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Checkout branch local | `git switch --quiet --no-guess --end-of-options <tên>` |
| Checkout remote (lần đầu) | `git switch --quiet --create <tên> --track --end-of-options refs/remotes/origin/<tên>` |
| Tạo + checkout | `git switch --quiet --create <tên> --no-track\|--track --end-of-options <điểm bắt đầu>` |
| Chỉ tạo | `git branch --no-track\|--track --end-of-options <tên> <điểm bắt đầu>` |
| Kiểm tra tên | `git check-ref-format --branch <tên>` |

### Ghi chú
- **`git switch`** thay cho `git checkout`: `checkout` vừa đổi branch vừa khôi phục file, một lệnh hai nghĩa. `switch` chỉ đổi branch nên an toàn hơn.
- **`--no-guess`**: không để git "đoán". Thiếu nó, `git switch feature` khi chưa có branch local `feature` sẽ tự tạo từ `origin/feature`. App làm việc này một cách rõ ràng ở nhánh Remote.
- **Đặt tên branch local khi checkout remote**: bỏ phần tên remote, `origin/feature/login` → `feature/login`.
  - Nếu đã có `feature/login` **và nó theo dõi đúng** `origin/feature/login` → chỉ checkout.
  - Nếu có nhưng **không** theo dõi remote đó → báo lỗi, không tự đè.
- **Tracking**: tạo từ một remote branch → `--track` (branch mới theo dõi remote đó, như SmartGit). Tạo từ commit hay branch local → `--no-track`. Dùng `--no-track` rõ ràng để cấu hình `branch.autoSetupMerge` của bạn không làm khác đi.
- **Kiểm tra tên** bằng chính quy tắc của git (`check-ref-format --branch`): không dấu cách, `..`, `~`, `^`, `:`, `?`, `*`, `[`, không kết thúc bằng `.lock`… App còn tự từ chối tên rỗng, tên bắt đầu bằng `-` và `HEAD` trước khi gọi git.
- **Điểm bắt đầu** chỉ được là commit id (hex) hoặc một ref đang tồn tại (so với `list_refs`), giống cách bước 16c xử lý ref.

## Frontend
- **`branchMenus.ts`** (mới), giống `sidebarMenus.ts`: `showBranchMenu`, `showCommitMenu`, `askNewBranch`. Kết quả báo về qua hai callback `onStatus` (status mới) / `onError`, nên dùng được từ cả RepoView và HistoryView.
- **Menu native có dấu ✓**: phần tử có thuộc tính `checked` thành "check menu item". Submenu là phần tử có `items`. Đường kẻ là `{ item: "Separator" }`.
- **`NameDialog`** có thêm checkbox tùy chọn (`option` trong `NameRequest`); `submit(name, optionChecked)`. Các chỗ dùng cũ (tạo/đổi tên nhóm) không cần sửa.
- **RepoView `applyStatus`**: nhận status mới, tăng `statusVersion`, nên bảng file, diff và History đều tải lại. HistoryView không tự sửa status mà `emit("statusChanged")` lên RepoView, để status chỉ có một nơi giữ.
- **CommitList**: chuột phải một commit → chọn nó trước, rồi `emit("rowMenu", commit)` (giống bảng file ở bước 7).

## Luồng dữ liệu

```
Bấm "⎇ main ▾" → BranchStatus emit("openMenu") → RepoView openBranchMenu
  └─ showBranchMenu → listRefs → Menu.new([...branch local ✓, Remote Branches ▸, New Branch…])
       └─ bấm "feature" → checkoutBranch(repoId, "refs/heads/feature") → invoke("checkout_branch")
            └─ Rust git::branch::checkout → list_refs → git switch … → read_status
       └─ callbacks.onStatus(status) → RepoView applyStatus → statusVersion++ → mọi thứ tải lại
History: chuột phải commit → CommitList emit("rowMenu") → HistoryView openCommitMenu
  └─ showCommitMenu → "Create Branch Here…" → askNewBranch → NameDialog
       └─ submit(name, checkOut) → createBranch(repoId, name, commitId, checkOut)
            └─ Rust git::branch::create → check_branch_name → git switch --create … / git branch …
       └─ onStatus → HistoryView emit("statusChanged") → RepoView applyStatus
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/branch.rs` (mới) | `checkout`, `checkout_remote`, `create`, `check_branch_name` + 6 test |
| `src-tauri/src/commands/branch.rs` (mới) | `checkout_branch`, `create_branch` |
| `src/api/branch.ts` (mới) | wrapper |
| `src/components/branchMenus.ts` (mới) | menu branch, menu commit, hộp thoại New Branch |
| `src/stores/dialogs.ts`, `src/components/NameDialog.vue` | checkbox tùy chọn |
| `src/components/BranchStatus.vue` | tên branch thành nút mở menu |
| `src/components/CommitList.vue` | chuột phải → `rowMenu` |
| `src/views/HistoryView.vue` | menu commit, `emit("statusChanged")` |
| `src/views/RepoView.vue` | `applyStatus`, mở menu branch |

## Khái niệm mới

### Rust
- **Lifetime trong hàm trả về tham chiếu** `fn find<'a>(refs: &'a [RefInfo], …) -> Result<&'a RefInfo, …>`: kết quả **mượn** từ `refs`, nên sống được bao lâu thì phụ thuộc vào `refs`. Rust bắt mình ghi rõ quan hệ đó bằng `'a`. Java không cần, vì garbage collector giữ object còn được tham chiếu.
- **Match guard với điều kiện** `Some(start) if start.starts_with("refs/") => …`: một nhánh `match` chỉ khớp khi điều kiện đúng; không đúng thì thử nhánh kế tiếp (`Some(commit) => …`).
- **`split_once('/')`** + `map(|(_, rest)| rest)`: tách tại dấu `/` đầu tiên, lấy phần sau; `_` bỏ qua phần trước.

### Vue / TS
- **Callback object** (`BranchCallbacks`): truyền một object có hàm `onStatus` / `onError` cho code không phải component (menu native), để nó báo kết quả về đúng component gọi nó. Giống truyền một listener interface trong Java.
- **Spread có điều kiện trong mảng** `...(remote.length > 0 ? [submenu, separator] : [])`: thêm phần tử chỉ khi cần, ngay trong literal mảng.
- **`as const`** ở `{ item: "Separator" as const }`: báo TypeScript rằng đây là đúng chữ `"Separator"` (một kiểu cụ thể), không phải `string` bất kỳ, để khớp với kiểu menu của Tauri.

## Cách thử

App dev đã được khởi động lại. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `branchy` → bấm `⎇ main ▾` | Menu: feature, hotfix, ✓ main, topic; New Branch… (không có Remote Branches vì repo không có remote) |
| 2 | Bấm `feature` | Header thành `⎇ feature`; bảng Changes, History cập nhật |
| 3 | `⎇ … ▾` → **New Branch…** → gõ `my test` | Lỗi "not a valid branch name" trong hộp thoại; hộp thoại không đóng |
| 4 | Sửa thành `my-test`, để tick "Check out" → Create | Header `⎇ my-test` |
| 5 | History → chuột phải một commit cũ → **Create Branch Here…** → `old-point`, **bỏ tick** Check out | Branch vẫn là `my-test`; nhãn `old-point` xuất hiện ở commit đó (bật nó trong **refs ▾** nếu đang lọc "Current branch") |
| 6 | Chuột phải commit có nhãn `topic` → **Check Out topic** | Header `⎇ topic` |
| 7 | Sửa một file mà branch khác cũng sửa (ví dụ trong `branchy`: sửa `f1.txt` trên `main`), rồi checkout `feature` | Lỗi của git: "Your local changes … would be overwritten … commit your changes or stash them"; file của bạn **vẫn nguyên** |
| 8 | Repo có remote (`moon-git`) → `⎇ main ▾` → Remote Branches → `origin/main` | Vì `main` đã theo dõi `origin/main` → chỉ checkout `main` (không báo lỗi) |
| 9 | (Nếu có một remote branch chưa có bản local) chọn nó | Tạo branch local cùng tên, `→ origin/…` hiện trên header |
