# Bước 36 — Fetch dọn branch đã xóa trên server; xóa nhiều branch local

*Bạn yêu cầu (2026-11-01). Bạn chọn: chỉ giữ branch local, bản sao `origin/…` thì dọn.*

## Trạng thái
- `cargo test`: 243/243 (thêm 2 test). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit.

## Thay đổi

### 1. Fetch dùng `--prune`
- Fetch (và phần tải về của Pull) giờ là `git fetch --all --prune --progress`.
- Branch bị xóa trên server (vd. đồng nghiệp merge PR rồi bấm *Delete branch*): bản sao `origin/feature/x` **biến mất** sau Fetch. Nó chỉ là ảnh chụp của server, không chứa gì của bạn.
- **Branch local không bao giờ bị xóa tự động.**

### 2. Branch local "deleted on server"
Branch local đang theo dõi một branch đã bị xóa trên server: panel Branches hiện một **nhãn đỏ** bên phải: icon **đám mây bị gạch chéo** + chữ **deleted on server**, nền đỏ nhạt (đọc được cả khi dòng đang được chọn, cả ở dark mode). Trước đây chỉ ghi chữ "gone" màu cam; bạn thấy chưa đủ nổi bật nên đổi. Di chuột lên để thấy nó từng theo dõi branch nào.

### 3. Chọn nhiều branch local để xóa
- **⌘-click** / **⇧-click** trong *Local Branches* (như danh sách repo, bước 35b). Branch được chọn thêm có nền xanh nhạt.
- **Chuột phải một branch đang chọn** (từ 2 branch trở lên):

```
3 Branches
──────────
Delete 3 Branches…
```

- **Chuột phải dòng "Local Branches"**:

```
Select 4 Branches Deleted on Server
Delete Branches Deleted on Server…
```

- Xóa luôn **hỏi trước** (liệt kê tối đa 12 tên). Branch **đang checkout** không bao giờ bị xóa (bị bỏ khỏi danh sách).
- Branch có **commit không nằm ở branch nào khác** (chưa merge): không bị xóa ở lần đầu. Hộp thoại thứ hai liệt kê chúng: **Delete Anyway** hoặc **Keep Them** (như xóa từng branch ở bước 19b).
- Branch không xóa được vì lý do khác: thông báo cuối cùng liệt kê, các branch khác vẫn được xóa.

## Luồng dữ liệu

```
Fetch → git fetch --all --prune --progress
      → panel Branches tải lại: branch local có upstream "[gone]" → "deleted on server"
Chuột phải "Local Branches" → Delete Branches Deleted on Server… → confirmDeleteLocals
  ask(...) → deleteBranches(repoId, names, false) → invoke("delete_branches")
     → git::branch::delete_locals: từng branch `git branch --delete` (lỗi riêng từng cái)
     ← { deleted, notMerged, failed, status }
  notMerged? → ask(...) → deleteBranches(…, force = true)   (`git branch --delete --force`)
  failed? → message(...)
```

## File

| File | Việc |
|---|---|
| `src-tauri/src/git/fetch.rs` | `--prune` + 1 test (origin/x bị dọn, branch local còn, upstream "[gone]") |
| `src-tauri/src/git/branch.rs` | `delete_locals` + 1 test |
| `src-tauri/src/models.rs` | `DeletedBranches`, `DeleteBranchesResult` |
| `src-tauri/src/commands/branch.rs`, `lib.rs` | `delete_branches` |
| `src/api/branch.ts` | `deleteBranches` |
| `src/components/branchMenus.ts` | `confirmDeleteLocals` (hỏi, xóa, hỏi lại cho branch chưa merge) |
| `src/components/BranchPanel.vue` | ⌘ / ⇧-click, menu nhiều branch, menu "Local Branches", nhãn đỏ "deleted on server" |
| `src/components/AppIcon.vue` | icon `serverGone` (đám mây gạch chéo, đỏ) |

## Khái niệm
- **`#[serde(flatten)]`**: gộp các trường của một struct con vào JSON của struct cha. `DeleteBranchesResult { branches: DeletedBranches, status }` ra JSON `{ deleted, notMerged, failed, status }`, không lồng thêm một tầng.
- **Dùng lại hàm thuần** `rangeBetween` (bước 35b) cho ⇧-click trên danh sách branch: hàm chỉ làm việc với mảng chuỗi, không biết gì về repo hay branch.

## Cách thử
App dev tự build lại phần Rust; bấm ⇧⌘R nếu giao diện chưa đổi.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Tạo branch, push lên server, rồi xóa trên GitHub (hoặc `git push origin --delete <tên>` trong terminal) | |
| 2 | Fetch | Remotes ▸ origin không còn branch đó; branch local ghi "deleted on server" |
| 3 | Chuột phải "Local Branches" | "Select N Branches Deleted on Server", "Delete Branches Deleted on Server…" |
| 4 | Click branch A, ⌘-click B, ⇧-click… | Chọn được nhiều branch |
| 5 | Chuột phải một branch đang chọn → Delete N Branches… → Delete | Các branch biến mất |
| 6 | Trong số đó có branch chưa merge | Hỏi lần hai; Keep Them → branch đó còn lại |
| 7 | Chọn cả branch đang checkout | Menu ghi "(not the current one)", branch đó không bị xóa |
