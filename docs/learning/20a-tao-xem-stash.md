# Bước 20a — Tạo stash, xem danh sách và nội dung

Bước 20 của CLAUDE.md (stash: tạo, danh sách, apply, pop, drop) được chia làm hai:
- **20a (bước này):** tạo stash (message, tùy chọn gồm file untracked); tab **Stashes** liệt kê stash và nội dung của từng cái.
- **20b:** apply, pop, drop (drop có hộp thoại xác nhận), xử lý khi apply bị conflict.

## Trạng thái
- `cargo test`: 136/136 pass. Thêm 7 test:
  - stash thay đổi đã stage + chưa stage, kèm message bắt đầu bằng `--`; file untracked ở lại;
  - stash có file untracked: commit cha thứ 3 chứa đúng file đó;
  - danh sách mới nhất trước;
  - từ chối khi không có gì để stash (gợi ý tick "Include untracked files" nếu chỉ có file untracked);
  - từ chối khi repo chưa có commit;
  - danh sách rỗng khi chưa có stash;
  - parse output `stash list -z`.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-22). Đã commit.

## Mục tiêu
- Nút **Stash…** trên thanh công cụ (tab Changes và Stashes) mở hộp thoại:
  - ô **message** (không bắt buộc; để trống thì git tự ghi `WIP on main: <commit>`);
  - checkbox **Include untracked files** (mặc định tắt, như git). File bị `.gitignore` không bao giờ được đưa vào.
  - Lỗi (không có gì để stash, đang conflict…) hiện ngay trong hộp thoại.
- Tab mới **Stashes**:
  - phía trên: danh sách `stash@{0}`, `stash@{1}`…, message, nhãn **+ untracked**, thời gian;
  - phía dưới: nội dung stash đang chọn, dùng lại `CommitDetails` của bước 18 (danh sách file + diff);
  - nếu stash có file untracked: hai nút **Changes** / **Untracked Files** để xem từng phần.

## Stash được git lưu thế nào
Một stash là **một commit đặc biệt**, chỉ được trỏ tới từ reflog của `refs/stash` (nên không hiện trong lịch sử branch):

```
        W  ← stash@{0}  (cây file = working tree lúc stash)
      / | \
     H  I  U
     │  │  └─ chỉ có khi --include-untracked: commit gốc chứa các file untracked
     │  └──── trạng thái index (phần đã stage)
     └─────── commit đang checkout lúc stash (HEAD)
```

Vì vậy:
- **So W với H** (cha thứ nhất) ra mọi thay đổi của file đã theo dõi, cả phần đã stage và chưa stage. Đây đúng là việc `CommitDetails` vẫn làm với merge commit ("so với cha thứ nhất").
- **U** là commit không có cha, nên mọi file trong đó hiện là **A** (added), so với "trống".

## Lệnh git

| Việc | Lệnh |
|---|---|
| Tạo stash | `git stash push --quiet [--include-untracked] [--message=<message>]` |
| Danh sách | `git stash list -z --no-show-signature --no-color --format=%H%x1f%P%x1f%ct%x1f%gs` |
| Nội dung | như bước 18: `git show …`, `git diff-tree …`, `git cat-file blob …` với id của W hoặc U |

### Ghi chú
- **Kiểm tra trước khi stash**, vì khi không có gì để stash, `git stash` chỉ in một dòng rồi **thoát với mã 0** (thành công), và người dùng sẽ không biết là chưa có gì xảy ra. App đọc status trước:
  - có file conflict → từ chối (git cũng không stash được khi đang conflict);
  - không có thay đổi → báo "no local changes"; chỉ có file untracked → gợi ý tick checkbox;
  - repo chưa có commit → báo cần commit đầu tiên (stash cần H).
- **`--message=<text>` viết liền một tham số**: dù message bắt đầu bằng `-`, git không thể hiểu nhầm nó là option (có test với `--my work`).
- **`%gs`** là "reflog subject", tức dòng `On main: <message>` mà `git stash list` vẫn in. **`%P`** cho danh sách cha: có 3 cha nghĩa là stash có file untracked.
- **`stash list` thay vì `log -g refs/stash`**: khi chưa có stash nào, `stash list` in ra rỗng, còn `log` thì báo lỗi vì `refs/stash` chưa tồn tại.
- **Stash được nhận diện bằng id**, không phải số thứ tự: `stash@{0}` luôn là cái mới nhất, nên tạo thêm một stash là mọi số thứ tự dịch đi 1. Tab Stashes giữ lựa chọn theo id. Bước 20b dựa vào điều này để không bao giờ drop nhầm stash.

## Luồng dữ liệu

```
Nút "Stash…" → RepoView askStash → NameDialog (message, ☐ Include untracked files)
  └─ submit → createStash(repoId, message, includeUntracked) → invoke("create_stash")
       └─ Rust git::stash::create → read_status (kiểm tra) → git stash push …
       └─ read_status → RepoStatus mới
  └─ applyStatus → statusVersion++ → bảng file trống đi; StashView tải lại
Tab Stashes → StashView watch(version) → listStashes → invoke("list_stashes")
  └─ Rust git::stash::list → git stash list -z … → parse_list → Vec<StashEntry>
  └─ chọn stash → shownCommit = W (hoặc U) → <CommitDetails :commit-id hide-parents>
       └─ (như bước 18) getCommitDetails / getCommitFileDiff
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/stash.rs` (mới) | `create`, `list`, `parse_list` + 7 test |
| `src-tauri/src/commands/stash.rs` (mới) | `create_stash`, `list_stashes` |
| `src-tauri/src/models.rs` | `StashEntry` |
| `src/types/stash.ts`, `src/api/stash.ts` (mới) | kiểu + wrapper |
| `src/views/StashView.vue` (mới) | tab Stashes: danh sách + nội dung |
| `src/components/CommitDetails.vue` | prop `hideParents`: không hiện link tới cha (H, I là commit nội bộ của stash) |
| `src/stores/dialogs.ts`, `src/components/NameDialog.vue` | `placeholder`, `allowEmpty` (message stash được để trống) |
| `src/views/RepoView.vue` | tab thứ ba, nút **Stash…**, `askStash` |

## Khái niệm mới

### Rust
- **`enumerate()`**: đánh số phần tử khi duyệt, cho ra cặp `(index, phần tử)`. Giống vòng `for (int i = 0; …)` nhưng vẫn là iterator. Ở đây số thứ tự chính là `n` trong `stash@{n}`.
- **`collect()` thành `Result<Vec<_>, _>`**: một iterator gồm các `Result` được gom thành **một** `Result`: tất cả `Ok` thì ra `Ok(Vec)`, gặp `Err` đầu tiên thì dừng và trả `Err` đó. Trong Java phải tự viết vòng lặp + `throw`.
- **`Vec<String>` rồi đổi sang `Vec<&str>`**: `format!("--message={message}")` tạo một `String` mới (sở hữu). Hàm `run_git` cần `&[&str]` (mượn), nên tạo `Vec<String>` trước, rồi `iter().map(|a| a.as_str())` mượn từng phần tử. Vec `String` phải sống lâu hơn Vec `&str`, vì bên này mượn bên kia.
- **`parents.get(2)`**: lấy phần tử thứ 3 dưới dạng `Option` (không panic nếu không có), khác `parents[2]` (panic khi thiếu, giống `IndexOutOfBoundsException`).

### Vue
- **Prop boolean không cần giá trị**: `<CommitDetails hide-parents />`. Viết tên prop không kèm `="…"` nghĩa là `true`, không viết là `false`. Giống thuộc tính `disabled` của HTML.
- **Dùng lại component bằng cách đổi dữ liệu đầu vào**: `StashView` chỉ đổi `commitId` (W hoặc U) cho `CommitDetails`, không viết lại phần danh sách file + diff.
- **Tránh `{{ }}` lồng dấu ngoặc nhọn**: `stash@{0}` có dấu `{ }`, dễ lẫn với `{{ }}` của template, nên được tạo trong một hàm TS (`stashName`).

## Cách thử

App dev đang chạy và đã tự build lại. Nếu giao diện trông lạ, bấm ⌘R (vì `CommitDetails` có prop mới).

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo sạch (không có thay đổi) | Nút **Stash…** bị mờ |
| 2 | Sửa một file, tạo một file mới (untracked) → **Stash…** → message `thử 1`, **không** tick → Stash | Hộp thoại đóng; file đã sửa biến mất khỏi Changes, file untracked **vẫn còn** |
| 3 | Tab **Stashes** | `stash@{0}  On main: thử 1`; phía dưới: file đã sửa + diff |
| 4 | Lại **Stash…** (chỉ còn file untracked), không tick → Stash | Lỗi trong hộp thoại: "Only untracked files changed. Tick …" |
| 5 | Tick **Include untracked files** → Stash (để trống message) | Changes trống. Stashes: cái mới là `stash@{0}  WIP on main: …` với nhãn **+ untracked**; `thử 1` thành `stash@{1}` và **vẫn được chọn** |
| 6 | Chọn stash mới → bấm **Untracked Files** | Danh sách có file untracked, hiện là **A**, diff bên trái trống |
| 7 | Stage một file, sửa một file khác → Stash | Stash chứa cả hai file (phần đã stage và chưa stage) |
| 8 | Tab **History** | Không thấy commit nào của stash trong graph |
| 9 | Kiểm tra bằng terminal: `git stash list` | Giống danh sách trong app |

Apply / pop / drop sẽ có ở bước 20b. Trong lúc chờ, muốn lấy lại thay đổi thì chạy `git stash pop` trong terminal.
