# Bước 14 — `user.name` / `user.email` riêng cho từng repo, và cho từng nhóm

*Lộ trình CLAUDE.md mục 8: "`user.name` / `user.email` riêng cho từng repo". Tạm hoãn từ 2026-10-21, làm 2026-10-29.*

## Trạng thái
- `cargo test`: 229/229 (thêm 6 test: 3 cho repo, 3 cho nhóm). `cargo clippy`: sạch. `pnpm test`: 38/38. `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit (2026-11-01).

## Vì sao cần
Mỗi commit ghi tên và email người làm (`Author: halab <halab4it@gmail.com>`). Git lấy hai giá trị này từ:
1. **config của repo** (`.git/config`): nếu có thì ưu tiên;
2. **config chung của máy** (`~/.gitconfig`, đặt bằng `git config --global`);
3. config của cả hệ thống (hiếm khi dùng).

Ví dụ: repo công ty dùng email công ty, các repo khác dùng email cá nhân.

## Giao diện
Mở bằng **Repository → Commit Identity…**, hoặc **chuột phải một repo** trong sidebar → **Commit Identity…**, hoặc link **Change…** trong hộp thoại Commit:

```
Commit Identity · moon-git
The name and email written into every commit made in this repository.
◉ Same as the rest of this computer
   halab <halab4it@gmail.com>
○ Only for this repository (e.g. a work email)
     Name   [ halab              ]
     Email  [ halab4it@gmail.com ]
                                   [Cancel] [Save]
```

- **Same as the rest of this computer**: xóa thiết lập riêng của repo (nếu có), dùng `~/.gitconfig`. Nếu máy chưa có gì: dòng cảnh báo "not set on this computer: commits will fail".
- **Only for this repository**: ghi vào `.git/config` của repo. Các ô điền sẵn giá trị đang dùng, chỉ cần sửa phần khác.
- Kiểm tra trước khi ghi: không rỗng, một dòng, không có `<` `>` (git ghi `Tên <email>` vào commit), email phải có `@`.
- **Hộp thoại Commit** có thêm một dòng: *Committing as halab <halab4it@gmail.com>*, kèm *(this repository)* nếu là thiết lập riêng, và link **Change…**. Chưa có danh tính: "No name / email for commits yet."
- Lỗi "git does not know your name and email" khi commit giờ chỉ tới **Repository → Commit Identity…**.
- Chỉ thay đổi `.git/config` của repo đó. Không bao giờ sửa `~/.gitconfig`.

## Danh tính theo nhóm (bạn yêu cầu thêm, "14b")
**Chuột phải một nhóm → Commit Identity…**. Danh tính của nhóm được **ghi vào `.git/config` của các repo trong nhóm** (kể cả nhóm con), nên **git trong terminal cũng dùng**.

```
Commit Identity · group "Work"
Written into the 4 repositories of this group and its subgroups …
Name   [ Halab Work         ]
Email  [ halab@work.example ]
◉ Only repositories without their own setting
   Repositories that already have a name or email keep theirs.
○ Write for all repositories
   Also replaces the name and email some repositories already have.
                                         [Cancel] [Save]
```

| Repo đang có | Only repositories without their own setting | Write for all repositories |
|---|---|---|
| Chưa đặt gì | Ghi | Ghi |
| Đúng giá trị này | Không đổi | Không đổi |
| Giá trị khác (hoặc chỉ đặt tên / chỉ đặt email) | **Giữ nguyên** | **Ghi đè** |

- Cả hai lựa chọn đều nhập Name / Email. Mặc định là "Only repositories without their own setting" (an toàn hơn).
- Name / Email điền sẵn: giá trị đã đặt cho nhóm lần trước (lưu trong `repos.json`), nếu chưa có thì của máy.
- Save xong: một thông báo "Written to 3 of 4 repositories. 1 kept its own…". Repo không ghi được (vd. thư mục đã bị xóa) được liệt kê, không làm hỏng các repo khác.
- Repo được thêm / kéo vào nhóm **sau đó** không tự nhận danh tính: mở lại hộp thoại của nhóm và Save.
- Nhóm chưa có repo nào: nút Save mờ.

*(Hai bản trước của phần này đã bỏ: (1) thêm `-c user.name=…` vào lệnh commit lúc chạy, không ghi vào repo; (2) liệt kê từng repo khác giá trị để chọn Overwrite / Keep. Bạn muốn đơn giản: hai lựa chọn cho cả nhóm.)*

## Lệnh git

| Việc | Lệnh |
|---|---|
| Đọc của repo | `git config --local --default "" --get user.name` |
| Đọc của máy | `git config --global …`, không có thì `--system …` |
| Ghi | `git config --local -- user.name <tên>` (rồi `user.email`) |
| Xóa | `git config --local --unset-all user.name` (mã thoát 5 = vốn chưa có: bỏ qua) |

- `--default ""`: khóa chưa có thì trả về chuỗi rỗng, thay vì báo lỗi (mã thoát 1).
- `--` trước tên khóa: giá trị bắt đầu bằng `-` vẫn là giá trị, không bị hiểu là option (có test).

## Luồng dữ liệu

```
Repository → Commit Identity… → dialogs.askIdentity({ repoId, repoName })  → <IdentityDialog>
  mở → getIdentity → invoke("get_identity") → git::identity::read → Identity { name, email, inheritedName, inheritedEmail }
  Save → setIdentity(repoId, { name, email } | null) → invoke("set_identity") → git::identity::set
           ├─ riêng: git config --local -- user.name … / user.email …
           └─ chung: git config --local --unset-all user.name / user.email
Commit… → <CommitDialog> → getIdentity → "Committing as …" · Change… → (hộp thoại trên) → saved() → đọc lại
```

## File

| File | Việc |
|---|---|
| `src-tauri/src/git/identity.rs` (mới) | `read`, `set`, `read_computer`, `clean_identity`, `compare`, `apply_group_identity(repo, name, email, all)` + 5 test |
| `src-tauri/src/storage/repo_list.rs`, `repos.rs` | `set_group_identity`, `repos_in_group` (gồm nhóm con) + 1 test |
| `src-tauri/src/models.rs` | `Identity`, `GroupIdentity`, `RepoGroup.identity`, `GroupIdentityResult` |
| `src-tauri/src/commands/identity.rs` (mới), `lib.rs` | `get_identity`, `set_identity`, `get_computer_identity`, `apply_group_identity` |
| `src-tauri/src/git/commit.rs` | Câu báo lỗi chỉ tới menu mới |
| `src/types/identity.ts`, `src/api/identity.ts` (mới) | Kiểu + hàm gọi |
| `src/components/IdentityDialog.vue` (mới) | Hộp thoại của repo |
| `src/components/GroupIdentityDialog.vue` (mới) | Hộp thoại của nhóm: Name / Email + hai lựa chọn ghi |
| `src/components/CommitDialog.vue`, `src/views/RepoView.vue` | Dòng "Committing as …" + Change… |
| `src/stores/dialogs.ts`, `src/App.vue`, `src/appMenu.ts`, `src/stores/menuCommands.ts`, `src/components/sidebarMenus.ts` | Mở hộp thoại từ menu bar và menu chuột phải |

## Khái niệm mới

### Rust
- **Closure trả `Result` lồng nhau** (`inherited` gọi `get`): closure dùng được closure khác khai báo trước nó, giống lambda gọi lambda.
- **`match` với guard trên `Err`** (`Err(err) if err.message.contains("exit status: 5") => Ok(())`): biến một lỗi cụ thể thành thành công.
- **Viết hoa chữ đầu** (`first.to_uppercase().chain(chars).collect()`): `to_uppercase()` trả về iterator vì một ký tự viết hoa có thể thành nhiều ký tự (vd. `ß` → `SS`).

- **Enum ba trạng thái** (`LocalIdentityState::Unset / Same / Different`) + `match` trả `bool`: quyết định "có ghi không" gọn trong một chỗ.
- **Lỗi của từng phần tử không dừng cả vòng lặp**: mỗi repo `match` riêng, lỗi được gom vào `failed` thay vì `?` (dấu `?` sẽ thoát ngay ở repo hỏng đầu tiên).
- **Struct update `Identity { name: None, email: None, ..identity }`**: lấy mọi trường khác từ `identity`.

### Vue
- **Callback trong request** (`saved?: () => void`): hộp thoại Commit nhờ hộp thoại Identity "báo lại khi lưu xong" để tự đọc lại danh tính.
- **`||=`** (`name.value ||= computer.inheritedName ?? ""`): chỉ gán khi ô đang trống, để không ghi đè thứ đã có.

## Cách thử
App dev bạn đang chạy tự build lại phần Rust. Bấm ⇧⌘R nếu giao diện chưa đổi.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Chọn `moon-git` → Repository → Commit Identity… | "Same as the rest of this computer: halab <…>" được chọn |
| 2 | Chọn "Only for this repository", sửa email → Save | Đóng; terminal: `git config --local user.email` ra email mới |
| 3 | Commit… | "Committing as … (this repository)" |
| 4 | Commit một thay đổi → History | Author là tên/email mới |
| 5 | Mở lại, chọn "Same as the rest of this computer" → Save | `git config --local user.email` không còn gì; Commit… hiện lại email chung |
| 6 | Email không có `@`, hoặc tên chứa `<` | Báo lỗi trong hộp thoại, không lưu |
| 7 | Chuột phải repo trong sidebar | Có mục Commit Identity… |
| 8 | Chuột phải một nhóm → Commit Identity… | Name / Email + hai lựa chọn; "Only repositories without…" được chọn |
| 9 | Một repo trong nhóm đã có danh tính riêng → nhập email khác, giữ "Only…" → Save | "Written to N-1 of N… 1 kept its own"; repo đó không đổi |
| 10 | Mở lại → "Write for all repositories" → Save | Mọi repo trong nhóm có email mới (`git config --local user.email`) |
| 11 | Nhóm có nhóm con | Repo trong nhóm con cũng được ghi |
| 12 | Commit trong terminal ở một repo của nhóm | Author là danh tính của nhóm |
