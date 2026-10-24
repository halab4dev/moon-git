# Bước 22d — Discard: bỏ thay đổi của file (file mới vào Thùng rác)

*Bước này bạn yêu cầu thêm (2026-10-24). CLAUDE.md không có.*

## Trạng thái
- `cargo test`: 179/179 pass. Thêm 6 test:
  - file sửa (đã stage, chưa stage, hay cả hai) về đúng như HEAD;
  - file bị xóa (chưa stage / đã `git rm`) quay lại;
  - file untracked và file mới đã stage bị bỏ đi, thư mục trống bị dọn, thư mục còn file khác thì giữ;
  - file đổi tên quay về tên cũ;
  - repo chưa có commit;
  - chỉ nhận đường dẫn có trong status (file không đổi, `../outside.txt`, `*.txt` bị từ chối, không file nào bị đụng); không discard file conflict.
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: bạn đã xác nhận (2026-10-24). Đã commit.

## Mục tiêu
Tab **Changes** → chọn file → nút **Discard…** (thanh công cụ) hoặc chuột phải → **Discard…**.

Luôn hỏi trước, kèm danh sách file (tối đa 10 tên, còn lại "… and N more"), và nói rõ:
- "The changes to N tracked file(s) are lost for good.": thay đổi của file tracked **không lấy lại được**;
- "N new or untracked file(s) are moved to the Trash.": file mới **vào Thùng rác**, lấy lại được.

| Trạng thái (cột State) | Discard làm gì |
|---|---|
| Modified, Staged, Staged Modified, Removed, Missing, Type Changed | Về đúng như HEAD, cả index lẫn file |
| Added, Added Modified, Intent to Add | Bỏ khỏi index, file vào **Thùng rác** |
| Untracked | File vào **Thùng rác** |
| Renamed | Tên mới vào Thùng rác, tên cũ quay lại từ HEAD |
| Copied | Bản copy vào Thùng rác (bản gốc không bị đổi) |
| Conflicted | Không cho (bước 24: chọn ours/theirs) |

## Lệnh git

| Việc | Lệnh |
|---|---|
| File HEAD có | `git --literal-pathspecs restore --source=HEAD --staged --worktree -- <file>…` |
| File mới trong index | `git --literal-pathspecs rm --cached --quiet -- <file>…`, rồi chuyển vào Thùng rác |

### Ghi chú
- **`restore --source=HEAD --staged --worktree`**: lấy bản trong HEAD, ghi vào **cả** index (`--staged`) **và** file trên đĩa (`--worktree`). Như vậy cả phần đã stage lẫn chưa stage đều mất. Bước 7 chỉ dùng `restore --staged` (unstage, file giữ nguyên).
- **`rm --cached`**: chỉ bỏ khỏi index, file vẫn trên đĩa, để app tự chuyển nó vào Thùng rác (`git rm` thường sẽ xóa hẳn).
- **Chỉ đường dẫn có trong status được xử lý**: Rust đọc status trước, rồi xếp từng đường dẫn vào một trong ba nhóm (`Plan`: bỏ khỏi index / restore / vào Thùng rác). Đường dẫn không có trong status → lỗi `NotFound`, **không file nào bị đụng**. Nhờ vậy frontend không thể làm Rust xóa file ngoài repo (`../…`) hay theo pattern (`*.txt`).
- **Thư mục trống được dọn**: bỏ `new/deep/u.txt` thì `new/deep` và `new` trống. git không hiện thư mục trống, nên chúng chỉ là rác. `fs::remove_dir` chỉ xóa được thư mục **trống**, nên thư mục còn file khác chắc chắn được giữ.
- **Repo chưa có commit**: không có HEAD, nên mọi file đều coi là "mới": bỏ khỏi index + vào Thùng rác.
- **Chia lô 100 đường dẫn** mỗi lệnh git, như stage/unstage.

## Thư viện mới: `trash` (5.2.9)
- **Dùng để làm gì:** chuyển file vào Thùng rác của hệ điều hành (macOS: Finder Trash; Ubuntu: chuẩn freedesktop `~/.local/share/Trash`; Windows: Recycle Bin), thay vì xóa hẳn.
- **Vì sao chọn:** Rust không có sẵn chức năng này, mỗi hệ điều hành làm một kiểu. `trash` lo cả ba, chỉ dùng một hàm `trash::delete(path)`. Nó chỉ kéo thêm 2 package (`trash`, `urlencoding`); phần macOS dùng chung thư viện Tauri đã có.
- **Test không đụng Thùng rác thật**: `discard_with` nhận "cách bỏ file" làm tham số. App truyền `trash::delete`, test truyền `fs::remove_file`. Nhờ vậy chạy test không làm đầy Thùng rác của bạn.

## Luồng dữ liệu

```
Chọn file → Discard… → RepoView discardSelected
  └─ ask("Discard the changes of N file(s)? • a.txt … lost for good … moved to the Trash")
       └─ discardChanges(repoId, paths) → invoke("discard_changes")
            └─ Rust git::discard::discard → discard_with(…, trash::delete)
                 ├─ read_status → plan(): unindex / restore / remove (chỉ đường dẫn có trong status)
                 ├─ git rm --cached …   (file mới trong index)
                 ├─ git restore --source=HEAD --staged --worktree …
                 └─ trash::delete(file) + dọn thư mục trống
            └─ read_status → RepoStatus mới
  └─ run() → status mới → bảng file + diff tải lại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/Cargo.toml` | thêm `trash = "5.2.9"` |
| `src-tauri/src/git/discard.rs` (mới) | `discard`, `discard_with`, `plan`, `remove_empty_folders` + 6 test |
| `src-tauri/src/commands/stage.rs`, `lib.rs` | `discard_changes` |
| `src/api/stage.ts` | `discardChanges` |
| `src/views/RepoView.vue` | nút + menu **Discard…**, hộp thoại xác nhận có danh sách file |

## Khái niệm mới

### Rust
- **Truyền hành vi vào hàm** (`remove: impl Fn(&Path) -> Result<(), AppError>`): hàm nhận một closure để biết phải làm gì với file. Code thật và test truyền hai closure khác nhau. Giống truyền một `Consumer<Path>` hay một interface trong Java, để test thay bằng bản giả.
- **`is_some_and(|f| …)`**: `Option` có giá trị **và** giá trị đó thỏa điều kiện. Gọn hơn `matches!(x, Some(f) if …)`.
- **`filter` + `and_then` trên `Option`**: `staged.filter(|f| f.kind == Renamed).and_then(|f| f.orig_path.clone())`: chỉ lấy tên cũ khi đúng là rename.
- **`symlink_metadata().is_ok()`**: kiểm tra file có tồn tại mà **không đi theo** symlink (`exists()` sẽ báo `false` cho symlink trỏ vào chỗ không còn).
- **Struct riêng cho "kế hoạch"** (`Plan`): tách bước "quyết định làm gì" (thuần, dễ đọc) khỏi bước "làm" (chạy git, đụng file). Mọi kiểm tra xảy ra trước khi bất cứ thứ gì thay đổi.

### TS
- **`Set` để tra nhanh nhóm trạng thái** (`NEW_FILE_STATES.has(row.state)`).
- **Câu hỏi xác nhận ghép từ nhiều phần** (`notes` chỉ thêm câu khi có loại file đó).

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Sửa một file tracked → chọn → **Discard…** | Hỏi, có tên file, "The changes to 1 tracked file(s) are lost for good." |
| 2 | **Cancel** | Không có gì đổi |
| 3 | **Discard** | File về như HEAD; biến mất khỏi Changes |
| 4 | Stage một file sửa, rồi sửa thêm (Staged Modified) → Discard | Cả hai phần đều mất |
| 5 | Tạo thư mục mới có một file (untracked) → Discard | "… moved to the Trash."; file **có trong Thùng rác** của Finder; thư mục trống cũng biến mất |
| 6 | Tạo file mới, Stage (Added) → Discard | File vào Thùng rác; không còn trong Changes |
| 7 | Xóa một file tracked (Missing) → Discard | File quay lại |
| 8 | Đổi tên file trong terminal (`git mv a b`) → Discard dòng Renamed | Tên cũ quay lại, tên mới vào Thùng rác |
| 9 | Chọn nhiều file (⌘A) → chuột phải → **Discard…** | Danh sách file, tối đa 10 tên + "… and N more" |
| 10 | Có file Conflicted trong lựa chọn | Nút Discard bỏ qua file đó; nếu chỉ chọn file conflict thì nút mờ |
