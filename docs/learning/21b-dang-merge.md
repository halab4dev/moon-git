# Bước 21b — Trạng thái "đang merge": Abort, message điền sẵn, merge commit / commit thường

## Trạng thái
- `cargo test`: 155/155 pass. Thêm 6 test:
  - status báo đang merge, kèm message git chuẩn bị sẵn;
  - message bỏ các dòng `#` (danh sách conflict git thêm vào);
  - Abort: branch giữ nguyên, thay đổi của merge biến mất; abort khi không merge → lỗi;
  - commit lúc đang merge → merge commit 2 cha;
  - **Simple commit** lúc đang merge → commit 1 cha, đủ file;
  - không cho amend lúc đang merge.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-23), sau khi thêm màu cho dòng báo. Đã commit.

## Mục tiêu
Khi repo **đang merge** (sau **Merge to Working Tree**, hoặc merge dừng vì conflict):
- **Dòng báo** dưới tiêu đề repo, nền **xanh dương** khi chỉ chờ commit, nền **đỏ** khi có conflict (bạn yêu cầu, để dễ phân biệt): "**Merging:** Merge branch 'feature' into main.", rồi một trong hai:
  - "Review the changes, then commit to finish.";
  - "N file(s) have conflicts: fix them, then stage them."
- Nút **Abort Merge** trên dòng báo, **có hộp thoại xác nhận** (CLAUDE.md). Abort bỏ toàn bộ thay đổi của merge, kể cả phần sửa conflict.
- **Message commit được điền sẵn** bằng message git đã chuẩn bị, nếu ô message đang trống. Khi merge kết thúc theo cách khác (Abort, hoặc commit trong terminal), message đó được xóa đi nếu bạn chưa sửa nó.
- **Commit box** thay ô "Amend" bằng hai lựa chọn, giống SmartGit:
  - **Merge commit** (mặc định): commit 2 cha, lịch sử ghi lại là đã merge branch kia. Nút ghi **Commit Merge**.
  - **Simple commit**: commit thường 1 cha, giống `merge --squash`. Thay đổi giống hệt, nhưng graph không nối sang branch kia.

```
Merge commit:              Simple commit:
  *   Merge … ← main         * Bring in feature ← main
  |\                         |
  | * f ← feature            | * f ← feature
  * | m                      * | m
  |/                         |/
  * base                     * base
```

## git lưu trạng thái merge ở đâu
`git status --porcelain` **không** báo đang merge. git giữ trạng thái này bằng các file trong `.git/`:

| File | Nghĩa |
|---|---|
| `MERGE_HEAD` | commit đang được merge vào. Có file này nghĩa là "đang merge". `git commit` thấy nó thì tạo merge commit 2 cha |
| `MERGE_MSG` | message git chuẩn bị cho merge commit. Khi có conflict, git thêm các dòng `# Conflicts:` / `#\tfile` |
| `MERGE_MODE` | tùy chọn của merge (ví dụ `no-ff`) |

`git commit` (kể cả trong terminal) hoặc `git merge --abort` xóa các file này.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Đường dẫn file trạng thái | `git rev-parse --git-path MERGE_HEAD --git-path MERGE_MSG` |
| Abort | `git merge --abort` |
| Simple commit | `git update-ref -d MERGE_HEAD`, rồi `git commit …` như bình thường |

### Ghi chú
- **`--git-path`** cho đường dẫn thật của file, kể cả khi repo là worktree hoặc thư mục `.git` nằm chỗ khác. Không tự ghép cứng `.git/MERGE_HEAD`.
- **Mỗi lần đọc status có thêm một lệnh git** (`rev-parse`), rất nhanh. Bước 24 sẽ dùng cùng chỗ này để phát hiện đang rebase / cherry-pick.
- **Simple commit = xóa `MERGE_HEAD` trước khi commit.** Không có `MERGE_HEAD`, `git commit` tạo commit thường với đúng những gì đã stage. git tự xóa `MERGE_MSG`/`MERGE_MODE` sau khi commit (đã thử).
  - App kiểm tra message / file đã stage / conflict **trước** khi xóa `MERGE_HEAD`. Nếu commit vẫn thất bại sau đó (ví dụ thiếu tên/email), các thay đổi vẫn ở trạng thái đã stage, và lần commit sau là commit thường, đúng như bạn đã chọn.
- **Không amend lúc đang merge**: git không cho, nên app ẩn ô Amend (và tự bỏ tick nếu đang tick) và kiểm tra cả ở Rust.
- **Abort và thay đổi của bạn**: bước 21a đã để git từ chối merge khi nó phải đụng vào file bạn đang sửa dở, nên `merge --abort` chỉ bỏ những gì merge mang vào. Thay đổi ở các file khác vẫn còn.

## Luồng dữ liệu

```
Merge to Working Tree (21a) → RepoStatus mới
  └─ Rust read_status → parse_status + merge_state (rev-parse --git-path, đọc MERGE_MSG)
       → RepoStatus.merge = { message: "Merge branch 'feature' into main" }
RepoView
  ├─ computed merge → dòng báo "Merging: …" + nút Abort Merge
  ├─ watch(merge.message) → điền commitMessage (nếu trống), bỏ tick Amend
  └─ <CommitBox :merging v-model:simple> → radio Merge commit / Simple commit
Commit → commitChanges(repoId, message, amend=false, simpleCommit) → invoke("commit_changes")
  └─ Rust git::commit::commit → kiểm tra → (simple) update-ref -d MERGE_HEAD → git commit -F -
Abort Merge → ask(...) → abortMerge(repoId) → invoke("abort_merge") → git merge --abort
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/models.rs` | `RepoStatus.merge`, `MergeState` |
| `src-tauri/src/git/status.rs` | `merge_state`, `clean_message` |
| `src-tauri/src/git/merge.rs` | `abort` + 3 test |
| `src-tauri/src/git/commit.rs` | tham số `simple`, chặn amend lúc merge + 3 test |
| `src-tauri/src/commands/commit.rs`, `commands/merge.rs`, `lib.rs` | `simple_commit`, `abort_merge` |
| `src/types/status.ts`, `src/api/commit.ts`, `src/api/merge.ts` | kiểu + wrapper |
| `src/components/CommitBox.vue` | radio Merge commit / Simple commit khi đang merge |
| `src/views/RepoView.vue` | dòng báo, Abort (xác nhận), điền sẵn message |

## Khái niệm mới

### Rust
- **`let (Some(a), Some(b)) = (x, y) else { … }`**: ghép hai `Option` thành tuple rồi khớp cả hai cùng lúc. Chỉ đi tiếp khi **cả hai** đều có giá trị.
- **`unwrap_or_default()`**: lỗi thì dùng giá trị mặc định (`String` rỗng). Ở đây: không đọc được `MERGE_MSG` thì message rỗng, vẫn báo đang merge.
- **`Path::join` với đường dẫn tuyệt đối**: `repo.join("/abs/path")` cho ra `/abs/path`. Vì vậy cùng một dòng code xử lý được cả đường dẫn tương đối lẫn tuyệt đối mà `--git-path` trả về.

### Vue
- **Radio gắn với boolean**: `<input type="radio" v-model="simple" :value="false">`. Viết `:value` (có dấu `:`) để giá trị là boolean `false` thật, không phải chuỗi `"false"`.
- **`watch` có giá trị cũ** `(next, previous) => …`: biết merge vừa **bắt đầu** (`null` → có) hay vừa **kết thúc** (có → `null`), để điền hoặc xóa message đúng lúc.
- **`||` thay vì `??`** ở `mergeTitle`: `""` (message rỗng) cũng phải đổi thành `"Merge"`. `??` chỉ đổi `null`/`undefined`.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R nếu giao diện trông lạ.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Merge một branch bằng **Merge to Working Tree** | Dòng báo "Merging: Merge branch '…' into main. Review the changes…"; ô message đã có "Merge branch '…' into main"; commit box có **Merge commit** (đã chọn) / **Simple commit**, không có Amend; nút **Commit Merge** |
| 2 | Bấm **Commit Merge** | Dòng báo biến mất; History: merge commit 2 cha |
| 3 | Làm lại bước 1, chọn **Simple commit** → **Commit** | History: commit thường 1 cha (graph không nối sang branch kia), đủ file của branch kia |
| 4 | Làm lại bước 1 → **Abort Merge** → Cancel | Không có gì thay đổi |
| 5 | **Abort Merge** → **Abort Merge** | Dòng báo biến mất; tab Changes trống (các file của merge không còn); message được xóa khỏi ô |
| 6 | Merge hai branch cùng sửa một dòng (conflict) | Dòng báo "1 file(s) have conflicts: fix them, then stage them."; **Abort Merge** đưa mọi thứ về như trước |
| 7 | Bắt đầu merge, rồi chạy `git merge --abort` trong terminal → quay lại app | Dòng báo tự biến mất (status được tải lại khi app được focus) |
| 8 | Đang merge, sửa message thành chữ khác → Abort | Chữ bạn gõ **được giữ** trong ô (chỉ message do app điền mới bị xóa) |
