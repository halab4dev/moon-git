# Bước 25 — Danh sách branch ở sidebar (giống SmartGit)

*Bạn yêu cầu thêm (2026-10-24), CLAUDE.md không có.*

## Trạng thái
- `cargo test`: 196/196 pass. Thêm 3 test:
  - đọc chữ `ahead 1, behind 2` / `gone` của git;
  - danh sách branch local (branch hiện tại, upstream, số commit ahead) và remote branch chia theo từng remote, kể cả remote chưa có branch nào; `origin/HEAD` bị bỏ;
  - upstream đã bị xóa trên server → `gone`.
- `cargo clippy`: sạch. `pnpm test`: 28/28. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK (gồm các sửa ở mục "Sửa sau khi bạn thử"). Đã commit.

## Mục tiêu
Sidebar chia làm hai, ở giữa có thanh kéo (tỉ lệ được nhớ):
- **trên**: danh sách repo (như trước);
- **dưới**: **Branches** của repo đang chọn.

```
BRANCHES                       ↻
▾ Local Branches              3
   ✓ main                ↑1 ↓2
     feature/login
     old-work             gone
▾ Remotes                     2
   ▾ origin                   2
       main
       feature/login
   ▾ upstream                 1
       main
```

- **Local Branches**:
  - branch đang checkout có **✓** và chữ đậm;
  - bên phải là **↑ahead ↓behind** so với upstream;
  - **gone** nếu upstream đã bị xóa trên server;
  - di chuột lên để thấy branch đó theo dõi remote branch nào.
- **Remotes**: mỗi remote một nhóm.
  - Một repo **có thể có nhiều remote**, ví dụ `origin` = fork của bạn, `upstream` = repo gốc.
  - Remote chưa fetch (chưa có branch) vẫn hiện, với dòng "No branches".
- Các nhóm gập / mở được; trạng thái gập được nhớ (cho mọi repo).
- **Double-click**: checkout. Remote branch → tạo branch local theo dõi nó (như bước 19a).
- **Chuột phải**: mở menu, dùng lại các hộp thoại đã có:

| Mục | Local | Remote | Branch hiện tại |
|---|---|---|---|
| Check Out | ✓ | ✓ | |
| Merge … into Current Branch… (bước 21) | ✓ | ✓ | |
| Rebase Current Branch onto …… (bước 22) | ✓ | ✓ | |
| New Branch from Here… | ✓ | ✓ | ✓ |
| Rename …… | ✓ | | ✓ |
| Delete …… (hỏi 2 bước nếu chưa merge) | ✓ | | |
| Delete … on the Server… | | ✓ | |

- Danh sách tự tải lại sau mọi thao tác git (commit, pull, checkout…) và khi cửa sổ được focus. Nút **↻** tải lại ngay.

### Sửa sau khi bạn thử
- **Checkout bị chặn vì thay đổi chưa commit**: trước đây hiện nguyên văn lỗi của git ("error: Your local changes … would be overwritten by checkout: … Aborting"). Giờ `branch.rs` đổi nó thành câu ngắn, nói rõ vì sao và phải làm gì, **không liệt kê file** (bạn yêu cầu; tab Changes đã hiện các file): "The branch can't be checked out: it changes 1 file that you have uncommitted changes in. Commit, stash or discard your changes first." Có câu riêng cho file untracked bị đè. Áp dụng cho mọi chỗ checkout (header, History, sidebar).
- **Lỗi của thao tác (checkout, rename, delete…) hiện bằng hộp thoại** (bạn yêu cầu): `message(…, { kind: "error" })` của plugin dialog, không lẫn vào danh sách. Ô lỗi nhỏ (có viền, nút **×**) phía trên danh sách giờ chỉ dùng khi **không tải được** chính danh sách branch.

- **Sắp xếp theo tên A→Z / Z→A** (bạn yêu cầu): nút trên header của panel; lựa chọn được nhớ (`localStorage`, cho mọi repo). Sắp bên trong từng nhóm (Local, mỗi remote); thứ tự các remote giữ nguyên. Dùng `Intl.Collator` như bảng file ở bước 6: "feature-2" trước "feature-10", chữ hoa/thường xếp chung.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Branch + trạng thái theo dõi | `git for-each-ref --format=%(refname)%1f%(HEAD)%1f%(upstream:short)%1f%(upstream:track,nobracket) refs/heads refs/remotes` |
| Danh sách remote | `git remote` |

### Ghi chú
- **`%(upstream:track,nobracket)`**: git tự tính ahead/behind cho **mọi** branch trong một lệnh: `ahead 1, behind 2`, `ahead 3`, `behind 4`, `gone`, hoặc rỗng (khớp / không có upstream). `parse_track` đọc chữ này (có test).
- **Tách tên remote**: `upstream/feature/x` → nhóm `upstream`, branch `feature/x`. Dùng lại `split_remote_name` của bước 19b (tên remote **dài nhất** khớp, vì tên remote cũng có thể chứa `/`). Remote branch của một remote đã bị xóa (còn sót ref) được xếp theo phần trước dấu `/` đầu tiên.
- **`origin/HEAD`** không phải branch, chỉ cho biết branch mặc định của server, nên bị bỏ (dùng lại `ref_info` của bước 16c).

## Sidebar và RepoView nói chuyện với nhau thế nào
Danh sách branch nằm trong sidebar (`AppSidebar`), **không** nằm trong `RepoView`. Hai bên cần:
1. RepoView vừa có status mới (commit, pull, checkout…) → danh sách branch phải tải lại.
2. Thao tác trong danh sách branch (checkout…) trả về status mới → RepoView phải hiển thị nó. RepoView vẫn là nơi **duy nhất** giữ status.
3. Khi RepoView đang chạy một lệnh git, danh sách branch không được chạy thêm lệnh khác.

Giải pháp: một Pinia store nhỏ **`repoActivity`** làm "bảng tin chung":

| Trường | Ai ghi | Ai đọc |
|---|---|---|
| `statusVersion` | RepoView (`statusChanged()` mỗi khi có status mới) | BranchPanel `watch` → tải lại |
| `busy` | RepoView (bản sao của `working`) | BranchPanel: bỏ qua double-click / menu khi đang bận |
| `incomingStatus` | BranchPanel (`deliverStatus()` sau checkout…) | RepoView `watch` → `applyStatus`, rồi xóa |

## Luồng dữ liệu

```
Chọn repo → AppSidebar → <BranchPanel :key="repo.id"> → listBranches → invoke("list_branches")
  └─ Rust git::branches::list_branches → for-each-ref + git remote → BranchList { local, remotes[] }
Double-click "feature" → checkOutItem → checkoutBranch → invoke("checkout_branch") (bước 19a)
  └─ callbacks.onStatus → activity.deliverStatus(status)
       └─ RepoView watch(incomingStatus) → applyStatus → statusVersion++ → activity.statusChanged()
            └─ BranchPanel watch(statusVersion) → listBranches → ✓ chuyển sang "feature"
Chuột phải → showBranchItemMenu → askMerge / confirmRebase / askRename / confirmDeleteLocal … (đã có)
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/branches.rs` (mới) | `list_branches`, `parse_track` + 3 test |
| `src-tauri/src/git/branch.rs` | `split_remote_name` thành `pub(crate)` |
| `src-tauri/src/models.rs` | `BranchList`, `LocalBranch`, `RemoteGroup`, `RemoteBranch` |
| `src-tauri/src/commands/branch.rs`, `lib.rs` | `list_branches` |
| `src/types/branches.ts` (mới), `src/api/branch.ts` | kiểu + `listBranches` |
| `src/stores/repoActivity.ts` (mới) | "bảng tin chung" giữa sidebar và RepoView |
| `src/components/BranchPanel.vue` (mới) | cây branch: nhóm, gập/mở, double-click, chuột phải |
| `src/components/branchMenus.ts` | `showBranchItemMenu`, `checkOutItem` |
| `src/components/AppSidebar.vue` | chia trên/dưới với thanh kéo (`useSplitRatio`, bước 18) |
| `src/views/RepoView.vue` | báo `statusChanged` / `busy`, nhận `incomingStatus` |

## Khái niệm mới

### Rust
- **`Some(x).filter(|u| !u.is_empty())`**: biến chuỗi rỗng thành `None` trong một dòng.
- **`or_else(...).unwrap_or_else(...)`**: thử cách thứ nhất (tên remote đã biết), không được thì cách thứ hai (phần trước `/`), cuối cùng mới dùng giá trị dự phòng.
- **`iter_mut().find(...)`** rồi sửa phần tử tìm được, hoặc `push` nhóm mới nếu không có: kiểu "tìm hoặc thêm" quen thuộc, giống `computeIfAbsent` của `Map` trong Java (ở đây là một `Vec`, để giữ thứ tự).

### Vue
- **Store làm "bảng tin chung" giữa hai component không lồng nhau**: props/emit chỉ đi giữa cha và con; sidebar và RepoView là "anh em" trong `App.vue`, nên dùng một store nhỏ. Mỗi trường chỉ có **một bên ghi**, để luồng dữ liệu vẫn dễ theo dõi.
- **`:key` để làm mới component khi đổi repo** (`<BranchPanel :key="repo.id">`): Vue tạo component mới, không còn dữ liệu của repo cũ (giống RepoView từ bước 6).
- **`useSplitRatio` dùng lại lần thứ ba** (tab Changes, History, giờ là sidebar): lợi ích của composable ở bước 18.
- **Cây gập/mở bằng `Set` các khóa** (`"local"`, `"remotes"`, `"remote:origin"`), lưu vào `localStorage` dạng mảng JSON.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Chọn một repo | Sidebar chia đôi: Repositories ở trên, Branches ở dưới |
| 2 | Kéo thanh ngang giữa hai phần, đóng/mở lại app | Tỉ lệ được nhớ |
| 3 | Repo `branchy` | Local Branches: feature, hotfix… và ✓ main (đậm); Remotes: "No remotes." |
| 4 | Repo có remote (`moon-git`) | Remotes ▸ origin với các branch của nó |
| 5 | Thêm remote thứ hai (terminal: `git remote add upstream <url>` rồi `git fetch upstream`) → ↻ | Nhóm **upstream** riêng |
| 6 | Commit một lần trên branch có upstream | Branch đó hiện **↑1** (sau khi danh sách tự tải lại) |
| 7 | Double-click một branch local khác | Header `⎇ …` đổi; ✓ chuyển sang branch đó |
| 8 | Double-click một remote branch chưa có bản local | Tạo branch local theo dõi nó, ✓ ở branch mới |
| 9 | Chuột phải một branch → **Merge … into Current Branch…** | Hộp thoại Merge của bước 21 |
| 10 | Chuột phải branch hiện tại | Chỉ có New Branch from Here… và Rename… |
| 11 | Gập "Local Branches", đổi repo | Vẫn gập |
| 12 | Chuột phải remote branch → Delete … on the Server… | Hỏi trước (như bước 19b) |
