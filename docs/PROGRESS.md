# MoonGit — Kế hoạch và tiến độ

Cập nhật lần cuối: 2026-10-21

**Đang ở:** Bước 7 (stage / unstage) đang làm.

Ký hiệu: ✅ xong · 🟡 đang làm / chờ xác nhận · ⬜ chưa làm

Giải thích chi tiết từng bước: [docs/learning/](learning/README.md)

---

## Giai đoạn 1 — dùng được hằng ngày

| # | Bước | Trạng thái | Ghi chú |
|---|---|---|---|
| 0 | Chuẩn bị môi trường | ✅ | Rust 1.99.1, Node 24, pnpm 10, git 2.33, CLT (SDK 13.1). Còn cần `sudo xcode-select --switch /Library/Developer/CommandLineTools` |
| 1 | Khung Tauri v2 + Vue 3 + TS, layout sidebar \| nội dung, command `greet` | ✅ | Đã xác nhận giao diện (2026-10-21) |
| 2 | Lớp chạy git + kiểu lỗi chung | ✅ | `git/runner.rs`, `error.rs`, command `check_repo`. 7 test pass, clippy sạch. Đã xác nhận 4 trường hợp trên giao diện (2026-10-21) |
| 3 | Danh sách repo + thêm repo local | ✅ | `storage/repos.rs` (`repos.json`, ghi qua file tạm + rename), plugin dialog, Pinia store. 13 test pass, clippy sạch. Đã xác nhận trên giao diện (2026-10-21) |
| 4 | Nhóm / nhóm con repo | ✅ | `storage/repo_list.rs`, `repos.json` v2, menu chuột phải native, `NameDialog`. 23 test pass. Đã xác nhận (2026-10-21), thêm icon folder SVG. Chưa có: kéo-thả, di chuyển nhóm |
| 5 | Clone có tiến trình + hủy | ✅ | `git/operations.rs` (thread worker + Tauri event + hủy cả process group), `git/clone.rs`, `CloneDialog`. 36 test pass. Đã xác nhận (2026-10-21). Fetch/pull/push sẽ dùng lại `operations.rs` |
| 6 | Danh sách file thay đổi | ✅ | `git/status.rs` (porcelain v2 -z, `--no-optional-locks`), bảng `FileTable.vue` (Name/State/Location, sắp xếp theo cột, giống SmartGit), tự làm mới khi focus. 45 test pass. Đã xác nhận (2026-10-21) |
| 7 | Stage / unstage | ⬜ | |
| 8 | Diff side-by-side | ⬜ | Monaco diff editor (sẽ trình bày các lựa chọn trước khi cài) |
| 9 | Commit + amend | ⬜ | |
| 10 | Branch hiện tại + ahead/behind + fetch | ⬜ | |
| 11 | Pull, push | ⬜ | |
| 12 | Thiết kế cách cấp credential | ⬜ | Chỉ đề xuất `GIT_ASKPASS` hay credential helper, chờ bạn chọn rồi mới code |
| 13 | Account theo server + keyring | ⬜ | Token chỉ lưu trong keyring, gắn account cho từng repo |
| 14 | `user.name` / `user.email` riêng cho từng repo | ⬜ | `git config --local` |
| 15 | Chọn SSH key theo server/repo | ⬜ | `GIT_SSH_COMMAND` |

## Giai đoạn 2 — lịch sử và branch (sẽ chia nhỏ khi tới nơi)

| # | Tính năng | Trạng thái |
|---|---|---|
| 16 | Graph lịch sử: chọn branch/tag hiển thị, lane layout bằng TS có test, danh sách ảo hóa + tải dần | ⬜ |
| 17 | Filter theo hash / message, chế độ highlight (mặc định) và ẩn | ⬜ |
| 18 | Bấm vào commit: thông tin, danh sách file, diff | ⬜ |
| 19 | Branch: tạo, checkout, đổi tên, xóa local/remote, tracking | ⬜ |

## Giai đoạn 3 — thao tác nâng cao (sẽ chia nhỏ khi tới nơi)

| # | Tính năng | Trạng thái |
|---|---|---|
| 20 | Stash: tạo (message, tùy chọn gồm untracked), danh sách, apply/pop/drop | ⬜ |
| 21 | Merge: `--no-ff` hoặc `--squash` / `--no-commit` | ⬜ |
| 22 | Rebase (không interactive) | ⬜ |
| 23 | Cherry-pick: giữ commit hoặc `-n` | ⬜ |
| 24 | Xử lý conflict: phát hiện trạng thái, ours/theirs, công cụ ngoài, Continue/Abort | ⬜ |

---

## Quyết định đã chốt
- Tên app **MoonGit**, identifier `com.github.halab4dev.moongit`.
- Package manager: **pnpm**.
- Bỏ plugin `opener` của template (chưa cần).
- Pinia v4, dùng kiểu "setup store" (`ref`/`computed`/hàm).
- Danh sách repo lưu ở `<app data>/repos.json` (`{ version, repos }`); file hỏng được đổi tên thành `repos.json.broken`.
- Id repo sinh từ thời gian (hex), không dùng crate `uuid`.
- Bỏ repo khỏi danh sách không đụng tới thư mục trên ổ đĩa.
- Nhóm lưu phẳng (`parentId`); xóa nhóm → nội dung lên một cấp. Tên nhóm cùng cha không trùng (không phân biệt hoa thường).
- Thao tác trên sidebar dùng menu native của Tauri; nút × ở bước 3 được thay bằng "Remove from List".
- Thao tác dài: thread worker (`spawn_blocking`) + đọc stderr, thay vì `tokio::process` (dễ đọc hơn, không thêm thư viện). Mỗi thao tác có `opId`; tiến trình qua event `git-progress`.
- Hủy = SIGTERM cho cả process group (crate `rustix`, chỉ unix). Windows tạm chỉ dừng `git`.
- Clone lỗi/hủy → xóa thư mục đích chỉ khi nó không tồn tại trước khi clone.
- Command thao tác trên repo nhận `repo_id` (Rust tự tra đường dẫn), chạy nền qua `run_blocking`.
- Status làm mới khi cửa sổ được focus + nút Refresh (chưa dùng file watcher).
- Bỏ ví dụ `greet` ở bước 2, thay bằng form thử `check_repo` (form này sẽ được thay bằng "Add repository" ở bước 3).
- `AppError` là struct `{ kind, message }`; git chạy với `GIT_TERMINAL_PROMPT=0` và `LC_ALL=C`.
- Test dùng crate `tempfile` (chỉ dev-dependency).

## Việc còn treo
- [ ] Bạn chạy `sudo xcode-select --switch /Library/Developer/CommandLineTools`.
- [x] Commit đầu tiên `dd92830` (bước 1–2).
- [x] Bước 3 đã xác nhận và commit.
- [x] Bước 4, 5 đã xác nhận.
- [x] Bước 6 đã xác nhận và commit.
