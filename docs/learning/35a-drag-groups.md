# Bước 35a — Kéo cả nhóm vào nhóm khác

*Bạn yêu cầu (2026-11-01): kéo cả nhóm sang nhóm khác, chọn nhiều repo để kéo hoặc làm thao tác chung. Chia: **35a** kéo nhóm, **35b** chọn nhiều repo (kéo, Move To, Remove), **35c** Commit Identity / Login cho các repo đang chọn.*

## Trạng thái
- `cargo test`: 241/241 (thêm 2 test). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK (sau khi sửa hot reload). Đã commit.

## Cách dùng
Kéo **dòng nhóm** (folder) như kéo repo ở bước 34:

| Thả lên | Kết quả |
|---|---|
| Một nhóm khác | Nhóm (cùng mọi nhóm con và repo bên trong) thành nhóm con của nhóm đó |
| Một repo | Vào cùng nhóm với repo đó |
| Khoảng trống trong danh sách | Ra cấp ngoài cùng |

- **Không thả được** (không tô màu, con trỏ "cấm"): vào **chính nó**, vào **một nhóm con của nó** (sẽ tạo vòng lặp), hoặc vào chỗ nó đang ở sẵn.
- Ở nơi đến đã có nhóm **cùng tên** (không phân biệt hoa thường): báo lỗi ở đầu sidebar, không đổi gì (như khi tạo / đổi tên nhóm).
- Giữ chuột trên một nhóm đang gập ~0,7 giây: nhóm mở ra (không áp dụng cho chính nhóm đang kéo).
- Nhóm mang theo danh tính / login đã đặt (bước 14b / 15b). Các thiết lập đó đã nằm trong `.git/config` của từng repo nên không đổi gì khi di chuyển.

## Sửa sau khi bạn thử
- Lỗi `TypeError: store.moveGroup is not a function`: **không phải lỗi của code mới**, mà của việc nạp lại nóng (hot reload) lúc dev. Store `repos` chưa có `acceptHMRUpdate`, nên sau khi sửa code, app vẫn giữ **bản store cũ** (không có `moveGroup`) cho tới khi nạp lại hẳn giao diện. Đã gặp đúng lỗi này ở bước 21 với store `dialogs`.
- Sửa: thêm `acceptHMRUpdate` cho store `repos` (như `dialogs`, `menuCommands`, `repoActivity`). Bản build không bao giờ gặp lỗi này (không có hot reload).

## Rust
`RepoList::move_group(id, parent_id)`: kiểm tra nhóm đích tồn tại, không phải chính nó hay nhóm con của nó (`is_inside`: đi ngược lên các nhóm cha), tên không trùng ở nơi đến; rồi chỉ đổi `parent_id` của nhóm. Mọi thứ bên trong đi theo, vì chúng trỏ tới nhóm, không trỏ tới vị trí.

## Frontend
`repoDrag.ts` giờ kéo được hai loại: `{ kind: "repos", repos: [...] }` (danh sách, để 35b kéo nhiều repo) và `{ kind: "group", group }`. Kiểm tra "thả được không" (`canDropInto`) chạy cả ở giao diện (để tô màu đúng) và ở Rust (để chắc chắn).

## File

| File | Việc |
|---|---|
| `src-tauri/src/storage/repo_list.rs` | `move_group`, `is_inside` + 2 test |
| `src-tauri/src/storage/repos.rs`, `commands/group.rs`, `lib.rs` | command `move_group` |
| `src/api/group.ts`, `src/stores/repos.ts` | `moveGroup` |
| `src/components/repoDrag.ts` | Kéo repo hoặc nhóm, `canDropInto`, `isDragged` |
| `src/components/GroupItem.vue`, `RepoItem.vue` | Dòng nhóm kéo được |

## Khái niệm
- **Union type có trường phân biệt** (`{ kind: "repos" } | { kind: "group" }`): kiểm tra `item.kind` xong, TS biết chính xác các trường còn lại (`item.repos` hay `item.group`). Giống `sealed interface` + `instanceof` pattern trong Java 17+.
- **Kiểm tra hai lần** (giao diện + Rust): giao diện để hiện đúng ngay khi kéo, Rust để dữ liệu không bao giờ sai, kể cả khi giao diện có lỗi.

## Cách thử
Bấm ⇧⌘R. Cần vài nhóm, có nhóm con.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Kéo nhóm A lên nhóm B | B được tô; thả → A (và mọi thứ bên trong) nằm trong B, B mở ra |
| 2 | Kéo A xuống khoảng trống | A ra ngoài cùng |
| 3 | Kéo nhóm cha lên một nhóm con của nó | Không tô màu, thả không có gì xảy ra |
| 4 | Kéo A vào một nhóm đã có nhóm con tên "A" | Lỗi "already exists" ở đầu sidebar |
| 5 | Kéo repo như trước | Vẫn chạy |
