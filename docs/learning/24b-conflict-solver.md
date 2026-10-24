# Bước 24b — Cửa sổ Conflict Solver (3 cột, giống SmartGit)

*Theo yêu cầu của bạn (2026-10-24): thay cho hai nút Take Ours / Take Theirs trên thanh công cụ (24a), và thay cho ý "mở công cụ so sánh ngoài" trong kế hoạch cũ.*

## Trạng thái
- `cargo test`: 193/193 pass. Thêm 2 test:
  - đọc 3 phiên bản (tổ tiên / ours / theirs) và file có dấu conflict; file bị một bên xóa → `missingSide`;
  - lưu kết quả; lưu + đánh dấu đã giải quyết (từ chối khi còn dấu, trừ khi `force`); chỉ ghi được file đang conflict.
- `pnpm test`: 28/28 (thêm 11 test cho hai module mới `lineDiff` và `conflictBlocks`). `cargo clippy`: sạch. `pnpm build`: OK; Monaco vẫn nằm ở chunk riêng, cửa sổ chính không tải nó khi khởi động.
- Giao diện: bạn đã xác nhận (2026-10-24). Đã commit.

## Mục tiêu
**Double-click một file Conflicted** (hoặc chọn nó → nút **Resolve…**, hoặc chuột phải → **Resolve…**) mở cửa sổ riêng:

```
┌ ‹ Conflict 1 of 3 › │ Take Left  Take Right  Take Both │        Unsaved changes  [Save] [Save & Mark Resolved] ┐
├─ Ours: main ────────────┬─ Result (editable) ──────────┬─ Theirs: feature ────────┤
│ (chỉ đọc)               │ <<<<<<< HEAD        (đỏ)      │ (chỉ đọc)                │
│ dòng main đã sửa (xanh  │ phần của main     (xanh dương)│ dòng feature đã sửa      │
│  dương)                 │ =======             (đỏ)      │  (xanh lá)               │
│                         │ phần của feature  (xanh lá)   │                          │
│                         │ >>>>>>> feature     (đỏ)      │                          │
└─────────────────────────┴───────────────────────────────┴──────────────────────────┘
```

- **Cột trái / phải**: toàn bộ file của mỗi phía, chỉ đọc. Các dòng phía đó **đã sửa so với tổ tiên chung** được tô màu (trái xanh dương, phải xanh lá).
- **Cột giữa**: file hiện tại, **sửa trực tiếp được**. Mỗi khối conflict được tô:
  - dòng dấu: đỏ (đậm hơn ở khối đang chọn);
  - phần của ours: xanh dương;
  - phần của theirs: xanh lá;
  - phần tổ tiên (chỉ khi `merge.conflictStyle = diff3`): xám.
- **‹ ›**: đi tới conflict trước / sau. Cột giữa và **hai cột bên đều cuộn tới đúng chỗ** đó.
- **Take Left / Take Right / Take Both**: thay khối đang chọn (khối có con trỏ, hoặc khối vừa đi tới) bằng phần trái, phần phải, hoặc cả hai (trái trước, phải sau). **⌘Z** hoàn tác được.
- Gõ trực tiếp cũng được. Danh sách conflict tự cập nhật khi bạn xóa các dòng dấu.
- **Save** (⌘S): ghi file, chưa đánh dấu đã giải quyết.
- **Save & Mark Resolved**: ghi file, stage nó (= đã giải quyết), rồi đóng cửa sổ. Còn dấu conflict → hỏi "Mark as resolved anyway?". Cửa sổ chính tự tải lại khi được focus.
- **Đóng cửa sổ khi chưa lưu** → hỏi "Close without saving?".

Những file cửa sổ này **không** xử lý được (cửa sổ báo và chỉ cách khác):
- file nhị phân / không phải UTF-8 / quá lớn;
- một bên đã **xóa** file (không có gì để ghép theo dòng).

Với chúng, chuột phải trong cửa sổ chính → **Take Ours** / **Take Theirs** (các mục này vẫn còn trong menu chuột phải, chỉ bỏ khỏi thanh công cụ).

## Thanh công cụ của cửa sổ chính (thay đổi so với 24a)
Khi chọn file Conflicted: chỉ còn **Resolve…** (mở solver) và **Mark Resolved** (khi bạn đã sửa file bằng editor khác). Take Ours / Take Theirs chỉ còn trong menu chuột phải.

## Dữ liệu đến từ đâu

| Cột | Nội dung | Lệnh |
|---|---|---|
| (để so sánh) | tổ tiên chung, stage 1 | `git cat-file blob :1:<file>` |
| Trái | ours, stage 2 | `git cat-file blob :2:<file>` |
| Phải | theirs, stage 3 | `git cat-file blob :3:<file>` |
| Giữa | file trong working tree (có dấu) | đọc file |
| Lưu | ghi file; Mark Resolved = `git add --all -- <file>` (bước 24a) | |

- Dùng lại `read_blob` / `read_working_file` của bước 8 (giới hạn 5 MB, nhận diện file nhị phân), giờ là `pub(crate)`.
- **Không phải UTF-8 → không cho sửa**: nếu file dùng mã hóa khác (ví dụ Latin-1), hiển thị rồi lưu lại sẽ làm hỏng các byte. App coi như file nhị phân.
- **Chỉ file đang conflict** mới đọc/ghi được qua hai command này (kiểm tra với status).

## Hai module TypeScript mới (`src/conflict/`)

### `conflictBlocks.ts`: tìm khối conflict
- `findConflicts(lines)`: tìm các khối `<<<<<<<` … (`|||||||` …) `=======` … `>>>>>>>`.
  - Dấu phải đúng 7 ký tự, đứng riêng hoặc theo sau là dấu cách.
  - Khối thiếu dấu bị bỏ qua.
  - Dòng `=======` lẻ (tiêu đề Markdown) không bị nhầm.
- `resolvedLines(lines, block, "ours" | "theirs" | "both")`: các dòng thay cho khối.
- `sideView(lines, blocks, side)`: file "theo cách một phía nhìn": mọi khối thay bằng phần của phía đó, kèm vị trí bắt đầu của từng khối. Dùng để tìm chỗ tương ứng ở cột bên.

### `lineDiff.ts`: so sánh theo dòng (thuật toán Myers)
`diffLines(a, b)` trả về các vùng khác nhau (`Hunk`: dòng `a[aStart..aEnd)` được thay bằng `b[bStart..bEnd)`). `mapLine(hunks, line)` cho biết dòng `line` của `a` nằm ở đâu trong `b`.

**Myers (1986)**, thuật toán mặc định của `git diff`:
- Hình dung một lưới, `a` theo chiều ngang, `b` theo chiều dọc. Đi **sang phải** = xóa một dòng của `a`, đi **xuống** = thêm một dòng của `b`, đi **chéo** = giữ một dòng hai bên giống nhau (miễn phí).
- Cần tìm đường từ góc trên-trái tới góc dưới-phải với **ít bước ngang/dọc nhất**.
- Với d = 0, 1, 2…: trên mỗi đường chéo k = x − y, ghi lại điểm xa nhất đi được với đúng d bước. Đi chéo hết mức có thể (gọi là "snake"). Dừng ở d đầu tiên chạm góc cuối.
- Lưu lại mảng trạng thái của mỗi vòng, rồi **đi ngược** để biết những dòng nào được giữ.
- Bỏ phần đầu và phần cuối giống nhau trước (nhanh với file chỉ sửa ít). Quá `maxEdits` dòng khác nhau thì gộp phần giữa thành một vùng.

Dùng ở hai chỗ:
1. **Tô màu cột bên**: `diffLines(tổ tiên, ours)` → các dòng ours đã sửa.
2. **Cuộn cột bên tới conflict đang chọn**: `diffLines(sideView(giữa, "ours"), ours)` rồi `mapLine` vị trí khối. (Không so trực tiếp được, vì cột giữa có các dòng dấu và các thay đổi đã tự ghép từ phía kia.)

## Luồng dữ liệu

```
Double-click file Conflicted → RepoView openInWindow → openConflictWindow(repoId, …, sides.ours, sides.theirs)
  └─ WebviewWindow "conflict-<hash>" → index.html?view=conflict&repoId&path&ours&theirs
       └─ main.ts → ConflictWindow.vue → getConflictVersions → invoke("get_conflict_versions")
            └─ Rust git::conflict::versions → read_blob(:1: :2: :3:), read_working_file → ConflictVersions
       └─ <ConflictSolver> (tải lười, Monaco) → 3 editor
            ├─ findConflicts → tô màu cột giữa; diffLines(base, side) → tô cột bên
            ├─ ‹ › → goTo → sideView + diffLines + mapLine → cuộn ba cột
            ├─ Take Left/Right/Both → resolvedLines → pushEditOperations (⌘Z được)
            └─ Save / Save & Mark Resolved → emit("save") → saveConflictResult → invoke("save_conflict_result")
                 └─ Rust git::conflict::save → ghi file → (mark) resolve(Resolved) → git add
                 └─ conflictMarkers → ask("anyway?") → force → đóng cửa sổ
Cửa sổ chính được focus lại → refresh → file thành Staged
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/conflict.rs` | `versions`, `save`, `check_conflicted` + 2 test |
| `src-tauri/src/git/diff.rs` | `Version`, `read_blob`, `read_working_file`, `is_binary` thành `pub(crate)` |
| `src-tauri/src/models.rs` | `ConflictVersions` |
| `src-tauri/src/commands/stage.rs`, `lib.rs` | `get_conflict_versions`, `save_conflict_result` |
| `src-tauri/capabilities/default.json` | cửa sổ `conflict-*`; quyền `close` / `destroy` (đóng cửa sổ, hỏi trước khi đóng) |
| `src-tauri/src/git/{merge,rebase,cherry_pick,pull,stash,discard}.rs` | thông báo conflict: "Double-click a file to resolve it" |
| `src/conflict/lineDiff.ts` (+ test, mới) | Myers diff, `mapLine` |
| `src/conflict/conflictBlocks.ts` (+ test, mới) | tìm / thay khối conflict, `sideView` |
| `src/components/ConflictSolver.vue` (mới) | 3 editor Monaco, tô màu, điều hướng, Take, Save |
| `src/views/ConflictWindow.vue` (mới) | gốc của cửa sổ: tải dữ liệu, lưu, hỏi khi đóng |
| `src/windows.ts`, `src/main.ts` | `openConflictWindow`, `view=conflict` |
| `src/api/stage.ts`, `src/types/status.ts` | wrapper + kiểu |
| `src/views/RepoView.vue` | double-click file conflict → solver; nút **Resolve…**; bỏ Take Ours/Theirs khỏi thanh công cụ |

## Khái niệm mới

### TS / thuật toán
- **Myers diff** (ở trên). Tách thành module thuần, có test, giống `laneLayout` bước 16b.
- **`Int32Array`** và **`.slice()`** để chụp lại trạng thái mỗi vòng: mảng số nguyên cỡ cố định, nhanh và gọn hơn mảng thường.
- **`as const` cho mảng tuple** trong `for (const [editor, side, lines] of [...] as const)`: TypeScript giữ đúng kiểu từng phần tử (editor, `"ours"`, mảng dòng), không gộp thành một kiểu chung.

### Vue / Monaco
- **`defineExpose` + template ref tới component**: `ConflictSolver` mở hàm `markSaved` cho cửa sổ cha gọi (`solver.value?.markSaved(content)`). Thường cha–con nói chuyện bằng props/emit; `defineExpose` dành cho lúc cha cần "ra lệnh" một việc.
- **`<style>` không scoped** cho class Monaco tự gắn (`cs-ours-line`…): `scoped` chỉ áp lên phần tử do component render, mà các dòng là do Monaco tạo.
- **Decorations** (`createDecorationsCollection`, `isWholeLine`): tô nền cả dòng mà không đổi nội dung.
- **`pushEditOperations`**: sửa nội dung theo cách người dùng gõ, nên **⌘Z** hoàn tác được (khác `setValue`, vốn xóa lịch sử undo).
- **`addCommand(KeyMod.CtrlCmd | KeyCode.KeyS, …)`**: phím tắt riêng cho editor (⌘ trên macOS, Ctrl trên Linux/Windows).
- **`onCloseRequested`** của Tauri: chặn đóng cửa sổ bằng `event.preventDefault()` khi người dùng chọn "Keep Editing".

## Cách thử

App dev đã build lại và đang chạy (capabilities thay đổi nên cần build lại Rust). Bấm ⌘R ở cửa sổ chính trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Merge gây conflict (hai branch sửa cùng dòng, tốt nhất 2–3 chỗ khác nhau) | Thanh đỏ "… double-click a file to resolve it." |
| 2 | Double-click file Conflicted | Cửa sổ "Resolve … — repo", 3 cột: Ours: main / Result / Theirs: feature; "Conflict 1 of N" |
| 3 | Nhìn cột trái / phải | Các dòng mỗi bên đã sửa được tô (xanh dương / xanh lá) |
| 4 | **›** | Ba cột cuộn tới conflict kế tiếp |
| 5 | **Take Right** | Khối thay bằng phần phải; "Conflict 1 of N-1"; hiện "Unsaved changes" |
| 6 | ⌘Z | Khối quay lại |
| 7 | Sửa tay một khối (bỏ các dòng dấu) | Số conflict giảm |
| 8 | Đóng cửa sổ khi chưa lưu | Hỏi "Close without saving?" → Keep Editing |
| 9 | **Save & Mark Resolved** khi còn conflict | Hỏi "Mark as resolved anyway?" |
| 10 | Giải quyết hết → **Save & Mark Resolved** | Cửa sổ đóng; cửa sổ chính: file thành Staged, thanh chuyển xanh → Commit Merge |
| 11 | Conflict "một bên xóa file" (hoặc file ảnh) → double-click | Cửa sổ báo không ghép theo dòng được, chỉ cách dùng Take Ours / Theirs ở menu chuột phải |
| 12 | Rebase / cherry-pick gây conflict → double-click | Tên cột: "Ours: xxxxxxxx (rebasing onto)" / "Theirs: your commit …" |
