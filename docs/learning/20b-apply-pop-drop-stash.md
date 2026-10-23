# Bước 20b — Apply, pop, drop stash

## Trạng thái
- `cargo test`: 141/141 pass. Thêm 5 test:
  - apply giữ stash, pop xóa stash;
  - tìm đúng stash theo id dù số thứ tự đã dịch (có stash mới hơn); drop lần hai → `NotFound`;
  - pop bị conflict → stash **được giữ**, file ở trạng thái conflict; không cho apply chồng lên;
  - không đè thay đổi chưa commit (báo tên file, file của bạn còn nguyên);
  - không đè file untracked đã có (báo tên file, stash còn nguyên).
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-23). Đã commit.

## Mục tiêu
Trong tab **Stashes**:
- Ba nút trên danh sách: **Apply**, **Pop**, **Drop…**, cho stash đang chọn.
- Chuột phải một stash: cùng ba lựa chọn.
- **Apply**: đưa thay đổi của stash trở lại working tree, **giữ** stash (dùng lại được, ví dụ áp vào branch khác).
- **Pop**: apply, rồi xóa stash. Chỉ xóa khi apply **không** bị conflict.
- **Drop…**: xóa stash. Thay đổi trong đó mất, nên **luôn hỏi trước** (CLAUDE.md).
- Trong lúc chạy, các nút khác của repo (stage, commit, pull…) bị khóa, như khi pull.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Apply | `git stash apply --quiet stash@{n}` |
| Pop | `git stash pop --quiet stash@{n}` |
| Drop | `git stash drop --quiet stash@{n}` |

### Ghi chú
- **Frontend gửi id, không gửi số thứ tự.** Ngay trước khi chạy git, Rust đọc lại danh sách, tìm stash có id đó, rồi mới tạo `stash@{n}`. Nếu bạn (hoặc terminal) vừa tạo thêm một stash, số thứ tự đã dịch đi nhưng app vẫn trúng đúng stash. Nếu stash không còn → `NotFound`, danh sách được tải lại.
- **`stash@{n}` được tạo từ một con số**, nên không thể bị hiểu nhầm là option.
- **Ba cách apply thất bại**, mỗi cách được đổi thành một thông báo nói rõ phải làm gì:
  1. **Conflict**: git đã áp phần lớn thay đổi, nhưng một số file bị conflict (`UU`). Giống pull, app dựa vào status (không dựa vào stdout) để biết. Với **pop**, git **không** xóa stash trong trường hợp này. Thông báo nói rõ là stash vẫn còn. Xử lý conflict trong app là bước 24; trước đó bạn sửa file, rồi stage.
  2. **Thay đổi chưa commit sẽ bị đè** ("would be overwritten"): git không đụng vào gì. App lấy tên file từ các dòng bắt đầu bằng tab trong lỗi của git.
  3. **File untracked trong stash đã có ở working tree** ("already exists, no checkout"): git cũng dừng mà không đổi gì.
- **Đang có conflict** thì app từ chối apply (không chồng thêm thay đổi lên file đang conflict).
- **Không dùng `--index`**: git mặc định không khôi phục phần "đã stage" của stash. Mọi thay đổi quay về dạng chưa stage. `--index` khôi phục được, nhưng hay thất bại hơn khi index đã thay đổi. Bạn chỉ cần stage lại.

## Luồng dữ liệu

```
Tab Stashes → nút Pop (hoặc chuột phải → Pop)
  └─ StashView perform(stash, "pop") → locked = true (RepoView khóa các nút khác)
       └─ runStashAction(repoId, stash.id, "pop") → invoke("run_stash_action")
            └─ Rust git::stash::run → list() → tìm theo id → stash@{n}
                 └─ read_status (không có conflict?) → git stash pop --quiet stash@{n}
                      ├─ OK → read_status → RepoStatus
                      └─ lỗi → explain_apply_error → Conflict / InvalidInput
       ├─ OK → emit("statusChanged") → RepoView applyStatus → statusVersion++ → danh sách stash tải lại
       └─ lỗi → locked = false → emit("failed") → RepoView onStashFailed → refresh() rồi hiện lỗi
Drop… → ask("Drop stash@{n}? …") → (đồng ý) → như trên với "drop"
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/stash.rs` | `run` (apply/pop/drop theo id), `explain_apply_error` + 5 test |
| `src-tauri/src/models.rs` | enum `StashAction` (Deserialize: frontend gửi `"apply"`…) |
| `src-tauri/src/commands/stash.rs`, `lib.rs` | `run_stash_action` |
| `src/types/stash.ts`, `src/api/stash.ts` | `StashAction`, `runStashAction` |
| `src/views/StashView.vue` | nút Apply/Pop/Drop…, menu chuột phải, hỏi trước khi drop, `v-model:locked` |
| `src/views/RepoView.vue` | `stashLocked` vào `working`; `onStashFailed` |

## Khái niệm mới

### Rust
- **`find` + `ok_or_else`**: `list(repo)?.into_iter().find(|s| s.id == id).ok_or_else(|| lỗi)?`. `find` trả `Option` (có thể không thấy), `ok_or_else` đổi `None` thành `Err(lỗi)`, rồi `?` trả lỗi ra ngoài. Giống `stream().filter(...).findFirst().orElseThrow(...)` trong Java.
- **`strip_prefix` / `strip_suffix` trong `filter_map`**: trả `Some(phần còn lại)` nếu chuỗi bắt đầu/kết thúc đúng như vậy, ngược lại `None`. Đặt vào `filter_map`, ta vừa lọc dòng vừa cắt phần thừa trong một bước.
- **Một enum thay cho ba command**: `StashAction { Apply, Pop, Drop }` đi từ TS (`"pop"`) sang Rust nhờ `Deserialize`. Ba thao tác dùng chung việc tìm stash, nên gộp vào một hàm `run` + `if`/`match` gọn hơn ba hàm gần giống nhau.

### Vue
- **Emit sau `finally`**: `locked` phải được trả về `false` **trước** khi báo lỗi lên RepoView, vì RepoView cần tải lại status, mà việc đó bị chặn khi còn khóa. Vì vậy lỗi được giữ trong biến `failure` và chỉ emit sau khối `try/finally`.
- **`@contextmenu.prevent`** trên từng dòng: chặn menu mặc định của webview, mở menu native của mình.

## Cách thử

App dev đang chạy và đã tự build lại. Bấm ⌘R nếu giao diện trông lạ.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Tab Stashes → chọn một stash → **Apply** | Thay đổi quay lại tab Changes (dạng chưa stage); stash **vẫn còn** |
| 2 | Bấm **Apply** lần nữa khi các thay đổi vẫn còn đó | Lỗi: "would overwrite your uncommitted changes to: …"; file không đổi |
| 3 | Bỏ các thay đổi (hoặc stash lại) → chọn stash cũ → **Pop** | Thay đổi quay lại; stash biến mất khỏi danh sách |
| 4 | **Drop…** → **Cancel** | Không có gì thay đổi |
| 5 | **Drop…** → **Drop** | Stash biến mất |
| 6 | Conflict: sửa dòng 1 của `a.txt` → Stash. Sửa dòng 1 khác đi → commit. Chọn stash → **Pop** | Lỗi "… have conflicts … The stash was kept"; tab Changes có file **Conflicted**; stash vẫn trong danh sách |
| 7 | Khi đang có file conflict ở bước 6 → **Apply** | Lỗi "Resolve them before applying a stash." |
| 8 | Stash có **+ untracked** → tạo lại một file cùng tên → **Pop** | Lỗi "untracked files that already exist here: …"; stash còn nguyên |
| 9 | Chuột phải một stash | Menu: Apply, Pop (Apply and Drop), Drop… |

Để dọn conflict ở bước 6 trong lúc chưa có bước 24: sửa file, rồi **Stage** (app đã chặn stage file conflict, nên tạm dùng `git add a.txt` trong terminal), hoặc `git checkout HEAD -- a.txt` để bỏ thay đổi.
