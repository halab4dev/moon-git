# Bước 8 — Xem diff side-by-side (Monaco)

## Trạng thái
- `cargo test`: 61/61 pass (thêm 9 test: unstaged/staged/conflict, file mới/xóa/đổi tên, repo chưa có commit, file nhị phân, file lớn, tên file có `:`, chặn đường dẫn ra ngoài repo). `cargo clippy`: sạch. `pnpm build`: OK.
- Dev server tải được module Monaco và web worker (đã kiểm tra bằng request trực tiếp). Giao diện: bạn đã thử và xác nhận (cả ô diff ở cửa sổ chính lẫn cửa sổ riêng khi double-click). Đã commit.

## Mục tiêu
- Bố cục kiểu SmartGit: **bảng file ở trên, diff ở dưới**, thanh kéo ở giữa để chỉnh tỉ lệ (được nhớ).
- Chọn **một** file → diff side-by-side, tô màu cú pháp, đánh dấu tới từng ký tự thay đổi.
- File vừa có phần đã stage vừa có phần chưa stage → nút chuyển **Working Tree** / **Staged**.
- Tùy chọn **Side by side** (bỏ tick = xem inline một cột) và **Ignore whitespace** (được nhớ).
- ↑ / ↓ trong bảng file: chuyển file, diff đổi theo.
- File nhị phân / lớn hơn 5 MB: thông báo thay vì hiển thị.

## Thư viện mới: `monaco-editor` (npm)
- Editor của VS Code, dùng ở đây **chỉ để đọc** và so sánh.
- Vì sao chọn: CLAUDE.md đã chọn; diff đẹp và đầy đủ nhất (tô màu ~80 ngôn ngữ, đánh dấu ký tự, thanh tổng quan bên phải, inline/side-by-side, bỏ qua khoảng trắng). So sánh với CodeMirror 6 và tự vẽ từ `git diff` đã trình bày trước khi cài — bạn chọn Monaco.
- Chi phí: một file JS ~2,7 MB. **Chỉ tải khi mở diff lần đầu** (`defineAsyncComponent`), nên app vẫn khởi động nhanh (phần chính ~116 KB).
- Chỉ nạp phần cần thiết (`src/monaco.ts`):
  - `monaco-editor/editor/editor.api` — lõi editor, có sẵn diff editor.
  - `monaco-editor/basic-languages/monaco.contribution` — tô màu cú pháp; mỗi ngôn ngữ là một file nhỏ, chỉ tải khi gặp file thuộc ngôn ngữ đó.
  - `monaco-editor/editor/editor.worker?worker` — web worker tính diff.
  - **Không** nạp dịch vụ ngôn ngữ (gợi ý code TypeScript/JSON/CSS…): rất nặng mà diff chỉ đọc thì không cần.

## Ba chế độ so sánh

| Chế độ | Bên trái | Bên phải | Khi nào |
|---|---|---|---|
| **Working Tree** (`unstaged`) | Index | Working Tree (file trên ổ đĩa) | File có thay đổi chưa stage (Modified, Untracked, Missing…) |
| **Staged** (`staged`) | HEAD | Index | File có thay đổi đã stage (Staged, Added, Removed, Renamed…) |
| **Conflict** (`conflict`) | HEAD | Working Tree (có dấu `<<<<<<<`) | File Conflicted |

"Staged Modified" / "Added Modified" có cả hai → hiện nút chuyển; mặc định Working Tree. Stage xong mà file không còn thay đổi chưa stage → tự chuyển sang Staged.

## Rust: lấy hai phiên bản file

| Phiên bản | Cách lấy |
|---|---|
| HEAD | `git cat-file blob HEAD:<path>` (file đổi tên: dùng tên cũ) |
| Index | `git cat-file blob :0:<path>` |
| Working Tree | đọc file trên ổ đĩa |

Chi tiết đáng chú ý:
- **`cat-file -s`** (in kích thước) chạy trước: thất bại = phiên bản đó **không tồn tại** (file mới, file đã xóa, repo chưa có commit) → bên đó để trống. Biết kích thước trước để không tải file > 5 MB.
- **`:0:`** — "stage 0" của index, viết rõ để file tên `1:x` không bị hiểu thành `:1:x` (stage 1, dùng khi merge). Có test riêng.
- **Nhị phân**: giống quy tắc của git — có byte `0` trong 8000 byte đầu → nhị phân, không gửi nội dung.
- **`run_git_bytes`**: biến thể mới của `run_git` trả về **byte** thay vì chuỗi — cần để nhận ra file nhị phân trước khi đổi sang chữ.
- **Symlink**: git lưu symlink như một file nhỏ chứa đường dẫn đích → bên Working Tree cũng hiện đường dẫn đích (`symlink_metadata` + `read_link`), không đọc file mà nó trỏ tới.
- **Chặn đường dẫn ra ngoài repo**: `check_relative` chỉ chấp nhận đường dẫn tương đối "bình thường" — từ chối `../x`, `/etc/passwd`, `C:\…`. Dù đường dẫn luôn đến từ git status, Rust vẫn tự kiểm tra (không tin dữ liệu từ giao diện).
- Không chạy filter/`textconv` của git (vd. Git LFS) → luôn so sánh nội dung thật trong git.

## File đã tạo / sửa

**Rust**
| File | Việc |
|---|---|
| `git/diff.rs` | **Mới.** `read_diff(repo, path, orig_path, mode)`, 9 test |
| `git/runner.rs` | Thêm `run_git_bytes`; `run_git` gọi qua nó |
| `models.rs` | `DiffMode` (nhận từ frontend → `Deserialize`), `FileDiff` |
| `commands/diff.rs` | **Mới.** `get_file_diff` (chạy nền) |

**Frontend**
| File | Việc |
|---|---|
| `monaco.ts` | **Mới.** Nạp Monaco tối thiểu, đăng ký worker, chọn ngôn ngữ theo đuôi file, theme sáng/tối |
| `components/DiffEditor.vue` | **Mới.** Bọc diff editor của Monaco: tạo, đổi nội dung, giữ vị trí cuộn khi làm mới, dọn dẹp |
| `components/DiffPanel.vue` | **Mới.** Tải diff cho file đang chọn, nút chế độ, tùy chọn, các thông báo |
| `views/RepoView.vue` | Bố cục trên/dưới + thanh kéo; truyền file đang chọn cho DiffPanel |
| `components/FileTable.vue` | Phím ↑/↓ |
| `types/diff.ts`, `api/diff.ts` | Kiểu + wrapper |
| `vite.config.ts` | Dịch cú pháp JS về mức webview cũ nhất (Safari 15 / Chrome 105) cho cả dev và build; không cảnh báo về file Monaco lớn |

## Luồng dữ liệu
```
FileTable: chọn 1 file → selectedPaths
RepoView: diffRow = file đó (null nếu chọn 0 hoặc nhiều file)
DiffPanel (row, version):
  modes = [unstaged?, staged?] hoặc [conflict]  → nút chế độ
  watch(row.path, mode, version) → load()
    getFileDiff(repoId, path, origPath, mode) → invoke("get_file_diff")
      commands::diff::get_file_diff → repo_path → run_blocking:
        git::diff::read_diff
          kiểm tra đường dẫn → đọc 2 phiên bản (cat-file / file trên đĩa)
          → nhị phân? quá lớn? → FileDiff { oldText, newText, oldLabel, newLabel, ... }
    ← FileDiff
  → DiffEditor (tải Monaco lần đầu)
      createDiffEditor → setModel({ original: oldText, modified: newText })
      Monaco (web worker) tính khác biệt → tô màu
Focus cửa sổ → RepoView refresh status → version++ → DiffPanel tải lại → DiffEditor giữ vị trí cuộn
```

## Sự cố gặp phải: diff không hiện trên macOS 12

**Triệu chứng:** chọn file, khu vực diff trống. Log của `pnpm tauri dev` (Vite chuyển lỗi của webview ra terminal) có:
```
[Vue warn]: Unhandled error during execution of async component loader
[Unhandled rejection] SyntaxError: Unexpected token '{'
```

**Nguyên nhân:**
- Webview của Tauri trên macOS là **WebKit của hệ điều hành** — không phải của ứng dụng Safari.
- Trên macOS 12, WebKit hệ thống là bản **613** ≈ **Safari 15.6**. Máy có Safari 17.5 nhưng đó là bản cập nhật riêng của ứng dụng Safari, mang engine riêng; app khác vẫn dùng engine cũ.
- Monaco 0.57 dùng cú pháp `class X { static { ... } }` (khối khởi tạo tĩnh, có từ Safari 16.4) ở 228 file → engine cũ không đọc được → cả file Monaco không tải được.

**Cách tìm ra:** macOS có sẵn `jsc` (`/System/Library/Frameworks/JavaScriptCore.framework/Versions/A/Helpers/jsc`) — dòng lệnh của **đúng** engine JavaScript hệ thống. Dùng hàm `checkModuleSyntax(source)` của nó để thử từng tính năng và từng file Monaco → xác định chính xác cú pháp bị thiếu.

**Cách sửa** (`vite.config.ts`): khai báo mức engine cũ nhất cần hỗ trợ — `WEBVIEW_TARGET = ["safari15", "chrome105"]` — để Vite **viết lại** cú pháp mới thành cú pháp cũ tương đương (transpile, giống Babel):
1. `build.target` — bản build chính thức (cả code app, thư viện, web worker).
2. `optimizeDeps.rolldownOptions.transform.target` — thư viện được Vite gói sẵn khi chạy dev.
3. Plugin nhỏ `lowerDependencySyntax` (chỉ chạy khi dev) — file trong `node_modules` được phục vụ thẳng không qua bước gói sẵn (chuỗi file của web worker Monaco).

Đây cũng là khuyến nghị của Tauri: luôn đặt `build.target` theo webview của từng hệ điều hành (template mới bỏ dòng này nên lỗi mới lộ ra).

**Kiểm tra sau khi sửa:** dùng `jsc` đọc toàn bộ 84 file của bản build và 237 module mà webview tải khi mở diff ở chế độ dev → tất cả đọc được, không còn `static {`.

**Bài học:**
- Với Tauri, **webview của từng hệ điều hành là "trình duyệt" thật** của app — không phải Chrome mới nhất. macOS cũ → WebKit cũ; Ubuntu → WebKitGTK (theo bản phân phối); Windows → WebView2 (Chromium, tự cập nhật).
- Lỗi khi tải component bất đồng bộ (`defineAsyncComponent`) mặc định chỉ là một cảnh báo — xem log của `tauri dev` khi giao diện "không hiện gì".

## Khái niệm Rust mới

### `matches!`
```rust
matches!(c, Component::Normal(_) | Component::CurDir)
```
Macro có sẵn: "giá trị có khớp mẫu này không?" → `true`/`false`. Gọn hơn viết `match` đầy đủ chỉ để lấy bool. `_` = "bất kỳ giá trị nào bên trong".

### `Path::components()`
Tách đường dẫn thành từng phần có kiểu: `Normal("src")`, `ParentDir` (`..`), `RootDir` (`/`), `Prefix` (`C:` trên Windows)… — an toàn hơn tự cắt chuỗi theo `/`, và đúng trên mọi hệ điều hành.

### Enum riêng tư làm kiểu trung gian
`enum Version { Missing, TooLarge, Content(Vec<u8>) }` chỉ dùng trong file — mô tả rõ ba khả năng của một phiên bản file, thay vì dùng `Option<Option<Vec<u8>>>` khó đọc. `Content(Vec<u8>)` là biến thể **mang dữ liệu** (đã nhắc ở bước 2).

### Slice của mảng byte: `&bytes[..n]`
`&bytes[..bytes.len().min(8000)]` — "nhìn" 8000 byte đầu (hoặc ít hơn nếu file ngắn) mà không sao chép. `.contains(&0)` tìm byte 0.

### `Deserialize` cho enum
`DiffMode` đi **từ frontend vào** Rust (`mode: "staged"`) → derive `Deserialize`, kèm `rename_all = "camelCase"`. Chuỗi khác ba giá trị hợp lệ → Tauri báo lỗi ngay, command không chạy.

### `into_owned()`
`String::from_utf8_lossy` trả `Cow` — "có thể mượn, có thể sở hữu" (mượn nếu byte đã là UTF-8 hợp lệ, tạo mới nếu phải thay ký tự lỗi). `into_owned()` → luôn ra `String` sở hữu.

## Khái niệm Vue / web mới

### Transpile (dịch cú pháp) và `target`
Công cụ build (Vite → Rolldown/Oxc) đọc JS hiện đại và viết lại thành JS cũ hơn làm **cùng việc**. Ví dụ `class A { static { this.x = 1 } }` thành `class A {}` + `A.x = 1` chạy ngay sau đó. `target` nói cho nó biết engine cũ nhất cần chạy được.

### Plugin Vite
`{ name, apply: "serve", transform(code, id) { ... } }` — Vite gọi `transform` cho mọi file nó phục vụ; plugin trả về code mới (hoặc `null` = không đổi). `apply: "serve"` = chỉ chạy khi dev.

### Dùng thư viện "không phải Vue" trong component
Monaco tự quản lý DOM của nó. Cách dùng chuẩn:
1. `<div ref="container">` — chỗ trống cho Monaco.
2. `onMounted` — tạo editor vào div đó (lúc này div đã có trên trang).
3. `watch` props — cập nhật editor khi dữ liệu đổi.
4. `onBeforeUnmount` — **`dispose()`** editor và các model. Không dispose → rò bộ nhớ mỗi lần đổi repo.

Biến giữ editor là `let editor = null`, **không** dùng `ref()`: Vue sẽ bọc object trong proxy reactive, Monaco không mong đợi điều đó và có thể chạy sai/chậm.

### `defineAsyncComponent` + `import()` động
```ts
const DiffEditor = defineAsyncComponent(() => import("./DiffEditor.vue"));
```
Component (và mọi thứ nó import — Monaco) được tách thành file riêng, chỉ tải khi lần đầu hiển thị. ≈ lazy loading class trong Java.

### Web Worker
Một luồng JavaScript chạy nền trong webview. Monaco tính diff ở đó để file dài không làm đơ giao diện. Vite gói worker nhờ hậu tố `?worker` trong import.

### Chống kết quả cũ (lại)
Bấm nhanh qua nhiều file → nhiều request cùng lúc. Mỗi request được đánh số (`lastRequest`); chỉ kết quả của request mới nhất được hiển thị. Cùng ý tưởng với gợi ý tên thư mục ở bước 5.

### Thanh kéo (splitter)
`mousedown` trên thanh → đăng ký `mousemove` / `mouseup` trên **window** (để vẫn theo được chuột khi nó ra khỏi thanh) → tính tỉ lệ theo vị trí chuột → `flex-basis` của vùng bảng file. `{ once: true }`: listener `mouseup` tự gỡ sau lần đầu.

### `CSS.escape` + `scrollIntoView`
Khi bấm ↑/↓, tìm dòng bằng `tr[data-path="..."]` (`CSS.escape` để tên file có ký tự đặc biệt không phá vỡ bộ chọn) rồi cuộn tới nó (`block: "nearest"`: chỉ cuộn khi cần).

## Cách chạy và kiểm tra
```bash
pnpm tauri dev
cd src-tauri && cargo test diff      # 9 test của bước này
```
Code bước 8 chưa commit → repo `moon-git` có sẵn file để thử.

| Thao tác | Kết quả mong đợi |
|---|---|
| Chọn `moon-git`, click một file Modified (vd. `RepoView.vue`) | Lần đầu chờ ~1 giây (tải Monaco); diff 2 cột, trái "Index", phải "Working Tree", có tô màu cú pháp |
| ↑ / ↓ | Chuyển file, diff đổi theo |
| Click file Untracked (vd. `diff.rs`) | Bên trái trống, bên phải toàn bộ file (màu xanh) |
| Stage một file, chọn lại nó | Nút **Staged** sáng; trái "HEAD", phải "Index" |
| Stage file rồi sửa tiếp (Staged Modified) | Hai nút **Working Tree** / **Staged**, bấm để chuyển |
| Bỏ tick **Side by side** | Diff một cột (inline) |
| Tick **Ignore whitespace** | Dòng chỉ khác khoảng trắng đầu/cuối không còn bị tô |
| Chọn nhiều file | "N files selected." |
| Kéo thanh giữa bảng và diff | Đổi tỉ lệ; đóng mở app vẫn giữ |
| Sửa file đang xem trong IDE, quay lại app | Diff cập nhật, vị trí cuộn giữ nguyên |
| Đổi Dark/Light của macOS | Diff đổi theme |
| Chép một ảnh PNG vào repo, chọn nó | "Binary file — no text to compare." |

## Bổ sung: double-click → cửa sổ diff riêng

- Double-click một file trong bảng → mở **cửa sổ riêng** (to, kéo giãn được) hiện **toàn bộ file** kèm các thay đổi. Cửa sổ này **không chặn** cửa sổ chính: vẫn chọn file khác, stage, mở thêm cửa sổ khác được.
- **Mỗi file một cửa sổ**: double-click lại cùng file → chỉ đưa cửa sổ đó lên trước (và bỏ thu nhỏ nếu đang thu nhỏ).
- Cửa sổ tự làm mới khi được focus; nó tự đọc lại status nên nút Working Tree / Staged luôn đúng. File hết thay đổi (đã commit/discard) → thông báo thay vì diff.
- Đóng **cửa sổ chính** → thoát cả app (đóng luôn các cửa sổ diff).
- Ghi chú: Monaco **mặc định đã hiện toàn bộ file** (`hideUnchangedRegions` tắt) — ô diff ở cửa sổ chính chỉ nhỏ nên phải cuộn. `DiffEditor.vue` giờ ghi rõ tùy chọn này.

### Cách làm
| File | Việc |
|---|---|
| `src/windows.ts` | **Mới.** `openDiffWindow`: tạo `WebviewWindow` (hoặc focus cửa sổ đã có) |
| `src/views/DiffWindow.vue` | **Mới.** Màn hình của cửa sổ diff: đọc file cần hiện từ URL, tải status, dùng lại `DiffPanel` |
| `src/main.ts` | Chọn màn hình gốc theo `?view=`: không có → `App` (cửa sổ chính), `diff` → `DiffWindow` |
| `components/FileTable.vue` | `@dblclick` → phát sự kiện `rowOpen` |
| `views/RepoView.vue` | Nhận `rowOpen` → `openDiffWindow` |
| `src-tauri/src/lib.rs` | `on_window_event`: cửa sổ `main` bị hủy → `exit(0)` |
| `capabilities/default.json` | Áp dụng cho `main` và `diff-*`; thêm quyền tạo cửa sổ, focus, bỏ thu nhỏ |

### Khái niệm mới
- **Nhiều cửa sổ trong Tauri:** mỗi cửa sổ là một **webview riêng** với JavaScript, Vue, Pinia **riêng** — như hai tab trình duyệt. Chúng không chia sẻ biến JS; dữ liệu dùng chung nằm ở **Rust** (`RepoStore`…), cửa sổ nào cũng gọi command được. Vì vậy cửa sổ diff tự gọi `get_status` / `get_file_diff`.
- **Label cửa sổ:** tên định danh duy nhất (`main`, `diff-1a2b3c4d`), chỉ được chứa chữ, số và `- / : _` → dùng hash ngắn (FNV-1a) của repo + đường dẫn file. Cùng file → cùng label → tìm lại được bằng `WebviewWindow.getByLabel`.
- **Một trang, nhiều màn hình mà không cần router:** mọi cửa sổ tải cùng `index.html`; tham số URL `?view=diff&repoId=…&path=…` quyết định component gốc. (Khi app có nhiều màn hình hơn có thể dùng `vue-router`.)
- **Capability theo cửa sổ:** `"windows": ["main", "diff-*"]` — quyền chỉ áp dụng cho cửa sổ có label khớp; `*` là ký tự đại diện. Cửa sổ không khớp sẽ không gọi được API nào.
- **Sự kiện cửa sổ trong Rust:** `on_window_event(|window, event| …)` nhận mọi sự kiện của mọi cửa sổ; `matches!(event, WindowEvent::Destroyed)` lọc đúng lúc cửa sổ bị đóng hẳn.
- **`Promise` bọc sự kiện:** `new WebviewWindow(...)` trả về ngay; kết quả tạo cửa sổ báo qua sự kiện `tauri://created` / `tauri://error` → gói thành Promise để `await` và hiện lỗi nếu có.

### Kiểm tra
| Thao tác | Kết quả mong đợi |
|---|---|
| Double-click một file Modified | Cửa sổ mới "tên file — moon-git", diff toàn bộ file |
| Trong khi cửa sổ diff mở, click file khác / Stage ở cửa sổ chính | Cửa sổ chính vẫn dùng bình thường |
| Double-click lại cùng file | Không mở thêm; cửa sổ cũ lên trước |
| Double-click file khác | Thêm một cửa sổ nữa |
| Stage file ở cửa sổ chính, rồi bấm vào cửa sổ diff của nó | Nút chuyển sang **Staged** |
| Commit/discard file (bằng terminal) rồi bấm vào cửa sổ diff | "This file has no changes any more…" |
| Đóng cửa sổ chính | App thoát, các cửa sổ diff đóng theo |

## Ghi chú cho sau này
- Khác biệt chỉ ở ký tự xuống dòng (CRLF/LF) không hiện ra: Monaco chuẩn hóa xuống dòng khi tải văn bản.
- File không phải UTF-8 (vd. Shift-JIS, Latin-1) có thể hiện ký tự `�`.
- Chưa stage được từng đoạn (hunk) từ diff — có thể làm sau trên nền này.
- Ẩn các vùng không đổi (`hideUnchangedRegions` của Monaco) có thể bật sau nếu muốn gọn hơn.
