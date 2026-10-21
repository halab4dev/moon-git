# Bước 11 — Pull, push

## Trạng thái
- `cargo test`: 90/90 pass (thêm 13 test, mỗi test có repo "server" bare tạm):
  - **Pull:** fast-forward; fast-forward bị từ chối khi đã tách nhánh; merge tạo merge commit; rebase đưa commit local lên trên; conflict được báo là `Conflict` (cả merge lẫn rebase); không có upstream; đọc `pull.rebase` / `branch.<name>.rebase`; các cách viết true/false của git.
  - **Push:** push commit; push branch mới và gắn upstream; bị từ chối khi server có commit mới (server **không** bị ghi đè); từ chối tên remote lạ (kể cả `--force`); liệt kê remote.
- `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21). Đã commit.

## Mục tiêu
- Thanh công cụ có **Fetch · Pull · Push**. Mỗi lúc chỉ chạy một thao tác; khi đang chạy, ba nút được thay bằng dòng tiến trình và nút Cancel.
- **Pull** (bạn đã chọn "hỏi mỗi lần"):
  - Chỉ thiếu commit của server → fast-forward luôn, không hỏi.
  - Cả hai bên đều có commit mới → hộp thoại **Rebase / Merge**. Lựa chọn được chọn sẵn theo git config của bạn (máy bạn có `pull.rebase = true` nên chọn sẵn Rebase).
  - Không có gì mới → "Already up to date".
- **Push**:
  - Branch đã có upstream → `git push`.
  - Branch chưa từng push → hộp thoại "Push new branch to origin?", rồi push và gắn upstream.
  - **Không bao giờ force push.** Server có commit mới → báo "Pull first, then push again".

## Lệnh git

| Việc | Lệnh |
|---|---|
| Pull, bước 1 (mạng) | `git fetch --all --progress` (giống nút Fetch, có tiến trình và Cancel) |
| Pull, bước 2: chỉ thiếu commit | `git merge --ff-only @{upstream}` |
| Pull, bước 2: Merge | `git merge --no-edit @{upstream}` |
| Pull, bước 2: Rebase | `git rebase --fork-point @{upstream}` |
| Chọn sẵn Merge/Rebase | `git config --default "" --get branch.<name>.rebase`, rồi `… pull.rebase` |
| Push | `git push --progress` |
| Push lần đầu | `git push --progress --set-upstream <remote> HEAD` |
| Danh sách remote | `git remote` |

### Vì sao không gọi thẳng `git pull`
`git pull` = fetch + merge/rebase trong **một** lệnh, nên app không thể dừng giữa chừng để hỏi "merge hay rebase?" sau khi đã biết có tách nhánh hay không. App tách làm hai bước:
1. **Fetch.** Đây là phần chậm và có thể bị treo (mạng, đăng nhập), nên có tiến trình và **Cancel**.
2. **Đọc lại status** để có ahead/behind mới, rồi quyết định: không có gì mới → dừng; chỉ thiếu commit → fast-forward; cả hai bên đều có commit mới → hỏi.
3. **Merge/rebase cục bộ** với `@{upstream}`, tức branch mà branch hiện tại đang theo dõi (ví dụ `origin/main`). Bước này không dùng mạng, chạy nhanh, và **không có Cancel**: dừng giữa lúc rebase sẽ để lại repo dở dang.

Phần mạng vì vậy chỉ chạy một lần. `--fork-point` là cách `git pull --rebase` vẫn làm, giúp xử lý đúng trường hợp upstream bị người khác rebase lại.

### Trong lúc pull: khóa stage/commit
Merge và rebase sửa file và index, nên `SyncButtons` bật `locked` (`v-model:locked`). RepoView gộp `busy || syncLocked` thành `working` và tắt Stage, Unstage, Commit, Refresh trong lúc đó. Fetch và push không sửa file nên không khóa.

### Khi pull bị conflict
git in báo cáo conflict ra **stdout**, mà lỗi trả về chỉ chứa stderr. Vì vậy `explain_pull_error` đọc lại **status**: nếu có file conflicted thì trả `ErrorKind::Conflict`, kèm hướng dẫn:
- Merge: sửa file, stage rồi commit (hoặc `git merge --abort`).
- Rebase: sửa file, stage rồi `git rebase --continue` (hoặc `git rebase --abort`). Trong lúc rebase dừng, dòng branch hiện `HEAD detached`. Đó là đúng trạng thái của git lúc này.

Xử lý conflict ngay trong app là bước 24. Hiện tại bảng file đã hiện các dòng **Conflicted**.

### Push bị từ chối
Khi server có commit mà máy bạn chưa có, git in `! [rejected] main -> main (fetch first)` hoặc `(non-fast-forward)`. `failure()` trong `operations.rs` nhận ra dòng này và trả `ErrorKind::PushRejected`. App **không** đề nghị force push: theo CLAUDE.md, force push phải có xác nhận riêng, và sẽ được làm sau nếu bạn cần.

### Push branch mới
- Tên remote do frontend gửi lên được kiểm tra với danh sách `git remote` **trước** khi chạy git. Một giá trị như `--force` không bao giờ lọt được vào lệnh (có test).
- Remote được chọn: `origin` nếu có, nếu không thì remote đầu tiên. Chọn remote khi repo có nhiều remote có thể làm sau.

## Luồng dữ liệu

```
SyncButtons.vue  pull()
  perform(...)                     ← mỗi lúc một thao tác; luôn emit("finished", error|null) ở cuối
  ├─ locked = true                 → RepoView: working = true → tắt Stage/Commit
  ├─ withProgress("Fetching…", id => fetchRepo(id, repoId))     ← tiến trình + Cancel (bước 10)
  ├─ getStatus(repoId).branch → ahead / behind
  │    behind = 0  → "Already up to date"
  │    ahead = 0   → mode = fastForward
  │    cả hai > 0  → chooseMode(...) → hộp thoại Rebase/Merge (chọn sẵn = getPullDefault)
  └─ pullRepo(repoId, mode) → invoke("pull_repo")
       └─ Rust git::pull::pull → git merge / git rebase … @{upstream}
            └─ lỗi → explain_pull_error: đọc status → Conflict | lỗi thiếu danh tính | lỗi git
RepoView onSyncFinished(err): await refresh() rồi mới hiện err

SyncButtons.vue  push()
  ├─ chưa có upstream → listRemotes → ask("Push new branch…?") → newUpstream = "origin"
  └─ withProgress("Pushing…", id => pushRepo(id, repoId, newUpstream))
       └─ Rust git::push::push → git push --progress [--set-upstream origin HEAD]
            └─ "[rejected]" → PushRejected
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/pull.rs` (mới) | `pull`, `default_mode`, `explain_pull_error` + 8 test |
| `src-tauri/src/git/push.rs` (mới) | `push`, `list_remotes` + 5 test |
| `src-tauri/src/git/test_support.rs` (mới) | helper test dùng chung: repo "server" bare + các bản clone (fetch.rs cũng dùng) |
| `src-tauri/src/commands/pull.rs`, `push.rs` (mới) | `pull_repo`, `get_pull_default`, `push_repo`, `list_remotes` |
| `src-tauri/src/models.rs` | `PullMode` (fastForward / merge / rebase) |
| `src-tauri/src/error.rs`, `src/types/error.ts` | `Conflict`, `PushRejected` |
| `src-tauri/src/git/operations.rs` | `failure()` nhận ra push bị từ chối |
| `src-tauri/src/git/commit.rs` | `explain_commit_error` thành `pub(crate)` để pull dùng lại |
| `src/api/sync.ts`, `src/types/sync.ts` (mới) | wrapper + `PullMode` |
| `src/components/SyncButtons.vue` (mới) | Fetch/Pull/Push + hộp thoại Rebase/Merge, **thay** `FetchButton.vue` của bước 10 |
| `src/views/RepoView.vue` | `syncLocked`, `working`, `onSyncFinished` |

## Khái niệm mới

### Rust
- **`#[cfg(test)] pub mod test_support`**: module chỉ được biên dịch khi chạy test, chứa helper dùng chung giữa các file test. Giống thư mục `src/test/java` có class tiện ích dùng chung.
- **`match` trả về một slice**: `let args: &[&str] = match mode { … => &["merge", …], … }`. Mỗi nhánh là một mảng hằng, gán thẳng vào biến.
- **`matches!(x, "a" | "b" | …)`**: so một giá trị với nhiều mẫu, như `switch` nhiều `case` cùng trả `true`.
- **`if let Ok(status) = read_status(repo)`**: chỉ xử lý khi thành công, bỏ qua lỗi. Dùng ở đây vì nếu không đọc được status thì vẫn còn lỗi gốc để báo.

### Vue / TS
- **Biến một hộp thoại thành hàm `await` được**: `chooseMode()` trả về `new Promise(resolve => answer = resolve)`. Khi bạn bấm nút, `closeChoice(mode)` gọi `answer(mode)` và đoạn `await chooseMode(...)` chạy tiếp. Tương tự `CompletableFuture` trong Java: tạo future, giữ nó lại, gọi `complete(...)` khi có kết quả.
- **Hàm bậc cao chung khung** `perform(action)` và `withProgress(label, start)`: đưa phần lặp lại (cờ running, lắng nghe tiến trình, xử lý hủy, emit kết quả) ra một chỗ. Ba thao tác chỉ còn phần riêng của mình.
- **`emit` kèm lỗi thay vì hai event riêng**: `finished(error | null)`, để cha có thể **làm mới trước, hiện lỗi sau**. Nếu tách hai event `done`/`failed`, lần làm mới thành công sẽ xóa mất dòng lỗi vừa hiện.

## Cách thử

Nên dùng một repo GitHub thử nghiệm (đã clone bằng SSH hoặc có credential helper) và một bản clone thứ hai trong terminal để giả làm "người khác", ví dụ:

```bash
git clone <url> ~/tmp/other && cd ~/tmp/other
```

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Không có gì mới, bấm **Pull** | "Fetching…" rồi "Already up to date" |
| 2 | Từ `~/tmp/other`: commit + push. Trong app bấm Pull | Fast-forward, **không hỏi**; file mới xuất hiện; `up to date` |
| 3 | Commit trong app (↑1), bấm **Push** | "Pushing…" / `Writing objects …%`, rồi `up to date` |
| 4 | `~/tmp/other` push một commit; trong app commit một file khác rồi bấm **Push** | Lỗi "The server has commits that you don't have yet. Pull first…" |
| 5 | Bấm **Pull** | Hộp thoại Rebase/Merge, chọn sẵn **Rebase**; chọn Rebase → `↑1`, lịch sử thẳng (`git log --oneline --graph`) |
| 6 | Lặp lại 4 nhưng chọn **Merge** | `↑2` (commit của bạn + merge commit) |
| 7 | Cả hai bên cùng sửa **cùng một dòng** của một file, rồi Pull | Lỗi "The pull stopped because 1 file(s) have conflicts…", bảng có dòng **Conflicted**; xử lý trong terminal (`git rebase --abort` hoặc sửa file rồi `--continue`) |
| 8 | Terminal: `git checkout -b try-push`, commit một file, trong app bấm **Push** | Hộp thoại "Push New Branch"; Push → `⎇ try-push → origin/try-push up to date` |
| 9 | Đang Fetch/Pull: thử bấm Stage | Trong lúc **Pull**, Stage/Commit bị mờ; trong lúc Fetch/Push thì vẫn dùng được |

Sau bước 8 bạn có thể xóa branch thử trên GitHub. App chưa có chức năng xóa branch (bước 19).

> Nếu giao diện trông lạ ngay sau khi app tự build lại, bấm ⌘R để tải lại cửa sổ.
