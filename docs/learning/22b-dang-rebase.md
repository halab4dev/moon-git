# Bước 22b — Trạng thái "đang rebase": Continue / Skip / Abort

## Trạng thái
- `cargo test`: 166/166 pass. Thêm 5 test:
  - status cho biết rebase dừng ở đâu: branch, target, "commit 1 of 2", commit bị dừng;
  - Continue: từ chối khi còn conflict, từ chối khi có thay đổi chưa stage (nêu tên file), rồi chạy hết, lịch sử đúng thứ tự, nội dung file là bản đã sửa;
  - Skip: commit bị dừng bị bỏ đi, phần còn lại vẫn được replay;
  - Abort: về lại như trước khi rebase; abort khi không rebase → lỗi;
  - đọc dòng commit bị dừng trong file `done`.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-24). Đã commit.

## Mục tiêu
Khi rebase dừng giữa chừng (thường vì conflict), dưới tên repo có một **dòng báo** (đỏ khi có conflict, xanh khi không):

> **Rebasing:** "main" onto 1234abcd, commit 1 of 2 (5678ef90 main side). 1 file(s) have conflicts: fix them, stage them, then Continue. **[Continue] [Skip…] [Abort…]**

| Nút | Làm gì | Hỏi trước? |
|---|---|---|
| **Continue** | Commit bản đã sửa của commit đang dừng, rồi replay các commit còn lại. Bị mờ khi còn file conflict | Không |
| **Skip…** | Bỏ commit đang dừng ra khỏi branch (kể cả phần bạn đã sửa), replay phần còn lại | **Có** |
| **Abort…** | Hủy cả rebase: branch về đúng chỗ cũ | **Có** |

- Nếu commit tiếp theo cũng conflict, rebase lại dừng. Dòng báo cập nhật thành "commit 2 of 2…", và có thông báo "stopped again".
- Trong lúc rebase, **commit box bị khóa**: Continue thay cho Commit.

### Giới hạn tạm thời (tới bước 24)
App vẫn **chưa cho stage file conflict** (quyết định ở bước 7, vì `git add` trên file conflict nghĩa là "đã giải quyết xong", một việc thuộc bước 24). Vì vậy, sau khi sửa file, bạn tạm chạy **`git add <file>`** trong terminal, rồi quay lại app bấm **Continue**.

## git lưu trạng thái rebase ở đâu
Thư mục `.git/rebase-merge/` (git cũ hơn, hoặc `git am`: `.git/rebase-apply/`), mỗi giá trị là một file nhỏ:

| File | Nội dung |
|---|---|
| `head-name` | branch đang rebase: `refs/heads/main` |
| `onto` | id commit target |
| `msgnum` / `end` | đang ở commit thứ mấy / tổng số (rebase-apply: `next` / `last`) |
| `done` | các lệnh đã chạy, dòng cuối là commit đang dừng: `pick <id> <subject>` |
| `git-rebase-todo` | các commit còn lại |

Khi đang rebase, git **không** tạo `MERGE_HEAD`, nên dòng báo merge không hiện cùng lúc.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Đường dẫn (cùng lệnh với merge) | `git rev-parse --git-path MERGE_HEAD --git-path MERGE_MSG --git-path rebase-merge --git-path rebase-apply` |
| Continue | `GIT_EDITOR=true git rebase --continue` |
| Skip | `git rebase --skip` |
| Abort | `git rebase --abort` |

### Ghi chú
- **`GIT_EDITOR=true`**: `rebase --continue` muốn mở trình soạn thảo để bạn sửa message của commit vừa giải quyết. App không có cửa sổ nào cho nó, nên git sẽ đợi mãi. `true` là lệnh có sẵn trên macOS/Linux, không làm gì và thoát với mã 0. git coi như bạn đã lưu message cũ mà không sửa. Hàm mới `run_git_without_editor` trong `runner.rs` làm việc này; cherry-pick (bước 23) sẽ dùng lại.
- **Một lệnh `rev-parse` cho cả merge lẫn rebase**: `read_status` (bước 21b) giờ hỏi 4 đường dẫn trong một lần gọi git.
- **Kiểm tra trước Continue**:
  - còn file conflict → báo số file;
  - có thay đổi chưa stage ở file tracked → báo tên file (git cũng từ chối, nhưng không nói rõ file nào).
- **Commit trở thành rỗng sau khi giải quyết** (bạn chọn lại đúng bản của target): git 2.33 tự bỏ commit đó và đi tiếp (đã thử). Nếu git báo "No changes", app gợi ý **Skip**.
- **Sau khi Continue / Skip lỗi**: app tải lại status (file có thể đã đổi) nhưng **giữ** thông báo lỗi, giống sau khi pull lỗi.

## Frontend
- **`OperationBanner.vue`** (mới): thanh màu dùng chung. Dòng báo merge của bước 21b được chuyển sang dùng nó (sửa nhỏ, không đổi giao diện). Cherry-pick sẽ dùng lại.
  - prop `label` ("Merging:", "Rebasing:"), `conflicts` (đỏ hay xanh);
  - slot mặc định = câu mô tả, slot `actions` = các nút.
- **RepoView**:
  - `rebase`, `conflictCount`, `rebaseTitle` là `computed` từ status;
  - `runRebase(action)` hỏi trước Skip/Abort, rồi gọi lệnh;
  - commit box `busy` khi đang rebase.

## Luồng dữ liệu

```
Rebase (22a) dừng vì conflict → read_status
  └─ add_operation_state → rev-parse --git-path … → .git/rebase-merge có tồn tại
       └─ read_rebase_state(dir) → head-name, onto, msgnum, end, done → RebaseState
RepoView → <OperationBanner label="Rebasing:" :conflicts> → [Continue] [Skip…] [Abort…]
Sửa file → git add (terminal) → quay lại app (focus → refresh) → nút Continue sáng lên
Continue → runRebase("continue") → runRebaseAction(repoId, "continue") → invoke("run_rebase_action")
  └─ Rust git::rebase::run → kiểm tra → GIT_EDITOR=true git rebase --continue
       ├─ xong → status.rebase = null → dòng báo biến mất, branch "main" trở lại header
       └─ commit sau cũng conflict → Conflict "stopped again" → refresh, giữ lỗi
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/rebase.rs` | `read_rebase_state`, `stopped_commit`, `run` (continue/skip/abort), `explain_continue_error` + 5 test |
| `src-tauri/src/git/status.rs` | `add_operation_state`: một `rev-parse` cho merge + rebase |
| `src-tauri/src/git/runner.rs` | `run_git_without_editor` |
| `src-tauri/src/models.rs` | `RepoStatus.rebase`, `RebaseState`, `RebaseAction` |
| `src-tauri/src/commands/rebase.rs`, `lib.rs` | `run_rebase_action` |
| `src-tauri/src/git/commit.rs` | thông báo khi commit lúc đang rebase: chỉ tới nút Continue |
| `src/types/status.ts`, `src/types/rebase.ts`, `src/api/rebase.ts` | kiểu + wrapper |
| `src/components/OperationBanner.vue` (mới) | thanh màu dùng chung |
| `src/views/RepoView.vue` | dòng báo rebase, `runRebase`, dòng báo merge dùng `OperationBanner` |

## Khái niệm mới

### Rust
- **Closure dùng như hàm nhỏ cục bộ**: `let read = |name: &str| fs::read_to_string(dir.join(name)).ok()…;`. Closure "mượn" `dir` từ bên ngoài, gọi như hàm: `read("onto")`. Closure thứ hai (`number`) còn gọi lại closure thứ nhất. Giống lambda trong Java, nhưng không cần interface.
- **Chuỗi xử lý `Option`**: `read(a).or_else(|| read(b)).and_then(|t| t.parse().ok()).unwrap_or(0)`:
  - thử file thứ nhất, không có thì thử file thứ hai;
  - có chữ thì parse, parse lỗi thì thành `None`;
  - cuối cùng dùng 0.

  Như `Optional.or(...).flatMap(...).orElse(0)` trong Java.
- **`.env("GIT_EDITOR", "true")` trên `Command`**: đặt biến môi trường chỉ cho process git này, không ảnh hưởng app.
- **`lines().rev().find(...)`**: duyệt từ cuối lên để lấy dòng không rỗng cuối cùng.

### Vue
- **Component chỉ có slot** (`OperationBanner`): component lo khung (màu, bố cục), nơi dùng lo nội dung. Hai slot: mặc định (không tên) và `#actions`.
- **Giữ lỗi qua một lần refresh**: `refresh()` thành công sẽ xóa lỗi, nên lỗi được lưu vào biến trước, rồi gán lại sau.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

**Chuẩn bị** (trong terminal, repo thử):
```
git switch main && echo main > a.txt && git commit -qam "main side" && echo 2 > b.txt && git add b.txt && git commit -qm "m2"
git switch -c side HEAD~2 && echo side > a.txt && git commit -qam "side side"
git switch main
```
(`a.txt` phải đã có từ trước trong repo; hoặc dùng bất kỳ hai branch nào sửa cùng một dòng.)

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Trong app: **Rebase "main" onto ▸ side…** → Rebase | Lỗi conflict; thanh **đỏ** "Rebasing: "main" onto …, commit 1 of 2 (… main side). 1 file(s) have conflicts…"; **Continue** mờ; commit box bị khóa |
| 2 | Sửa `a.txt` (bỏ các dòng `<<<<<<<`…), chạy `git add a.txt` trong terminal, quay lại app | Thanh chuyển **xanh**: "Stage your fixes, then Continue…"; **Continue** sáng |
| 3 | **Continue** | Thanh biến mất; header lại `⎇ main`; History: `side side` → `main side` → `m2` thẳng hàng |
| 4 | Làm lại từ đầu (reset trong terminal hoặc tạo lại branch) → khi dừng → **Skip…** → Cancel | Không có gì đổi |
| 5 | **Skip…** → Skip Commit | Rebase xong; trong History không còn `main side`, `m2` vẫn còn |
| 6 | Làm lại → khi dừng → **Abort…** → Abort Rebase | Mọi thứ như trước khi rebase |
| 7 | Khi dừng, sửa và `git add`, rồi sửa tiếp file đó (không add) → **Continue** | Lỗi "Stage your changes before continuing…: a.txt" |
