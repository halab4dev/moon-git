# MoonGit — Kế hoạch và tiến độ

Cập nhật lần cuối: 2026-10-21

**Đang ở:** Bước 2 xong và đã xác nhận → chờ đồng ý để bắt đầu Bước 3.

Ký hiệu: ✅ xong · 🟡 đang làm / chờ xác nhận · ⬜ chưa làm

Giải thích chi tiết từng bước: [docs/learning/](learning/README.md)

---

## Giai đoạn 1 — dùng được hằng ngày

| # | Bước | Trạng thái | Ghi chú |
|---|---|---|---|
| 0 | Chuẩn bị môi trường | ✅ | Rust 1.99.1, Node 24, pnpm 10, git 2.33, CLT (SDK 13.1). Còn cần `sudo xcode-select --switch /Library/Developer/CommandLineTools` |
| 1 | Khung Tauri v2 + Vue 3 + TS, layout sidebar \| nội dung, command `greet` | ✅ | Đã xác nhận giao diện (2026-10-21) |
| 2 | Lớp chạy git + kiểu lỗi chung | ✅ | `git/runner.rs`, `error.rs`, command `check_repo`. 7 test pass, clippy sạch. Đã xác nhận 4 trường hợp trên giao diện (2026-10-21) |
| 3 | Danh sách repo + thêm repo local | ⬜ | `storage/` lưu JSON trong app data, chọn thư mục, thêm Pinia |
| 4 | Nhóm / nhóm con repo | ⬜ | Cây sidebar, tạo/đổi tên/xóa nhóm, di chuyển repo (kéo-thả làm sau) |
| 5 | Clone có tiến trình + hủy | ⬜ | `tokio::process`, Tauri event, nút Cancel. Fetch/pull/push dùng lại phần này |
| 6 | Danh sách file thay đổi | ⬜ | Parser `status --porcelain=v2 -z` có test; staged / unstaged / untracked |
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
- Pinia thêm ở bước 3.
- Bỏ ví dụ `greet` ở bước 2, thay bằng form thử `check_repo` (form này sẽ được thay bằng "Add repository" ở bước 3).
- `AppError` là struct `{ kind, message }`; git chạy với `GIT_TERMINAL_PROMPT=0` và `LC_ALL=C`.
- Test dùng crate `tempfile` (chỉ dev-dependency).

## Việc còn treo
- [ ] Bạn chạy `sudo xcode-select --switch /Library/Developer/CommandLineTools`.
- [ ] Commit đầu tiên (repo đã `git init`, chưa commit). Commit khi bạn yêu cầu.
