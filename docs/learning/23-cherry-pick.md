# Bước 23 — Cherry-pick (Commit / Apply to Working Tree)

## Trạng thái
- `cargo test`: 186/186 pass. Thêm 7 test:
  - Commit: commit mới (id khác), cùng message;
  - Apply to Working Tree: thay đổi đã stage, HEAD không đổi, không để lại trạng thái chờ;
  - commit đã có trên branch → báo "already has", không đổi gì, không để lại trạng thái chờ;
  - conflict → chờ; status cho biết commit nào; Continue từ chối khi chưa giải quyết, rồi commit với message gốc;
  - Abort → về như trước; không cho cherry-pick chồng;
  - merge commit → lấy thay đổi so với cha thứ nhất;
  - chỉ nhận commit id.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-24). Đã commit.

## Cherry-pick là gì
Lấy **thay đổi** của một commit (thường ở branch khác) và áp vào branch hiện tại, thành một commit mới:

```
Trước:                       Sau "Cherry-Pick f2" (Commit):
  * m ← main                   * f2' ← main     (cùng thay đổi + message + tác giả với f2, id mới)
  | * f2 ← feature             * m
  | * f1                       | * f2 ← feature
  |/                           | * f1
  * base                       |/
                               * base
```
Khác merge: chỉ lấy **một** commit (`f2`), không lấy `f1`, và không nối hai nhánh.

## Mục tiêu
**History → chuột phải một commit → Cherry-Pick…** mở hộp thoại (CLAUDE.md: hai lựa chọn):

| Lựa chọn | Kết quả | Lệnh |
|---|---|---|
| **Commit** (mặc định) | Commit mới trên branch hiện tại: cùng thay đổi, message và tác giả | `git cherry-pick --end-of-options <id>` |
| **Apply to Working Tree** | Chỉ đưa thay đổi vào, **đã stage**, chưa commit. Bạn xem lại rồi tự commit (có thể cùng với thay đổi khác) | `git cherry-pick --no-commit --end-of-options <id>` |

- **Merge commit**: có hai cha nên "thay đổi của nó" không rõ ràng. App lấy thay đổi **so với cha thứ nhất** (`--mainline 1`), đúng như phần chi tiết trong History hiển thị, và hộp thoại ghi rõ điều này.
- **Commit đã có trên branch** (ví dụ cherry-pick hai lần): git dừng lại với một cherry-pick "rỗng". App tự hủy trạng thái đó và báo "The current branch already has the changes of this commit. Nothing was changed."
- **Conflict (chế độ Commit)**:
  - git chờ, và app hiện **thanh màu** "Cherry-picking: 1234abcd "…"" với **Continue** / **Abort…** (Abort hỏi trước);
  - commit box bị khóa, như khi đang rebase;
  - Continue commit kết quả với message gốc (`GIT_EDITOR=true`, như rebase).
- **Conflict (Apply to Working Tree)**: git **không** để lại trạng thái chờ, chỉ có file conflict. Thông báo nói cách làm tiếp (sửa, stage) hoặc cách bỏ (`git reset --merge` trong terminal, cho tới bước 24).

## git lưu trạng thái cherry-pick ở đâu
| File | Nội dung |
|---|---|
| `CHERRY_PICK_HEAD` | id commit đang được cherry-pick. Có file này nghĩa là "đang chờ" |
| `MERGE_MSG` | message sẽ dùng (dòng đầu = tiêu đề của commit gốc), như khi merge |

`read_status` hỏi thêm đường dẫn `CHERRY_PICK_HEAD` trong cùng lệnh `rev-parse --git-path …` (giờ là 5 đường dẫn).

## Ghi chú
- **Không cherry-pick khi**:
  - có file conflict;
  - đang có merge / rebase / cherry-pick dở;
  - HEAD detached.

  Merge, rebase, reset cũng từ chối khi đang có cherry-pick dở.
- **Chỉ nhận commit id** (`check_commit_id`, như bước 18), cộng `--end-of-options` (cherry-pick có hỗ trợ, đã thử).
- **Thay đổi chưa commit bị đè** → báo tên file, git không đổi gì.

## Luồng dữ liệu

```
History → chuột phải commit → Cherry-Pick…
  └─ branchMenus askCherryPick → dialogs.askCherryPick({ commitId, subject, isMerge, submit })
       └─ <CherryPickDialog> (Commit / Apply to Working Tree) → submit(mode)
            └─ cherryPickCommit(repoId, id, mode) → invoke("cherry_pick_commit")
                 └─ Rust git::cherry_pick::cherry_pick → kiểm tra → (merge? --mainline 1)
                      → git cherry-pick [--no-commit] --end-of-options <id>
            ├─ OK → onStatus → RepoView applyStatus
            ├─ conflict → getStatus → onStatus, onError (hộp thoại đóng)
            │     └─ status.cherryPick → <OperationBanner label="Cherry-picking:"> [Continue] [Abort…]
            │          └─ runCherryPickAction(repoId, "continue"|"abort") → git cherry-pick --continue|--abort
            └─ lỗi khác (đã có, đè thay đổi…) → hiện trong hộp thoại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/cherry_pick.rs` (mới) | `cherry_pick`, `cherry_pick_in_progress`, `run` (continue/abort), `explain_pick_error` + 7 test |
| `src-tauri/src/commands/cherry_pick.rs` (mới), `lib.rs` | `cherry_pick_commit`, `run_cherry_pick_action` |
| `src-tauri/src/models.rs` | `RepoStatus.cherry_pick`, `CherryPickState`, `CherryPickMode`, `CherryPickAction` |
| `src-tauri/src/git/status.rs` | đọc `CHERRY_PICK_HEAD` + `MERGE_MSG` |
| `src-tauri/src/git/merge.rs`, `rebase.rs`, `reset.rs` | từ chối khi đang cherry-pick dở |
| `src/types/cherryPick.ts`, `src/api/cherryPick.ts` (mới), `src/types/status.ts` | kiểu + wrapper |
| `src/components/CherryPickDialog.vue` (mới) | hộp thoại 2 lựa chọn |
| `src/stores/dialogs.ts`, `src/App.vue` | `cherryPickRequest`, gắn hộp thoại |
| `src/components/branchMenus.ts` | mục **Cherry-Pick…**, `askCherryPick` |
| `src/views/RepoView.vue` | thanh "Cherry-picking:", `runCherryPick`, khóa commit box |

## Khái niệm mới

### Rust
- **Match guard trên `Err`**: `Err(err) if is_empty_pick(&err) => …` rồi `Err(err) => …`. Cùng ý với `Some(x) if …` (bước 19a), nhưng trên `Result`: chọn cách xử lý theo **nội dung** lỗi.
- **`let _ = …`**: cố ý bỏ qua kết quả. Ở đây abort một cherry-pick rỗng mà có lỗi cũng không sao, vì thông báo chính vẫn đúng. Không có `let _`, Rust cảnh báo "unused `Result`", vì quên xử lý lỗi thường là bug.
- **Closure không tham số** `let message = || clean_message(…)`: chỉ đọc file `MERGE_MSG` khi thật sự cần (lazy), và dùng được ở hai chỗ.

### Vue
- Không có gì mới: hộp thoại, thanh màu, store, menu đều dùng lại mẫu của merge (21), rebase (22b) và reset (22c). Đây là lợi ích của việc tách `OperationBanner` và các hộp thoại theo cùng một khuôn.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `branchy` (hoặc repo có 2 branch), ở `main` → History → hiện cả branch kia (refs ▾) → chuột phải một commit của branch kia → **Cherry-Pick…** | Hộp thoại "Cherry-Pick xxxxxxxx", **Commit** được chọn |
| 2 | **Cherry-Pick** | Commit mới trên `main`, cùng tiêu đề |
| 3 | Cherry-pick lại đúng commit đó | Lỗi trong hộp thoại: "already has the changes … Nothing was changed." |
| 4 | Commit khác → **Apply to Working Tree** | Changes: file của commit đó **Staged**; chưa có commit mới |
| 5 | Commit sửa cùng dòng với `main` → Cherry-Pick (Commit) | Hộp thoại đóng; lỗi conflict; thanh **đỏ** "Cherry-picking: …" với Continue (mờ) / Abort…; commit box bị khóa |
| 6 | Sửa file, `git add` trong terminal, quay lại app → **Continue** | Thanh biến mất; commit mới với tiêu đề gốc |
| 7 | Làm lại bước 5 → **Abort…** → xác nhận | Mọi thứ như trước |
| 8 | Chuột phải một merge commit → Cherry-Pick… | Có dòng "This is a merge commit: … first parent" |
| 9 | Đang ở bước 5 (chờ) → thử Merge / Rebase / Reset | Lỗi "… cherry-pick is in progress …" |
