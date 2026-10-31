# Bước 15b — Login (SSH & HTTPS) cho cả nhóm

*Tiếp 15a. Cùng cách với danh tính nhóm ở bước 14: ghi vào `.git/config` của các repo trong nhóm, hai kiểu ghi.*

## Trạng thái
- `cargo test`: 239/239 (thêm 1 test). `cargo clippy`: sạch. `pnpm test`: 38/38. `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit (2026-11-01).

## Giao diện
**Chuột phải một nhóm → Login (SSH & HTTPS)…**:

```
Login · group "Work"
Written into the 4 repositories of this group and its subgroups …
SSH
◉ Use this computer's SSH settings
○ Use this key   [ ~/.ssh/id_work ] [Choose…]
HTTPS
◉ MoonGit's accounts
○ Use this computer's credentials
Write into
◉ Only repositories without their own setting
   SSH and HTTPS on their own: a repository with its own key keeps it, and
   still gets the HTTPS choice if it has none.
○ Write for all repositories
   Also replaces what repositories already have ("this computer's" removes it).
                                              [Cancel] [Save]
```

**SSH và HTTPS được xét riêng** cho từng repo:

| Repo đang có (phần SSH hoặc HTTPS) | Only repositories without their own setting | Write for all repositories |
|---|---|---|
| Chưa đặt | Ghi lựa chọn của nhóm | Ghi lựa chọn của nhóm |
| Đã đặt | Giữ nguyên | Thay bằng lựa chọn của nhóm |

- "Use this computer's …" nghĩa là **không đặt gì**. Với "Only…" thì phần chưa đặt vẫn chưa đặt (không đổi gì); với "Write for all" thì **xóa** thiết lập riêng của mọi repo (quay về như máy).
- Hộp thoại nhớ lựa chọn lần trước của nhóm (lưu trong `repos.json`, trường `access` của nhóm).
- Save xong: thông báo "Changed in 3 of 4 repositories…", và danh sách repo không ghi được (nếu có).
- Key được kiểm tra **một lần** trước khi ghi (tồn tại, không phải `.pub`, `~/` được mở rộng); key sai thì không repo nào bị đổi.

## Luồng dữ liệu

```
Nhóm → Login (SSH & HTTPS)… → <GroupAccessDialog>
  Save → store.setGroupAccess(groupId, key | null, systemCredentials, all)
       → invoke("apply_group_access")
            checked_key(key)                       (một lần)
            repos_in_group(groupId)                (gồm nhóm con)
            mỗi repo: git::access::apply_group_access(repo, key, system, all)
                 SSH:   chưa đặt hoặc all → core.sshCommand = … / --unset-all
                 HTTPS: chưa đặt hoặc all → moongit.credentials = system / --unset-all
            set_group_access → repos.json
       ← GroupApplyResult { list, total, changed, failed } → thông báo
```

## File

| File | Việc |
|---|---|
| `src-tauri/src/git/access.rs` | `apply_group_access`, `checked_key` + 1 test |
| `src-tauri/src/commands/access.rs`, `lib.rs` | `apply_group_access` |
| `src-tauri/src/models.rs` | `GroupAccess`, `RepoGroup.access`; `GroupIdentityResult` đổi tên thành `GroupApplyResult` (dùng chung cho 14b và 15b) |
| `src-tauri/src/storage/repo_list.rs`, `repos.rs` | `set_group_access` |
| `src/components/GroupAccessDialog.vue` (mới) | Hộp thoại |
| `src/stores/repoTree.ts` | `countReposInGroup` (dùng chung cho hai hộp thoại nhóm) |
| `src/api/access.ts`, `src/api/identity.ts`, `src/stores/repos.ts`, `src/stores/dialogs.ts`, `src/types/repo.ts`, `src/App.vue`, `src/components/sidebarMenus.ts` | Hàm gọi, kiểu, mở từ menu chuột phải của nhóm |

## Khái niệm
- **Đổi tên kiểu khi nó được dùng chung** (`GroupIdentityResult` → `GroupApplyResult`): tên phải mô tả đúng mọi chỗ dùng. Rust và TS đều báo lỗi ở mọi chỗ còn tên cũ, nên đổi tên an toàn.
- **Hàm thuần dùng chung** (`countReposInGroup`): tách khỏi component để hai hộp thoại cùng dùng, không chép code.

## Cách thử

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Chuột phải nhóm → Login (SSH & HTTPS)… | SSH / HTTPS / Write into; nhớ lựa chọn lần trước |
| 2 | Use this key + Only… → Save | Repo chưa có key: `git config --local core.sshCommand` ra lệnh với key đó; repo đã có key riêng: giữ nguyên |
| 3 | Mở hộp thoại Login của một repo trong nhóm | Thấy đúng key đã ghi |
| 4 | Use this computer's SSH settings + Write for all → Save | Mọi repo trong nhóm không còn `core.sshCommand` |
| 5 | HTTPS → Use this computer's credentials + Only… | Repo chưa đặt: `moongit.credentials = system` |
| 6 | Chọn file `.pub` | Lỗi trong hộp thoại, không repo nào bị đổi |
