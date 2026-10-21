# Bước 10 — Branch hiện tại, ahead/behind, fetch

## Trạng thái
- `cargo test`: 77/77 pass (thêm 6 test: đọc header branch, repo chưa commit / detached / upstream đã bị xóa, header ab sai định dạng, fetch cập nhật ahead/behind mà không đổi file, upstream "gone" sau khi branch bị xóa trên server, repo không có remote, remote không truy cập được). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21). Đã commit.

## Mục tiêu
- Dưới tên repo hiện một dòng: **`⎇ main → origin/main ↑2 ↓1`**.
  - ↑ = số commit có ở máy bạn mà server chưa có (cần push).
  - ↓ = số commit có trên server mà máy bạn chưa có (cần pull).
  - Các trường hợp khác: `up to date`, `(no upstream)`, `origin/x (gone)` màu đỏ, `HEAD detached at abc1234` màu vàng, `(no commits yet)`.
- Nút **Fetch** trên thanh công cụ. Khi đang chạy, nút được thay bằng dòng tiến trình và nút **Cancel**. Fetch xong thì ahead/behind tự cập nhật.

## Lệnh git

| Việc | Lệnh | Ghi chú |
|---|---|---|
| Branch + ahead/behind | thêm `--branch` vào `git status --porcelain=v2 -z` đã có | không tốn thêm lệnh git nào; tự cập nhật mỗi lần làm mới |
| Fetch | `git fetch --all --progress` | `--all`: mọi remote; `--progress`: git chỉ in tiến trình ra terminal nếu không được yêu cầu rõ |

### Header `--branch`
Mỗi header là một bản ghi bắt đầu bằng `# `, nằm trước danh sách file:

```
# branch.oid 1234abcd…        ← commit hiện tại, hoặc "(initial)" khi chưa có commit
# branch.head main            ← tên branch, hoặc "(detached)"
# branch.upstream origin/main ← chỉ có khi branch theo dõi một upstream
# branch.ab +2 -5             ← ahead 2, behind 5; chỉ có khi upstream còn tồn tại
```

**"gone"**: nếu có `branch.upstream` mà **không** có `branch.ab`, nghĩa là branch đã cấu hình upstream nhưng upstream đó không còn (thường do branch bị xóa trên server). Parser tính `upstream_gone` sau khi đọc hết các header, nên không phụ thuộc vào thứ tự git in ra.

### ahead/behind tính theo **lần fetch gần nhất**
git so sánh `main` với `origin/main` **trên máy bạn**. `origin/main` chỉ được cập nhật khi fetch (hoặc pull/push). Vì vậy:
- Có người push lên server thì ↓ **chưa** tăng cho đến khi bạn bấm Fetch.
- Dòng branch có tooltip "as of the last fetch" để nhắc điều này.

### Fetch an toàn
Fetch chỉ di chuyển các **remote-tracking branch** (`origin/*`). Branch local và file trong working tree giữ nguyên, nên không cần hộp thoại xác nhận. Test `fetch_updates_ahead_and_behind` kiểm tra rằng file của người khác **không** xuất hiện trong thư mục của bạn.
- **Không dùng `--prune`**: các remote-tracking branch đã bị xóa trên server vẫn được giữ lại. Việc dọn chúng có thể thêm sau, thành một tùy chọn rõ ràng.

### Fetch không chặn các thao tác khác
Stage, unstage và commit dùng chung cờ `busy` (mỗi lần chỉ một thao tác). Fetch có thể kéo dài nhiều giây và không đụng tới index hay file, nên `FetchButton` giữ trạng thái **riêng**. Trong lúc fetch bạn vẫn stage và commit được.

### Đăng nhập
- **HTTPS** cần mật khẩu hoặc token: git không hỏi được (`GIT_TERMINAL_PROMPT=0`), nên báo lỗi `AuthRequired` như ở bước clone. Bước 12–13 sẽ làm phần account và keyring.
- **SSH** dùng ssh-agent hoặc keychain như khi bạn dùng terminal.

### Sửa nhỏ ở phần hủy: SIGTERM rồi SIGCONT
Khi app được chạy từ terminal (`pnpm tauri dev`) mà ssh cần hỏi gì đó (ví dụ host key lần đầu), ssh sẽ đọc từ terminal. Tiến trình git đang ở process group riêng (bước 5), tức là "nhóm nền", nên hệ điều hành **tạm dừng** nó (SIGTTIN). Một tiến trình đang bị dừng sẽ chưa xử lý SIGTERM cho đến khi được chạy tiếp, nên trước đây nút Cancel có thể không có tác dụng. Giờ `terminate` gửi thêm **SIGCONT** ngay sau SIGTERM. Khi app chạy từ Finder thì không có terminal, ssh sẽ báo lỗi ngay thay vì bị dừng.

## Luồng dữ liệu

```
Hiển thị branch (mỗi lần làm mới status, như trước):
RepoView refresh() → getStatus → Rust read_status (git status … --branch -z)
  └─ parse_status: bản ghi "#" → read_header() điền BranchInfo
  └─ RepoStatus { branch, staged, unstaged, untracked, conflicted }
RepoView → <BranchStatus :branch="status.branch">

Fetch:
FetchButton.vue: start()
  ├─ onGitProgress(...)    ← nghe trước, lọc theo opId
  └─ fetchRepo(opId, repoId) → invoke("fetch_repo")
       └─ Rust commands/fetch.rs → run_blocking
            └─ git::fetch::fetch → run_git_with_progress (bước 5)
                 └─ mỗi dòng stderr → emit_progress → event "git-progress"
  └─ xong / hủy → emit("done") → RepoView refresh() → ahead/behind mới
  └─ lỗi → emit("failed", err) → RepoView hiện lỗi
Cancel → cancelOperation(opId) → SIGTERM + SIGCONT cho cả process group
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/models.rs` | `BranchInfo`; `RepoStatus` thêm `branch` |
| `src-tauri/src/git/status.rs` | `--branch`, `read_header`, `parse_ahead_behind`, test |
| `src-tauri/src/git/fetch.rs` (mới) | `fetch` + 4 test với repo "server" bare tạm |
| `src-tauri/src/commands/fetch.rs` (mới) | command `fetch_repo` |
| `src-tauri/src/commands/mod.rs` | `PROGRESS_EVENT` + `emit_progress` chuyển từ `clone.rs` sang, để clone và fetch dùng chung |
| `src-tauri/src/commands/clone.rs` | dùng `emit_progress` |
| `src-tauri/src/git/operations.rs` | `terminate`: SIGTERM + SIGCONT |
| `src/api/operations.ts` (mới) | `cancelOperation`, `onGitProgress` chuyển từ `clone.ts` sang (dùng chung) |
| `src/api/fetch.ts` (mới), `src/types/status.ts` | wrapper + `BranchInfo` |
| `src/components/BranchStatus.vue` (mới) | dòng branch / upstream / ↑↓ |
| `src/components/FetchButton.vue` (mới) | nút Fetch + tiến trình + Cancel |
| `src/views/RepoView.vue`, `src/components/CloneDialog.vue` | gắn component mới; sửa import |

## Khái niệm mới

### Rust
- **Chuyển code chung lên module cha** (`commands/mod.rs`) và gọi bằng `use super::{emit_progress, run_blocking}`, giống tách một static helper ra class tiện ích trong Java.
- **Closure một dòng** `|line| emit_progress(&app, &op_id, line)`: closure **mượn** `app` và `op_id` (không lấy quyền sở hữu), vì chúng vẫn sống trong suốt lời gọi `fetch`.
- **Hàm trả về `Option` dùng `?` liên tiếp** (`parse_ahead_behind`): mỗi `?` gặp `None` thì cả hàm trả `None` ngay, giống một chuỗi `Optional.flatMap` trong Java nhưng đọc thẳng từ trên xuống.
- **`if … { None } else { Some(…) }` là một biểu thức**: gán thẳng vào biến, giống toán tử ba ngôi `a ? b : c` của Java.

### Vue
- **Component tự giữ trạng thái chạy dài** (`FetchButton`) và chỉ báo kết quả cho cha qua `emit("done")` / `emit("failed", err)`. Cha không cần biết opId hay tiến trình.
- **Handler inline trong template**: `@failed="(err) => (error = err)"`. Trong template, `error` tự hiểu là `error.value` (template tự "mở" ref).
- **`<template v-if>`**: nhóm nhiều phần tử theo một điều kiện mà không tạo thêm thẻ HTML.

## Cách thử

Cần một repo **có remote**, ví dụ một repo GitHub đã clone ở bước 5.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Mở repo đã clone, đang ở `main` | `⎇ main → origin/main up to date` |
| 2 | Commit một thay đổi trong app (chưa push) | `↑1` màu xanh lá |
| 3 | Bấm **Fetch** | Nút đổi thành dòng tiến trình (`Receiving objects …%` hoặc `From github.com:…`) + Cancel; xong quay lại nút Fetch |
| 4 | Push một commit từ chỗ khác (web GitHub hoặc thư mục clone khác), rồi Fetch | `↓1` xuất hiện; file trong thư mục của bạn **không đổi** |
| 5 | Trong terminal: `git checkout -b test-local` | (sau khi focus lại app) `⎇ test-local (no upstream)` |
| 6 | `git checkout --detach` | `⎇ HEAD detached at abc1234` màu vàng |
| 7 | Repo nháp `~/tmp/try-commit` (không có remote): bấm Fetch | Xong ngay, không lỗi |
| 8 | (Tùy chọn) repo trên mạng chậm: bấm Fetch rồi Cancel | Dừng lại, không hiện lỗi |
| 9 | Trong lúc Fetch đang chạy: stage một file | Vẫn stage được |
