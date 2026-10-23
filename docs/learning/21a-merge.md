# Bước 21a — Merge (giống hộp thoại Merge của SmartGit)

Bước 21 được chia làm hai:
- **21a (bước này):** hộp thoại Merge với ba lựa chọn như SmartGit, mở từ menu branch và từ History.
- **21b:** trạng thái "đang merge" sau **Merge to Working Tree** (hoặc sau conflict):
  - dòng báo "Merging feature into main" kèm nút **Abort** (có xác nhận);
  - message commit được điền sẵn;
  - khi commit, chọn **merge commit** hay **commit thường**, như SmartGit.

## Trạng thái
- `cargo test`: 149/149 pass. Thêm 8 test:
  - xem trước: đã chứa hết / fast-forward được / phải merge;
  - merge commit có message dễ đọc ("Merge branch 'feature' into main");
  - vẫn tạo merge commit khi fast-forward được;
  - fast-forward chỉ di chuyển branch, từ chối khi branch có commit riêng;
  - merge vào working tree dừng trước commit; commit sau đó tạo merge commit (kèm tag trong message);
  - conflict được báo, không cho merge chồng;
  - không đè thay đổi chưa commit / file untracked (báo tên file);
  - chỉ nhận ref đang tồn tại hoặc commit id (`--all`, `HEAD`, `main`, `feature~1` bị từ chối).
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-23). Đã commit. Sửa thêm sau khi thử: thiếu quyền `dialog:allow-message`; store `dialogs` dùng `acceptHMRUpdate` để hot reload thay được cả store; mặc định là Create Merge-Commit.

## Ba lựa chọn (giống SmartGit)
Giả sử đang ở `main`, merge `feature` vào:

```
Trước:                  Fast-Forward:           Create Merge-Commit:     Merge to Working Tree:
                                                *  Merge … ← main        (chưa commit; thay đổi
  * f2  ← feature       * f2 ← feature, main    |\                         của f1, f2 đã stage)
  * f1                  * f1                    | * f2 ← feature         Commit → giống cột bên trái
  * base ← main         * base                  | * f1
                                                |/
                                                * base
```

| Lựa chọn | Làm gì | Lệnh |
|---|---|---|
| **Fast-Forward** | Chỉ dời `main` tới `feature`, không tạo commit mới. Chỉ được khi `main` **không có commit riêng** | `git merge --ff-only --end-of-options <nguồn>` |
| **Create Merge-Commit** | Luôn tạo merge commit (2 cha), kể cả khi fast-forward được, nên lịch sử thể hiện rõ "đã merge branch này" | `git merge --no-ff --no-edit --message=… --end-of-options <nguồn>` |
| **Merge to Working Tree** | Merge nhưng **dừng trước khi commit**: thay đổi đã stage, để bạn xem/sửa rồi tự commit (tạo merge commit) | `git merge --no-ff --no-commit --message=… --end-of-options <nguồn>` |

- Hộp thoại luôn chọn sẵn **Create Merge-Commit** (bạn chọn), kể cả khi fast-forward được.
- Nếu `main` đã chứa mọi thứ của `feature` → không mở hộp thoại, chỉ báo "Nothing to Merge".

## Mở từ đâu
- **Menu `⎇ main ▾`** → **Merge into "main" ▸**: các branch local khác và các remote branch.
- **History → chuột phải một commit**:
  - **Merge \<x\> into Current Branch…** cho mỗi branch/tag trên commit đó;
  - **Merge Commit 1234abcd into Current Branch…** nếu commit không có nhãn nào.

## Lệnh git để xem trước

| Việc | Lệnh |
|---|---|
| Commit của HEAD / của nguồn | `git rev-parse --verify --quiet --end-of-options <x>^{commit}` |
| Commit chung mới nhất | `git merge-base HEAD <nguồn>` |

- `base == nguồn` → **đã chứa hết**, không có gì để merge.
- `base == HEAD` → branch không có commit riêng → **fast-forward được**.
- Không có commit chung (hai lịch sử không liên quan) → báo lỗi.
- `^{commit}`: với tag, lấy commit mà tag trỏ tới (không lấy object tag).

## Ghi chú
- **Message do app tự viết**: app truyền **tên ref đầy đủ** (`refs/heads/feature`) để không bao giờ lẫn branch với tag trùng tên. Nếu để git tự viết message, nó sẽ ghi "Merge branch 'refs/heads/feature'". Vì vậy app tự viết, giống cách git hay dùng:
  - `Merge branch 'feature' into main`
  - `Merge remote-tracking branch 'origin/feature' into main`
  - `Merge tag 'v1.0' into main`
  - `Merge commit '1234abcd' into main`
- **Không merge khi**:
  - HEAD detached (phải đang ở một branch);
  - có file conflict;
  - đang có một merge dở (`MERGE_HEAD` tồn tại).
- **Lỗi được giải thích**:
  - **Conflict** → hộp thoại đóng, tab Changes hiện file **Conflicted**, kèm thông báo cách xử lý. Bước 24 sẽ làm việc này trong app.
  - **Sẽ đè thay đổi chưa commit** hoặc **sẽ đè file untracked** → báo tên file ngay trong hộp thoại; git không đổi gì.
  - **Fast-forward không được** → gợi ý chọn Create Merge-Commit.
  - **Thiếu tên/email** → như bước 9.
- **Chỉ nhận ref đang tồn tại hoặc commit id** (như bước 19a), sau đó còn có `--end-of-options`.
- **Sau Merge to Working Tree** (cho tới bước 21b): bạn gõ message rồi **Commit** như bình thường; git tự tạo merge commit (2 cha) vì có `MERGE_HEAD`. Muốn bỏ merge thì tạm chạy `git merge --abort` trong terminal.

## Luồng dữ liệu

```
⎇ main ▾ → Merge into "main" ▸ feature…  (hoặc History → chuột phải → Merge feature into Current Branch…)
  └─ branchMenus askMerge → getMergePreview(repoId, "refs/heads/feature") → invoke("get_merge_preview")
       └─ Rust git::merge::preview → resolve_source → rev-parse, merge-base → MergePreview
  ├─ upToDate → message("Nothing to Merge")
  └─ dialogs.askMerge({ preview, submit }) → <MergeDialog> (3 radio)
       └─ submit(mode) → mergeIntoCurrent(repoId, source, mode) → invoke("merge_into_current")
            └─ Rust git::merge::merge → kiểm tra → git merge … → read_status
       ├─ OK → callbacks.onStatus → RepoView applyStatus → mọi thứ tải lại
       ├─ conflict → getStatus → onStatus, rồi onError (hộp thoại đóng)
       └─ lỗi khác → hiện trong hộp thoại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/merge.rs` (mới) | `preview`, `merge`, `merge_in_progress`, `resolve_source`, `explain_merge_error` + 8 test |
| `src-tauri/src/commands/merge.rs` (mới), `lib.rs` | `get_merge_preview`, `merge_into_current` |
| `src-tauri/src/models.rs` | `MergeMode`, `MergePreview` |
| `src/types/merge.ts`, `src/api/merge.ts` (mới) | kiểu + wrapper |
| `src/components/MergeDialog.vue` (mới) | hộp thoại 3 lựa chọn |
| `src/stores/dialogs.ts`, `src/App.vue` | `mergeRequest`, gắn `MergeDialog` |
| `src/components/branchMenus.ts` | submenu **Merge into**, mục Merge trong menu commit, `askMerge` |

## Khái niệm mới

### Rust
- **`impl` cho struct riêng của module** (`Source::describe`): struct `Source` không `pub`, chỉ dùng trong `merge.rs`. Giống một private nested class trong Java.
- **`args.extend([...])`**: thêm nhiều phần tử một lúc từ một mảng cố định, gọn hơn ba lần `push`.
- **`chars().take(8).collect()`**: lấy 8 ký tự đầu thành `String`. Cắt theo ký tự (không cắt theo byte như `&s[..8]`, vốn có thể panic với chữ có dấu).
- **`match` trên `Option<enum>`**: `Some(RefKind::Branch) => …`, `_ => …`. Mẫu lồng nhau: khớp cả "có giá trị" lẫn "giá trị nào" trong một nhánh.

### Vue / TS
- **Radio bị vô hiệu theo điều kiện** (`:disabled`), kèm lý do hiện ngay bên dưới, thay vì giấu lựa chọn đi. Người dùng hiểu vì sao không chọn được.
- **`switch` trên union type** (`fullNameOf`): TypeScript biết `r.kind` chỉ có thể là vài giá trị cố định.
- **Lỗi khác nhau → cách hiện khác nhau**: lỗi "chưa đổi gì" ở lại trong hộp thoại để sửa; conflict (đã đổi file) đóng hộp thoại và cập nhật màn hình chính.

## Cách thử

App dev đang chạy và đã tự build lại. Bấm ⌘R trước khi thử (có hộp thoại mới).

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `branchy`, ở `main` → `⎇ main ▾` → **Merge into "main" ▸** | Có các branch khác (và remote nếu có) |
| 2 | Chọn một branch đã merge vào `main` rồi | "Nothing to Merge" |
| 3 | Tạo branch `ff-test` ở commit cũ của `main` (History → Create Branch Here, tick Check out) → **Merge into "ff-test" ▸ main…** | Hộp thoại chọn sẵn **Create Merge-Commit**, Fast-Forward chọn được → chọn **Fast-Forward** → Merge → `ff-test` trỏ tới cùng commit với `main`, không có commit mới |
| 4 | Về `main`, tạo commit mới; trên một branch khác cũng tạo commit → merge branch đó vào `main` | **Fast-Forward** bị mờ với lý do; chọn **Create Merge-Commit** → History có commit "Merge branch '…' into main" với 2 cha |
| 5 | Làm như bước 4 nhưng chọn **Merge to Working Tree** | Tab Changes: các file của branch kia đã **Staged**; chưa có commit mới. Gõ message → Commit → History có merge commit 2 cha |
| 6 | Hai branch cùng sửa một dòng → merge | Hộp thoại đóng; lỗi "… have conflicts …"; file **Conflicted** trong Changes. Dọn: `git merge --abort` trong terminal |
| 7 | Sửa dở một file mà branch kia cũng sửa → merge | Lỗi trong hộp thoại: "would overwrite your uncommitted changes to: …"; file của bạn còn nguyên |
| 8 | History → chuột phải commit có tag → **Merge v… into Current Branch…** | Hộp thoại "Merge "v…" into "main"" |
