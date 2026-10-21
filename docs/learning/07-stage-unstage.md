# Bước 7 — Stage / unstage

## Trạng thái
- `cargo test`: 52/52 pass (thêm 7 test: stage file mới/sửa/xóa, unstage giữ nguyên nội dung file, unstage file đổi tên, repo chưa có commit, tên file có `*`, 250 file qua nhiều lô, lỗi đầu vào). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21).

## Mục tiêu
- Chọn file trong bảng: click / ⌘-click (Ctrl-click trên Ubuntu) / Shift-click / ⌘A.
- Nút **Stage**, **Unstage** trên thanh công cụ + menu chuột phải trên dòng.
- Nút chỉ sáng khi có file phù hợp trong vùng chọn (ví dụ chọn toàn file chưa stage thì Unstage bị mờ).
- Không làm thay đổi nội dung file trên ổ đĩa — chỉ di chuyển thay đổi giữa working tree và index.

## Lệnh git

| Thao tác | Lệnh | Ghi chú |
|---|---|---|
| Stage | `git --literal-pathspecs add --all -- <paths>` | `--all` để stage cả việc **xóa** file |
| Unstage | `git --literal-pathspecs restore --staged -- <paths>` | Lấy lại bản trong `HEAD` cho index; working tree giữ nguyên |
| Unstage khi repo **chưa có commit nào** | `git --literal-pathspecs rm --cached -r --quiet --ignore-unmatch -- <paths>` | Chưa có `HEAD` để "restore" → bỏ file khỏi index (`--cached`: không xóa trên ổ đĩa) |

Kiểm tra repo đã có commit chưa: `git rev-parse --verify --quiet HEAD` (thành công = có).

### `--literal-pathspecs` — vì sao cần
Tham số đường dẫn của git là **pathspec**: có thể chứa mẫu như `*.txt`. Nếu có file tên thật là `a*b.txt`, lệnh `git add -- a*b.txt` sẽ stage **cả** `aXb.txt`, `a123b.txt`… `--literal-pathspecs` bảo git: mọi đường dẫn là **tên file chính xác**. Test `treats_paths_literally_not_as_patterns` kiểm tra đúng trường hợp này.

### Chia lô 100 file
Hệ điều hành giới hạn độ dài một dòng lệnh (Windows chỉ khoảng 32.000 ký tự). Stage 5.000 file trong một lệnh có thể vượt giới hạn → chia thành nhiều lệnh, mỗi lệnh tối đa 100 đường dẫn (`paths.chunks(100)`).

### File đổi tên
Một lần đổi tên đã stage gồm 2 phần trong index: thêm `new.txt`, xóa `old.txt`. Unstage phải đưa **cả hai** đường dẫn, nếu không index còn lại một nửa. Frontend tự thêm `origPath` khi unstage dòng đổi tên. Sau khi unstage: `old.txt` thành *Missing*, `new.txt` thành *Untracked* — đúng như git làm.

### File conflict
Tạm thời **không cho stage** file conflict: với git, `git add` lên file conflict nghĩa là "tôi đã giải quyết xong". Làm việc đó đúng cách thuộc bước 15.

## File đã tạo / sửa

**Rust**
| File | Việc |
|---|---|
| `git/stage.rs` | **Mới.** `stage`, `unstage`, chia lô, `--literal-pathspecs`. 7 test |
| `commands/stage.rs` | **Mới.** `stage_files`, `unstage_files` — chạy nền, trả về status mới |
| `git/mod.rs`, `commands/mod.rs`, `lib.rs` | Đăng ký |

**Frontend**
| File | Việc |
|---|---|
| `api/stage.ts` | **Mới.** `stageFiles`, `unstageFiles` |
| `components/FileTable.vue` | Chọn dòng (click, ⌘/Ctrl, Shift, ⌘A), tô màu dòng được chọn, phát sự kiện `rowMenu` khi chuột phải |
| `views/RepoView.vue` | Thanh công cụ Stage / Unstage / Refresh, menu chuột phải, tính danh sách đường dẫn cần stage/unstage |

## Luồng dữ liệu
```
FileTable: click dòng → selected = ["src/a.ts", ...]   (v-model:selected ↔ RepoView.selectedPaths)
RepoView:
  selectedRows   = rows có path trong selectedPaths
  pathsToStage   = selectedRows có hasUnstaged, bỏ conflict          → nút Stage sáng/mờ
  pathsToUnstage = selectedRows có hasStaged (+ origPath nếu đổi tên) → nút Unstage sáng/mờ
Bấm Stage → run(() => stageFiles(repo.id, pathsToStage))
  invoke("stage_files", { repoId, paths })
    commands::stage::stage_files → repo_path(id) → run_blocking:
      git::stage::stage(repo, paths)      ← các lô `git add`
      git::status::read_status(repo)      ← đọc lại status
  ← RepoStatus mới
status.value = ...  → bảng vẽ lại; file đã stage giờ ghi "Staged"
                    → FileTable bỏ khỏi vùng chọn những file không còn trong bảng
```
`run()` chỉ cho **một** thao tác chạy tại một thời điểm (biến `busy`); refresh do focus cửa sổ trong lúc đang stage sẽ bị bỏ qua.

## Khái niệm Rust mới

### `chunks`
`paths.chunks(100)` chia slice thành các phần liên tiếp, mỗi phần tối đa 100 phần tử, **không sao chép** (mỗi phần là `&[String]` nhìn vào mảng gốc). ≈ `Lists.partition` của Guava.

### Slice `&[T]` và `&[&str]`
- `paths: &[String]` — mượn một dãy `String` (có thể là cả `Vec` hoặc một phần của nó). Hàm nhận slice thay vì `&Vec<String>` để dùng được với cả hai.
- `let command: &[&str] = if ... { &["restore", ...] } else { &[...] };` — hai nhánh `if` trả về mảng **độ dài khác nhau**; ghi rõ kiểu slice `&[&str]` để chúng cùng kiểu.

### Lifetime trên hàm có nhiều tham chiếu
```rust
fn with_paths<'a>(command: &[&'a str], paths: &'a [String]) -> Vec<&'a str>
```
Vector trả về chứa các `&str` mượn từ **cả** `command` lẫn `paths` → compiler cần biết chúng cùng sống ít nhất `'a`. Không viết thì compiler báo lỗi và gợi ý — đây là trường hợp phổ biến nhất phải viết lifetime.

### `extend_from_slice` / `extend`
Thêm nhiều phần tử vào `Vec` ≈ `addAll`. `extend(paths.iter().map(|p| p.as_str()))` thêm từ một iterator.

### Cú pháp cập nhật struct `..`
```rust
RepoStatus { staged: status.staged.clone(), ..RepoStatus::default() }
```
"Lấy `staged` như này, các trường còn lại lấy từ `RepoStatus::default()`". Trong test dùng để khẳng định: chỉ có staged, các danh sách khác rỗng.

## Khái niệm Vue mới

### `defineModel` — `v-model` cho component tự viết
```ts
// FileTable.vue
const selected = defineModel<string[]>("selected", { required: true });
selected.value = [row.path];          // ghi → cha nhận giá trị mới
```
```vue
<!-- RepoView.vue -->
<FileTable v-model:selected="selectedPaths" ... />
```
Ràng buộc **hai chiều** giữa biến của cha và component con — như `v-model` trên `<input>` (bước 1), nhưng cho component của mình. Ở bên dưới Vue biến nó thành prop `selected` + sự kiện `update:selected`. Cha giữ dữ liệu (để tính nút nào sáng), con lo việc chọn bằng chuột.

### `defineEmits` — component con báo lên cha
```ts
const emit = defineEmits<{ rowMenu: [] }>();
emit("rowMenu");
```
```vue
<FileTable @row-menu="showRowMenu" />
```
Props đi **xuống** (cha → con), emit đi **lên** (con → cha). ≈ listener/callback: con không biết cha sẽ làm gì, chỉ báo "có người chuột phải". Tên `rowMenu` trong script thành `@row-menu` trong template (quy ước kebab-case của HTML).

### Phím tắt trong template
`@keydown.meta.a.prevent="selectAll"` — chỉ khi nhấn ⌘ (meta) + A; `.ctrl.a` cho Ubuntu/Windows. `tabindex="0"` để bảng nhận được focus bàn phím (mặc định `<table>` không nhận).

### `event.metaKey` / `ctrlKey` / `shiftKey`
`MouseEvent` cho biết phím bổ trợ nào đang được giữ khi click — dùng để phân biệt click / ⌘-click / Shift-click.

### `flatMap`
`rows.flatMap(r => r.origPath ? [r.path, r.origPath] : [r.path])` — mỗi phần tử sinh ra 1 hoặc 2 phần tử, rồi trải phẳng thành một mảng. ≈ `Stream.flatMap` của Java.

## Cách chạy và kiểm tra
```bash
pnpm tauri dev
cd src-tauri && cargo test stage      # 7 test của bước này
```
Code bước 7 chưa commit → repo `moon-git` có sẵn file để thử.

| Thao tác | Kết quả mong đợi |
|---|---|
| Chưa chọn gì | Stage, Unstage đều mờ |
| Click một file `Modified` | Dòng tô xanh; Stage sáng, Unstage mờ |
| Bấm **Stage** | State thành `Staged`, chữ M chuyển xanh lá; Stage mờ, Unstage sáng (file vẫn được chọn) |
| Bấm **Unstage** | Quay lại `Modified` |
| ⌘-click thêm vài dòng | Các dòng đó được thêm/bớt khỏi vùng chọn |
| Click một dòng, Shift-click dòng khác | Chọn cả dải ở giữa (theo thứ tự đang sắp xếp) |
| Click vào bảng, nhấn ⌘A | Chọn tất cả |
| Chọn file `Untracked` → Stage | State `Added`, chữ `A` |
| Chuột phải một dòng **chưa** chọn | Chỉ dòng đó được chọn; menu Stage / Unstage (mục không áp dụng bị mờ) |
| Chuột phải trong vùng đã chọn nhiều dòng | Giữ nguyên vùng chọn; menu áp dụng cho tất cả |
| Trong terminal `git mv docs/learning/README.md docs/learning/X.md`, quay lại app, chọn `X.md` (Renamed) → Unstage | `X.md` thành Untracked, `README.md` thành Missing. Dọn lại: `git add -A docs/learning && git mv docs/learning/X.md docs/learning/README.md` rồi `git restore --staged docs/learning` |
| Kiểm tra bằng terminal sau khi stage trong app | `git status` cho kết quả giống bảng |

## Ghi chú cho sau này
- Chưa có Stage All / Unstage All riêng — dùng ⌘A rồi Stage/Unstage.
- Chưa stage được **một phần** file (từng đoạn — `git add -p`). Có thể làm sau khi có diff viewer (bước 8).
- Chưa có phím tắt cho Stage/Unstage.
