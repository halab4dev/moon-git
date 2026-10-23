# Bước 22a — Rebase branch hiện tại lên branch khác

Bước 22 được chia làm hai, giống merge:
- **22a (bước này):** rebase (không interactive), có xác nhận trước, mở từ menu branch và History.
- **22b:** trạng thái "đang rebase" khi dừng vì conflict:
  - dòng báo màu, kèm vị trí (commit thứ mấy / tổng số);
  - nút **Continue**, **Skip**, **Abort** (có xác nhận).

## Trạng thái
- `cargo test`: 161/161 pass. Thêm 6 test:
  - xem trước: số commit sẽ được replay, đã bắt đầu từ target rồi;
  - đếm số commit đã push;
  - rebase: lịch sử thẳng, commit có id mới, vẫn ở branch cũ;
  - từ chối khi có thay đổi chưa commit, nhưng cho phép file untracked;
  - dừng khi conflict, phát hiện được "đang rebase"; không cho rebase chồng, không cho commit từ commit box;
  - chỉ nhận ref đang tồn tại hoặc commit id.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-23). Đã commit.

## Rebase là gì
Rebase `main` lên `feature`: các commit **chỉ có trên `main`** được "chơi lại" từng cái một, theo thứ tự, trên đỉnh của `feature`.

```
Trước:                        Sau "Rebase main onto feature":
  * m2  ← main                  * m2'  ← main      (commit mới, id mới)
  * m1                          * m1'
  | * f ← feature               * f    ← feature
  |/                            * init
  * init
```

- Kết quả: lịch sử **thẳng một đường**, không có merge commit.
- `m1'`, `m2'` có **cùng thay đổi** với `m1`, `m2` nhưng là **commit mới** (id khác). Commit cũ biến mất khỏi branch.
- Nếu `m1`, `m2` **đã push**, server vẫn giữ bản cũ. Lần push sau phải dùng `--force`. App không bao giờ force, và cảnh báo trước khi rebase.
- So với merge (bước 21): merge giữ nguyên commit cũ và thêm một commit nối hai nhánh; rebase viết lại commit của bạn.

## Mở từ đâu
- **Menu `⎇ main ▾`** → **Rebase "main" onto ▸**: các branch local khác và các remote branch.
- **History → chuột phải một commit**:
  - **Rebase Current Branch onto \<x\>…** cho mỗi branch/tag trên commit đó;
  - **… onto Commit 1234abcd…** nếu commit không có nhãn.

Trước khi làm, app **luôn hỏi**:
- "The 2 commit(s) of "main" that are not in "feature" will be replayed on top of it, as new commits."
- Nếu có commit đã push: thêm cảnh báo (biểu tượng warning) về `--force`.
- Nếu `main` không có commit riêng: "it simply moves forward".
- Nếu `main` đã bắt đầu từ đỉnh của `feature`: "Nothing to Rebase", không hỏi.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Commit chung | `git merge-base HEAD <target>` |
| Số commit sẽ replay | `git rev-list --count <target>..HEAD` |
| Số commit chưa push trong đó | `git rev-list --count <target>..HEAD --not --remotes` |
| Rebase | `git rebase --no-autostash --end-of-options <target>` |
| Đang rebase? | `git rev-parse --git-path rebase-merge --git-path rebase-apply` → thư mục có tồn tại không |

### Ghi chú
- **`A..B`** trong `rev-list`: các commit có trong B mà không có trong A. **`--not --remotes`**: bỏ thêm mọi commit có trên một remote branch nào đó. Hai số trừ nhau ra số commit đã push.
- **Không có thay đổi chưa commit** (đã stage hoặc chưa): git cũng từ chối, nhưng app báo trước bằng lời dễ hiểu ("Commit or stash them"). File untracked thì được.
- **`--no-autostash`**: không bao giờ tự stash sau lưng bạn, dù cấu hình `rebase.autoStash` của bạn là gì.
- **Merge commit trên branch** bị bỏ đi khi rebase (đây là mặc định của git, kết quả là một đường thẳng).
- **Không merge / rebase / commit khi đang rebase dở.** Khi rebase dừng giữa chừng:
  - HEAD **detached** (git đang đứng ở commit đã replay gần nhất);
  - trạng thái nằm trong thư mục `.git/rebase-merge`.

  Commit từ commit box lúc này sẽ không vào đúng chỗ, nên app chặn và chỉ cách tiếp tục. Cho tới bước 22b, dùng `git rebase --continue` / `--abort` trong terminal.
- **Conflict**: app đọc status (như merge), tải lại màn hình để thấy file **Conflicted**, rồi giải thích cách làm tiếp.
- Các hàm `resolve_source`, `current_branch`, `commit_of` của `merge.rs` thành `pub(crate)` để `rebase.rs` dùng lại (cùng loại target: branch, remote, tag, commit id).

## Luồng dữ liệu

```
⎇ main ▾ → Rebase "main" onto ▸ feature…  (hoặc History → chuột phải → Rebase Current Branch onto feature…)
  └─ branchMenus confirmRebase → getRebasePreview(repoId, "refs/heads/feature") → invoke("get_rebase_preview")
       └─ Rust git::rebase::preview → merge-base, rev-list --count (2 lần) → RebasePreview
  ├─ upToDate → message("Nothing to Rebase")
  └─ ask("… will be replayed … [N already pushed …]")
       └─ rebaseCurrent(repoId, onto) → invoke("rebase_current")
            └─ Rust git::rebase::rebase → kiểm tra → git rebase … → read_status
       ├─ OK → onStatus → RepoView applyStatus → History vẽ lại thành đường thẳng
       └─ conflict → getStatus → onStatus, rồi onError (giải thích)
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/rebase.rs` (mới) | `preview`, `rebase`, `rebase_in_progress` + 6 test |
| `src-tauri/src/commands/rebase.rs` (mới), `lib.rs` | `get_rebase_preview`, `rebase_current` |
| `src-tauri/src/models.rs` | `RebasePreview` |
| `src-tauri/src/git/merge.rs` | dùng chung helper (`pub(crate)`); không merge khi đang rebase |
| `src-tauri/src/git/commit.rs` | không commit khi đang rebase |
| `src/types/rebase.ts`, `src/api/rebase.ts` (mới) | kiểu + wrapper |
| `src/components/branchMenus.ts` | submenu **Rebase onto**, mục Rebase trong menu commit, `confirmRebase` |

## Khái niệm mới

### Rust
- **`let Ok(x) = … else { return false; }`**: `let … else` (bước 5) dùng với `Result`. Không chạy được lệnh git thì coi như "không đang rebase", không làm hỏng việc gọi hàm.
- **`extend_from_slice(&[…])`**: nối một slice vào cuối `Vec` (copy từng phần tử), giống `addAll` trong Java.
- **`pub(crate)` để dùng chung giữa các module anh em**: `merge.rs` mở hàm cho cả crate, nhưng vẫn không lộ ra ngoài crate. Struct `Source` có field `pub` nhưng `kind` vẫn private: module khác đọc được `revision`/`label` mà không sửa được `kind`.
- **Gọi hàm bằng đường dẫn đầy đủ** `crate::git::commit::commit(...)` trong test, khi tên `commit` có thể nhầm với biến/hàm khác ở gần.

### TS
- **Destructuring** `const { branch, commits, pushed } = preview;`: lấy nhiều field ra biến cùng tên trong một dòng.
- **Câu thông báo ghép dần** (`let text = …; if (…) text += …`): thêm phần cảnh báo chỉ khi cần.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo thử: `main` và `feature` đều có commit riêng. Ở `main` → `⎇ main ▾` → **Rebase "main" onto ▸ feature…** | Hỏi "The N commit(s) of "main" … will be replayed …" → **Rebase** |
| 2 | Xem History | `main` thành đường thẳng phía trên `feature`; commit của `main` có id mới |
| 3 | Rebase lần nữa lên `feature` | "Nothing to Rebase" |
| 4 | Repo có remote, `main` có commit đã push và commit chưa push → rebase lên branch khác | Hộp thoại kiểu cảnh báo, có câu "N of them are already pushed … needs --force" → **Cancel** |
| 5 | Sửa dở một file → rebase | Lỗi "You have uncommitted changes. Commit or stash them before rebasing." |
| 6 | Hai branch cùng sửa một dòng → rebase | Lỗi conflict kèm hướng dẫn; tab Changes có file **Conflicted**; header hiện HEAD detached |
| 7 | Trong lúc ở bước 6: sửa file, `git add` trong terminal → bấm **Commit** trong app | Lỗi "A rebase is in progress. Finish it with `git rebase --continue` …" |
| 8 | Dọn: `git rebase --abort` trong terminal → quay lại app | Mọi thứ như trước khi rebase |
| 9 | History → chuột phải một commit có nhãn → **Rebase Current Branch onto …** | Hộp thoại xác nhận như bước 1 |
