# Bước 24a — Giải quyết conflict trong app: Take Ours / Take Theirs / Mark Resolved

Bước 24 (xử lý conflict) được chia làm hai:
- **24a (bước này):**
  - lấy nguyên file của một phía (ours / theirs);
  - đánh dấu đã giải quyết sau khi tự sửa (có kiểm tra dấu conflict còn sót).

  Từ giờ không cần `git add` trong terminal nữa.
- **24b:** mở **công cụ so sánh ngoài** (merge tool) cho file conflict, và chọn công cụ nào.

Các phần khác của bước 24 đã có từ trước:
- phát hiện đang merge / rebase / cherry-pick (21b, 22b, 23);
- danh sách file conflict (bước 6);
- Continue / Abort (21b, 22b, 23).

## Trạng thái
- `cargo test`: 191/191 pass. Thêm 5 test:
  - Take Theirs cho conflict nội dung; Take Ours cho conflict "một bên xóa, một bên sửa" (giữ bản sửa);
  - Take phía đã xóa file → file bị xóa (và được stage);
  - Mark Resolved từ chối khi còn dấu conflict, chấp nhận sau khi sửa, chấp nhận khi có `force`; sau đó commit merge được;
  - chỉ nhận file đang conflict;
  - nhận diện dấu conflict (cả ba loại phải có; dòng `=======` của tiêu đề Markdown không tính).
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-24). Đã commit.

> **Cập nhật sau bước 24b:** theo yêu cầu của bạn, nút Take Ours / Take Theirs được bỏ khỏi thanh công cụ (vẫn còn trong menu chuột phải, cho file nhị phân / file bị một bên xóa). Cách chính để giải quyết conflict giờ là double-click file → cửa sổ Conflict Solver (xem `24b-conflict-solver.md`). Thanh công cụ còn **Resolve…** và **Mark Resolved**.

## Mục tiêu
Tab **Changes** → chọn một (hoặc nhiều) file **Conflicted**. Thanh công cụ có thêm (cả trong menu chuột phải):

| Nút | Làm gì | Hỏi trước? |
|---|---|---|
| **Take Ours (main)…** | Thay cả file bằng bản của "phía mình", rồi stage | **Có**: thay đổi của phía kia và phần bạn đã sửa trong file sẽ mất |
| **Take Theirs (feature)…** | Thay cả file bằng bản của "phía kia", rồi stage | **Có** |
| **Mark Resolved** | Bạn đã tự sửa file: stage nó như hiện tại | Chỉ khi file **còn dấu** `<<<<<<<` / `=======` / `>>>>>>>`: "Mark as resolved anyway?" |

Sau khi mọi file đã giải quyết:
- **merge**: Commit (hoặc Abort Merge);
- **rebase / cherry-pick**: **Continue** trên thanh màu.

Câu hướng dẫn trên thanh màu và trong thông báo lỗi giờ chỉ tới các nút này thay vì terminal.

**Không lặp thông tin** (bạn góp ý): khi đã có thanh màu Merging / Rebasing / Cherry-picking, dòng lỗi conflict màu đỏ bên dưới bị ẩn (`shownError` trong RepoView), vì thanh màu đã nói đủ. Dòng lỗi chỉ hiện khi conflict **không** có thanh màu: stash apply/pop, cherry-pick "Apply to Working Tree".

## "Ours" và "theirs" là ai? (dễ nhầm)
App ghi **tên thật** trong ngoặc trên nút, vì nghĩa của hai chữ này đổi theo thao tác:

| Đang làm | Ours | Theirs |
|---|---|---|
| Merge `feature` vào `main` | `main` (branch hiện tại) | `feature` |
| Cherry-pick commit X | branch hiện tại | commit X |
| **Rebase** `main` lên `feature` | **`feature`** (commit đích, nơi đang replay lên) | **commit của bạn** đang được replay |
| Stash apply / pop | branch hiện tại | stash |

Rebase ngược với trực giác: git replay **commit của bạn** lên đỉnh target, nên "phía đang có" (ours) là target.

## git làm gì bên dưới

Khi một file conflict, index giữ tối đa **ba phiên bản** ("stage") của file:

| Stage | Là |
|---|---|
| 1 | tổ tiên chung |
| 2 | ours |
| 3 | theirs |

Phía nào đã **xóa** file thì không có stage của phía đó.

| Việc | Lệnh |
|---|---|
| Xem các stage của file | `git --literal-pathspecs ls-files --stage -z -- <file>` → `<mode> <id> <stage>\t<path>` |
| Take Ours / Theirs (phía đó còn file) | `git --literal-pathspecs checkout --ours\|--theirs -- <file>`, rồi `git add --all -- <file>` |
| Take phía đã xóa file | `git --literal-pathspecs rm --quiet --ignore-unmatch -- <file>` |
| Mark Resolved | `git --literal-pathspecs add --all -- <file>` |

### Ghi chú
- **Stage file = báo git "đã giải quyết"**: index chỉ còn một phiên bản (stage 0) thay vì 1/2/3. Đây là lý do từ bước 7 nút **Stage** bỏ qua file conflict: phải đi qua Mark Resolved (có kiểm tra).
- **Kiểm tra dấu conflict**: tìm dòng bắt đầu bằng `<<<<<<< `, dòng đúng bằng `=======`, và dòng bắt đầu bằng `>>>>>>> `, **cả ba** phải có. Nhờ vậy dòng `=======` dùng làm gạch dưới tiêu đề Markdown không bị nhầm.
- **Lỗi riêng `ConflictMarkers`**: frontend dựa vào `kind` này để hỏi lần hai rồi gửi lại với `force = true`, giống "NotMerged → Delete Anyway" ở bước 19b.
- **Chỉ file đang conflict** mới được xử lý (so với status trước), như discard.

## Luồng dữ liệu

```
Chọn file Conflicted → [Take Theirs (feature)…]
  └─ RepoView resolveSelected("theirs") → ask("Replace … with the version from feature?")
       └─ resolveConflicts(repoId, paths, "theirs", false) → invoke("resolve_conflicts")
            └─ Rust git::conflict::resolve → chỉ file conflict
                 └─ take_side → ls-files --stage (có stage 3?) → checkout --theirs + add  |  rm
       └─ status mới → file thành Staged → (hết conflict) thanh màu chuyển xanh, Continue sáng
[Mark Resolved] → resolve(…, "resolved", false)
  ├─ còn dấu → Err(ConflictMarkers) → ask("Mark as resolved anyway?") → resolve(…, true)
  └─ sạch → git add --all
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/conflict.rs` (mới) | `resolve`, `take_side`, `has_stage`, `has_conflict_markers` + 5 test |
| `src-tauri/src/models.rs`, `error.rs` | `ConflictChoice`; `ErrorKind::ConflictMarkers` |
| `src-tauri/src/commands/stage.rs`, `lib.rs` | `resolve_conflicts` |
| `src-tauri/src/git/{merge,rebase,cherry_pick,pull,stash,discard}.rs` | thông báo conflict chỉ tới các nút mới, không còn "terminal" / "later step" |
| `src/types/status.ts`, `src/types/error.ts`, `src/api/stage.ts` | kiểu + wrapper |
| `src/views/RepoView.vue` | `conflictedPaths`, `sides` (tên thật của ours/theirs), `resolveSelected`, nút + menu, câu trên thanh màu |

## Khái niệm mới

### Rust
- **`|=` với `bool`**: `start |= line.starts_with(...)`: một khi đã `true` thì giữ `true`. Gọn hơn `if … { start = true }`.
- **`split(' ').nth(2)`**: lấy phần tử thứ 3 của iterator (đếm từ 0) dưới dạng `Option`.
- **`String::from_utf8_lossy`** (đã gặp ở runner) dùng lại để đọc file có thể không phải UTF-8 mà không lỗi.

### Vue / TS
- **`computed` trả về object** (`sides` → `{ ours, theirs }`): một chỗ quyết định tên hai phía theo trạng thái (rebase / cherry-pick / merge), cả nút, menu, câu hỏi đều dùng lại.
- **Regex lấy phần trong dấu nháy** `/'([^']+)'/.exec(message)?.[1]`: lấy `feature` trong `Merge branch 'feature' into main`; `?.[1]` cho `undefined` nếu không khớp.
- **Hàm trả về hàm** `const resolve = (force) => () => resolveConflicts(…)`: tạo sẵn "việc cần chạy" cho `run()`, với `force` khác nhau ở lần một và lần hai.
- **`"Separator" as const` trong spread có điều kiện**: TypeScript cần biết đúng chuỗi `"Separator"` cả khi phần tử nằm trong mảng tạo bằng `? […] : []`.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Merge hai branch cùng sửa một dòng (Create Merge-Commit) | Thanh đỏ "Merging: … fix them (or take ours / theirs), then Mark Resolved." |
| 2 | Chọn file Conflicted | Thanh công cụ có **Take Ours (main)…**, **Take Theirs (feature)…**, **Mark Resolved** |
| 3 | **Take Theirs** → Replace | File thành bản của `feature`, chuyển sang Staged; thanh chuyển xanh |
| 4 | Commit (Commit Merge) | Merge xong |
| 5 | Làm lại conflict → **Mark Resolved** ngay (chưa sửa) | Hỏi "still contain conflict markers … Mark as resolved anyway?" → **Keep Editing**: không có gì đổi |
| 6 | Sửa file trong editor (bỏ các dấu) → **Mark Resolved** | Không hỏi; file thành Staged |
| 7 | Rebase gây conflict | Nút ghi **Take Ours (xxxxxxxx (rebasing onto))** / **Take Theirs (your commit …)** |
| 8 | Take Ours → Continue | Rebase đi tiếp / xong |
| 9 | Cherry-pick gây conflict → Take Theirs (commit) → Continue | Commit mới với nội dung của commit được pick |
| 10 | Chuột phải file Conflicted | Menu có ba mục trên (sau Discard…) |
