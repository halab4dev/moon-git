# Bước 35b — Chọn nhiều repo: kéo cùng lúc, thao tác chung (gồm cả 35c)

*Phần hai của yêu cầu 2026-11-01 (35a: kéo nhóm). Bạn muốn làm luôn Commit Identity / Login cho các repo đang chọn (dự định là 35c), nên gộp vào đây.*

## Trạng thái
- `cargo test`: 241/241, `cargo clippy`: sạch (thêm 2 command, không đổi logic). `pnpm test`: 42/42 (thêm 4 test). `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit.

## Cách chọn (giống Finder)

| Thao tác | Kết quả |
|---|---|
| Click | Mở repo đó, chỉ chọn mình nó (như trước) |
| **⌘-click** | Thêm / bỏ một repo khỏi vùng chọn; repo đang mở vẫn mở |
| **⇧-click** | Chọn mọi repo **đang hiện** từ repo click lần trước tới repo này (bỏ qua bên trong nhóm đang gập) |

- Repo đang mở: nền xanh đậm như trước. Các repo khác được chọn: **nền xanh nhạt**.
- Click thường một repo bất kỳ: về lại chỉ một repo được chọn.

## Làm gì với nhiều repo
- **Kéo** một repo đang được chọn: **cả nhóm repo đang chọn** đi cùng (vào nhóm, ra ngoài cùng…, như bước 34). Kéo một repo không được chọn: chỉ repo đó.
- **Chuột phải** một repo đang được chọn (khi chọn từ 2 repo):

```
3 Repositories
───────────────
Move To            ▸  Top Level / Work / Work / Team …
───────────────
Commit Identity…
Login (SSH & HTTPS)…
───────────────
Remove 3 from List…
```

- **Remove N from List…** hỏi xác nhận trước (liệt kê tối đa 10 tên). Chỉ bỏ khỏi danh sách, **không xóa thư mục**. (Bỏ một repo vẫn không hỏi, như trước, vì thêm lại dễ.)
- Chuột phải một repo **không** được chọn: menu của riêng repo đó như cũ.
- **Commit Identity…** / **Login (SSH & HTTPS)…**: đúng hộp thoại của nhóm (bước 14b / 15b), tiêu đề "· 3 repositories", ghi vào `.git/config` của **các repo đang chọn**, với hai kiểu: *Only repositories without their own setting* / *Write for all repositories*. Hộp thoại không điền sẵn lựa chọn cũ (chỉ nhóm mới nhớ), danh tính điền sẵn của máy.

## Cách làm
- Store `repos` có thêm `picked` (tập id các repo đang chọn) cạnh `selectedId` (repo đang mở), và `anchorId` (điểm bắt đầu của ⇧-click).
- ⇧-click cần **thứ tự hiển thị** của repo: `visibleRepoOrder(tree, isCollapsed)` đi cây giống hệt sidebar vẽ (nhóm trước, repo sau ở mỗi cấp; bỏ qua nhóm gập), rồi `rangeBetween(order, từ, tới)` lấy đoạn giữa. Hai hàm thuần, có test.
- Kéo dùng lại `repoDrag.ts` của 35a, vốn đã nhận một **danh sách** repo.
- Identity / Login cho nhiều repo: hai hộp thoại của nhóm nhận request `{ groupId }` **hoặc** `{ repoIds }`. Phần "đang làm cho ai" (tiêu đề, số repo, câu mô tả) gom vào composable `useApplyTarget`. Rust thêm `apply_identity_to_repos` / `apply_access_to_repos`; vòng lặp "ghi từng repo, gom lỗi" của cả nhóm lẫn danh sách repo dùng chung hàm `apply_to_repos`.

## File

| File | Việc |
|---|---|
| `src/stores/repoTree.ts` + `repoTree.test.ts` (test mới) | `visibleRepoOrder`, `rangeBetween` + 4 test |
| `src/stores/repos.ts` | `picked`, `togglePicked`, `pickRange`, `isPicked`, `clearPicked`, `pickedRepos` |
| `src/components/RepoItem.vue` | Click / ⌘-click / ⇧-click; class `picked` |
| `src/components/repoDrag.ts` | Kéo cả vùng chọn |
| `src/components/sidebarMenus.ts` | Menu cho nhiều repo: Move To, Commit Identity…, Login…, Remove N from List… |
| `src/components/applyTarget.ts` (mới) | `useApplyTarget`: nhóm hoặc các repo đang chọn |
| `src/components/GroupIdentityDialog.vue`, `GroupAccessDialog.vue` | Nhận cả `{ repoIds }` |
| `src/api/identity.ts`, `src/api/access.ts`, `src/stores/repos.ts`, `src/stores/dialogs.ts` | `setReposIdentity`, `setReposAccess`; kiểu request |
| `src-tauri/src/commands/mod.rs`, `identity.rs`, `access.rs`, `storage/repos.rs`, `lib.rs` | `apply_to_repos` (dùng chung), `apply_identity_to_repos`, `apply_access_to_repos`, `repos_by_ids` |
| `src/styles.css` | `.tree-row.picked` |

## Khái niệm
- **`event.metaKey` / `ctrlKey` / `shiftKey`**: phím đang giữ khi click (⌘ trên macOS, Ctrl trên Linux/Windows).
- **"Anchor" của vùng chọn**: điểm mốc của ⇧-click là repo click lần cuối **không** giữ ⇧, nên ⇧-click liên tiếp đổi đầu kia của đoạn chứ không dời mốc. Cách mọi danh sách trên desktop làm.

## Cách thử
Bấm ⇧⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Click repo A, ⌘-click B và C | A xanh đậm, B C xanh nhạt |
| 2 | ⌘-click B lần nữa | B bỏ chọn |
| 3 | Click A, ⇧-click D (cách vài dòng) | Mọi repo từ A tới D được chọn |
| 4 | Kéo một repo đang chọn lên một nhóm | Tất cả repo đang chọn vào nhóm đó |
| 5 | Chuột phải một repo đang chọn | "3 Repositories", Move To, Commit Identity…, Login…, Remove 3 from List… |
| 5b | Commit Identity… → nhập email → Only… → Save | "Written to … of 3 repositories"; repo có danh tính riêng giữ nguyên |
| 5c | Login… → Use this key → Write for all → Save | Cả 3 repo có `core.sshCommand` với key đó |
| 6 | Remove N from List… → Remove | Các repo biến khỏi danh sách; thư mục vẫn còn trên ổ đĩa |
| 7 | Click thường một repo | Chỉ còn repo đó được chọn |
