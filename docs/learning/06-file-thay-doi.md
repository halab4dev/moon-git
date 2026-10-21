# Bước 6 — Danh sách file thay đổi

## Trạng thái
- `cargo test`: 45/45 pass (thêm 9 test cho parser status: 6 test chuỗi viết tay, 3 test trên repo thật — có đổi tên file, xóa file, thư mục mới, conflict khi merge). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21).

## Mục tiêu
- Chọn một repo → màn hình chính hiện **bảng** file thay đổi kiểu SmartGit: cột **Name**, **State**, **Location**; bấm tiêu đề cột để sắp xếp, bấm lần nữa để đảo chiều. Mỗi file có chữ cái trạng thái (A/M/D/R/C/T/U/?) có màu.
- Không có gì thay đổi → "Nothing to commit, working tree clean."
- Tự làm mới khi quay lại cửa sổ app; có nút **Refresh**.
- Stage/unstage là bước 7, xem diff là bước 8.

## Lệnh git và vì sao từng tham số
```
git --no-optional-locks status --porcelain=v2 -z --untracked-files=all
```
| Tham số | Vì sao |
|---|---|
| `--porcelain=v2` | Định dạng **dành cho máy đọc**: ổn định giữa các phiên bản git, không bị dịch sang ngôn ngữ khác. Bản v2 có thêm thông tin (mode, hash, điểm rename) so với v1 |
| `-z` | Ngăn cách bản ghi bằng ký tự **NUL** (`\0`) thay vì xuống dòng, và **không bao giờ đặt đường dẫn trong ngoặc kép**. Tên file có dấu cách, xuống dòng, ký tự Unicode đều an toàn |
| `--untracked-files=all` | Liệt kê **từng file** trong thư mục mới, thay vì chỉ `thư-mục/`. Bước 7 stage từng file sẽ cần |
| `--no-optional-locks` | Để làm nhanh, `git status` đôi khi ghi lại file index và phải giữ khóa `.git/index.lock`. App gọi status ngầm (khi focus cửa sổ) không được làm lệnh git bạn đang gõ trong terminal báo lỗi "index.lock exists". Tùy chọn này bảo git bỏ qua việc ghi không bắt buộc đó |

## Định dạng output và cách parse
Mỗi bản ghi (ngăn bởi `\0`) bắt đầu bằng một ký tự loại:
```
1 MM N... 100644 100644 100644 <hashHEAD> <hashIndex> src/a b.txt       ← file thay đổi
2 R. N... 100644 100644 100644 <h> <h> R100 new.txt \0 old.txt            ← đổi tên (đường dẫn cũ là bản ghi kế tiếp)
u UU N... 100644 100644 100644 100644 <h1> <h2> <h3> both.txt             ← conflict
? fresh dir/a.txt                                                          ← chưa track
```
- **`XY`** (vd. `MM`, `A.`, `.D`): **X** = thay đổi trong index (đã stage), **Y** = thay đổi trong working tree (chưa stage), `.` = không đổi.
  - `MM` → file vừa có phần đã stage, vừa có phần sửa tiếp chưa stage → xuất hiện ở **cả hai** nhóm.
  - `A.` → file mới đã stage. `.D` → đã xóa nhưng chưa stage việc xóa.
- Số trường cố định trước đường dẫn (8 với loại `1`, 9 với `2`, 10 với `u`) → dùng `splitn(n, ' ')`: tách đúng n phần, **phần cuối giữ nguyên dấu cách** — đó là đường dẫn.
- Gặp gì không hiểu (loại lạ, thiếu trường, `XY` không đúng 2 ký tự…) → trả lỗi `UnexpectedOutput` thay vì đoán. Theo CLAUDE.md: không nuốt lỗi.
- Test `rejects_malformed_records` đã bắt được một lỗi khi viết: bản ghi đổi tên thiếu đường dẫn cũ vẫn lọt qua (chuỗi rỗng sau `\0` cuối bị coi là đường dẫn). Đã sửa.

## File đã tạo / sửa

**Rust**
| File | Việc |
|---|---|
| `git/status.rs` | **Mới.** `read_status` (chạy git), `parse_status` (parse), 9 test |
| `models.rs` | `ChangeKind` (enum), `FileChange`, `RepoStatus` |
| `commands/status.rs` | **Mới.** `get_status(repo_id)` — async |
| `commands/mod.rs` | **Mới:** hàm dùng chung `run_blocking` |
| `commands/clone.rs` | Dùng `run_blocking` thay cho code tự viết ở bước 5 |
| `storage/repos.rs` | `repo_path(id)`: tra đường dẫn theo id |
| `error.rs` | Thêm `UnexpectedOutput` |

**Frontend**
| File | Việc |
|---|---|
| `types/status.ts`, `api/status.ts` | Kiểu + wrapper `getStatus` |
| `components/fileRows.ts` | **Mới.** `buildRows` (gộp 4 danh sách thành một dòng/file), `stateOf` (XY → State kiểu SmartGit), `STATE_TABLE` (tên, chữ cái, màu, thứ tự), `sortRows` — TypeScript thuần, không dính Vue |
| `components/FileTable.vue` | **Mới.** Bảng Name / State / Location, tiêu đề bấm để sắp xếp, nhớ cách sắp xếp |
| `views/RepoView.vue` | Tải status, nút Refresh, tự làm mới khi focus, hiện bảng |
| `styles.css` | Thêm màu `--warning` (cho "M") |

## Quyết định thiết kế
- **Command nhận `repo_id`, không nhận đường dẫn.** Rust tự tra đường dẫn trong danh sách. Frontend vì vậy chỉ chạy được git trong thư mục bạn đã thêm vào app — giảm rủi ro nếu sau này giao diện có lỗi.
- **Status chạy nền** (`async` + `run_blocking`): repo lớn có thể mất cả giây, giao diện không đơ.
- **State nằm trong `RepoView`, không trong store**: chỉ màn hình này dùng; đổi repo thì `RepoView` được tạo mới (nhờ `:key` ở bước 3) nên không lẫn status giữa các repo.
- **Làm mới khi focus cửa sổ** thay vì theo dõi file system: đơn giản, không thêm thư viện, đủ cho cách dùng thường ngày (sửa ở IDE → quay lại app). Theo dõi file (watcher) có thể thêm sau.

## Luồng dữ liệu
```
Chọn repo → App.vue tạo RepoView(:key = repo.id)
  onMounted → refresh()                       (và mỗi lần window "focus")
    getStatus(repo.id) → invoke("get_status", { repoId })
      commands::status::get_status(repo_id, store)      ← async
        store.repo_path(id)  → PathBuf (hoặc NotFound)
        run_blocking(move || git::status::read_status(&path))   ← thread worker
          run_git(["--no-optional-locks","status","--porcelain=v2","-z","--untracked-files=all"])
          parse_status(output) → RepoStatus { staged, unstaged, untracked, conflicted }
      ← JSON
    status.value = ...  → FileTable: buildRows → sortRows → vẽ bảng
```
Repo bị xóa/di chuyển trên ổ đĩa → `run_git` báo `Folder does not exist: …` → hiện chữ đỏ trong RepoView.

## Bảng file kiểu SmartGit

Rust vẫn trả 4 danh sách (`staged`, `unstaged`, `untracked`, `conflicted`) — đúng với cách git nghĩ. Frontend gộp thành **một dòng mỗi file** để hiển thị (`buildRows` trong `fileRows.ts`).

Một file có thể nằm ở cả staged lẫn unstaged → mỗi dòng có **một State**, đặt tên theo [bảng trạng thái file của SmartGit](https://docs.syntevo.com/SmartGit/Latest/Repositories-Directories-and-Files) (hàm `stateOf` trong `fileRows.ts`). X = phía index (đã stage), Y = phía working tree:

| git (XY) | State | Chữ |
|---|---|---|
| `u` (conflict) | Conflicted | U |
| `M.` | Staged | M (xanh lá) |
| `MM` | Staged Modified | M (xanh lá) |
| `A.` | Added | A |
| `AM` | Added Modified | A |
| `R.` / `RM` | Renamed / Renamed (modified) — Location có `← đường dẫn cũ` | R |
| `C.` | Copied *(SmartGit không có)* | C |
| `D.` | Removed (đã stage việc xóa) | D |
| `.M` | Modified | M (vàng) |
| `T` | Type Changed *(file ↔ symlink; SmartGit không có)* | T |
| `.D`, `MD`, `AD` | Missing (file không còn trên ổ đĩa) | ! |
| `.A` (`git add -N`) | Intent-to-Add | A |
| `?` | Untracked *(SmartGit gọi là Unversioned)* | ? |

Chưa làm được (cần thêm thông tin ngoài `git status`): *Staged, WT as HEAD*, *Modified (File Mode)*, *Modified (EOLs only)*, *Rename Source*, *Assume-Unchanged*, *Skipped*, *Ignored*.

Màu chữ cái: **xanh lá** = đã stage cho lần commit tới, **vàng** = thay đổi chưa stage, **đỏ** = conflict / xóa / mất file, **xanh dương** = đổi tên/copy.

Mỗi dòng còn giữ `hasStaged` / `hasUnstaged` — bước 7 dùng để biết nút nào (Stage / Unstage) có tác dụng.

**Sắp xếp** (`sortRows`):
- Bấm tiêu đề cột → sắp tăng dần theo cột đó (▲); bấm lại → giảm dần (▼).
- Cột **State** sắp theo **mức quan trọng**, không theo bảng chữ cái — đúng thứ tự của bảng trên: Conflicted → Staged → Staged Modified → Added → Added Modified → Renamed → Renamed (modified) → Copied → Removed → Modified → Type Changed → Missing → Intent-to-Add → Untracked. Thứ tự này nằm ở một chỗ: mảng `STATE_TABLE` trong `fileRows.ts`.
- Hai dòng bằng nhau ở cột chính thì xét tiếp cột khác (Name → Location; State → Location → Name; Location → Name) → thứ tự luôn ổn định, không "nhảy" khi làm mới.
- `Intl.Collator` với `numeric: true`: `file9` đứng trước `file10` (sắp kiểu chữ thường sẽ ra ngược); `sensitivity: "base"`: không phân biệt hoa thường.
- Cách sắp xếp được nhớ (localStorage), dùng chung cho mọi repo. Mặc định: Name tăng dần.

**Bảng HTML:** `table-layout: fixed` → độ rộng cột theo tiêu đề (Name 38%, State 24%, Location phần còn lại), đường dẫn dài bị cắt bằng `…` (rê chuột để xem đủ). Tiêu đề `position: sticky` → đứng yên khi cuộn danh sách dài. `aria-sort` cho trình đọc màn hình biết cột nào đang được sắp xếp.

**Vì sao tách `fileRows.ts` khỏi component:** logic gộp/sắp xếp là hàm thuần (vào dữ liệu → ra dữ liệu), không phụ thuộc Vue → dễ đọc, dễ viết unit test khi thêm Vitest (giai đoạn 2).

## Khái niệm Rust mới

### `while let`
```rust
let mut records = output.split('\0');
while let Some(record) = records.next() { ... }
```
"Lặp chừng nào còn khớp mẫu `Some(...)`". Dùng `while let` thay `for` ở đây vì bên trong vòng lặp, bản ghi đổi tên cần **lấy thêm** bản ghi kế tiếp (`records.next()`) — `for` không cho làm vậy. ≈ `while (it.hasNext()) { var r = it.next(); ... }` của Java.

### Iterator lười (lazy) — `split`, `splitn`
`output.split('\0')` không tạo danh sách ngay; nó trả về iterator, mỗi `next()` mới cắt ra một phần. Giống `Stream`/`Iterator` Java. `.collect()` mới gom thành `Vec`.

### `&str` trỏ vào chuỗi gốc + lifetime có tên `'a`
```rust
fn split_fields<'a>(rest: &'a str, count: usize, record: &str) -> Result<Vec<&'a str>, AppError>
```
Các phần sau khi tách (`&str`) **không sao chép** dữ liệu — chúng là "cửa sổ" nhìn vào chuỗi `rest`. Compiler cần biết: các phần trả về sống bao lâu? `'a` nói rõ: "chúng sống đúng bằng `rest`" (không liên quan `record`). Đây là lần đầu phải viết lifetime bằng tay — ở bước 3 `'_` để compiler tự suy. Không cần nhớ quy tắc, compiler sẽ báo khi thiếu.

### Mẫu tuple trong `let … else`
```rust
let (Some(x), Some(y), None) = (chars.next(), chars.next(), chars.next()) else {
    return Err(unexpected(record));
};
```
Lấy 3 ký tự liên tiếp và yêu cầu: có ký tự 1, có ký tự 2, **không có** ký tự 3 → đúng 2 ký tự. Một dòng kiểm tra cả độ dài lẫn lấy giá trị.

### `match` với nhiều mẫu và guard
- `"!" | "#" => {}` — `|` = "hoặc", khớp một trong hai.
- `ChangeKind::Renamed | ChangeKind::Copied => ...`
- `Some(orig) if !orig.is_empty() => orig` — guard (đã gặp ở bước 2).

### `char` vs `&str`
`'A'` (nháy đơn) là một ký tự `char`; `"A"` (nháy kép) là chuỗi. Giống Java.

### Generic có ràng buộc: `Send + 'static`
```rust
pub async fn run_blocking<T>(work: impl FnOnce() -> Result<T, AppError> + Send + 'static) -> Result<T, AppError>
where T: Send + 'static
```
- `Send` = "an toàn khi chuyển sang thread khác" — đa số kiểu đều có; `Rc`, tham chiếu tới dữ liệu của thread hiện tại thì không.
- `'static` = "không mượn gì có thể biến mất" — closure phải **sở hữu** mọi thứ nó dùng (vì thế dùng `move ||`).
- `where T: ...` — cách viết ràng buộc cho gọn, ≈ `<T extends ...>` của Java.
- Compiler kiểm tra các điều này **lúc biên dịch**: không thể vô tình gửi dữ liệu không an toàn sang thread khác — Java chỉ phát hiện lỗi kiểu này lúc chạy (nếu may).

### `use super::run_blocking`
`super` = module cha. `commands/status.rs` lấy hàm từ `commands/mod.rs`.

## Khái niệm Vue / TS mới

### `onUnmounted` — dọn dẹp
```ts
onMounted(() => window.addEventListener("focus", refresh));
onUnmounted(() => window.removeEventListener("focus", refresh));
```
Đăng ký listener ở `onMounted` thì **phải** gỡ ở `onUnmounted` (khi component bị hủy, ví dụ đổi repo). Không gỡ → listener cũ vẫn chạy, gọi status cho repo không còn hiển thị. ≈ `@PreDestroy`.

### `Record<K, V>`
`Record<ChangeKind, { letter: string; label: string }>` = object có **đúng** mỗi key là một `ChangeKind`. Thiếu một loại → lỗi compile. Thêm loại mới bên Rust + TS mà quên bảng này → TypeScript nhắc ngay.

### `const props = defineProps<...>()`
Lần này cần gán vào biến `props` vì code trong `<script>` đọc `props.repo.id`. (Trong template thì dùng thẳng `repo`.)

### `computed` nối chuỗi xử lý
```ts
const rows = computed(() => sortRows(buildRows(props.status), sortKey.value, ascending.value));
```
Khi `status` đổi (refresh) **hoặc** người dùng bấm sắp xếp, `rows` tự tính lại. Không cần tự gọi hàm sắp xếp ở từng chỗ.

### `watch` nhiều nguồn
`watch([sortKey, ascending], ...)` — chạy khi một trong hai đổi; dùng để lưu lựa chọn sắp xếp.

### `import { ..., type SortKey }`
`type` đánh dấu import chỉ là kiểu (biến mất khi build ra JS).

### `v-else-if`
Đi sau `v-if`/`v-else-if`: chỉ hiện "Loading…" khi chưa có status **và** đang tải.

## Cách chạy và kiểm tra
```bash
pnpm tauri dev
cd src-tauri && cargo test status     # 9 test của bước này (cargo test: cả 45)
```

| Thao tác | Kết quả mong đợi |
|---|---|
| Chọn `moon-git` (có file chưa commit) | Bảng: các file `M` vàng — State `Modified`; các file `?` — State `Untracked` |
| Bấm tiêu đề **Name** | ▲ chuyển thành ▼, thứ tự đảo ngược |
| Bấm **Location** | Sắp theo thư mục (`docs/learning`, `src/api`, `src/components`, `src-tauri/src/...`), trong cùng thư mục theo tên |
| Bấm **State** | Staged / Added lên trước, rồi Modified, Untracked cuối cùng |
| Đóng app, mở lại | Vẫn sắp theo cột đã chọn |
| Trong terminal: tạo file mới rồi `git add` nó, quay lại app | Tự làm mới: State `Added`, chữ `A` xanh lá |
| Sửa tiếp file đó, quay lại app | State `Added Modified` |
| `git mv docs/learning/README.md docs/learning/INDEX.md` | `INDEX.md` chữ `R`, State `Renamed`, Location `docs/learning ← docs/learning/README.md` |
| `git restore --staged . && git mv docs/learning/INDEX.md docs/learning/README.md` | Trả lại như cũ (nhớ chạy để dọn) |
| Bấm **Refresh** | Nút đổi thành "Refreshing…" thoáng qua |
| Chọn repo không có thay đổi (vd. repo vừa clone) | "Nothing to commit, working tree clean." |
| Đổi tên thư mục của một repo trên ổ đĩa rồi chọn nó trong app | Chữ đỏ `Folder does not exist: …` |

Rê chuột lên một dòng: tooltip hiện đường dẫn đầy đủ (đổi tên: `cũ → mới`).

## Ghi chú cho sau này
- Danh sách chưa ảo hóa: repo có hàng nghìn file thay đổi (vd. quên ignore `node_modules`) sẽ chậm. Có thể giới hạn/ảo hóa khi gặp.
- Submodule hiện hiển thị như một file thường.
- Chưa hiện branch hiện tại — bước 10 (cùng ahead/behind).
