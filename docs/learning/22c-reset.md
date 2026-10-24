# Bước 22c — Reset branch về một commit (Soft / Mixed / Hard)

*Bước này bạn yêu cầu thêm (2026-10-23). CLAUDE.md không có.*

## Trạng thái
- `cargo test`: 173/173 pass. Thêm 7 test:
  - xem trước: số commit bị bỏ lại, số commit đã push, số file có thay đổi; target là commit hiện tại;
  - đếm commit đã push bị bỏ lại (trước/sau khi push);
  - Soft: thay đổi của các commit bị bỏ ở dạng **đã stage**, thay đổi cũ vẫn chưa stage;
  - Mixed: tất cả thành **chưa stage** (file mới thành untracked);
  - Hard: file trở về như commit đó, **file untracked vẫn còn**;
  - target là tên branch;
  - chỉ nhận ref đang tồn tại hoặc commit id (`--hard`, `HEAD~1`… bị từ chối, file không bị đụng tới).
- `cargo clippy`: sạch. `pnpm test`: 17/17. `pnpm build`: OK.
- Giao diện: đã commit (2026-10-24) khi bạn chuyển sang bước 22d.

## Mục tiêu
**History → chuột phải một commit → Reset Current Branch to Here…** mở hộp thoại:
- tiêu đề "Reset "main" to 1234abcd";
- "N commit(s) are left behind" (không còn trên branch nữa);
- nếu có commit đã push trong đó: cảnh báo lần push sau cần `--force`;
- ba lựa chọn, mặc định **Mixed**.

| Kiểu | Branch | Phần đã stage (index) | File (working tree) | Dùng khi |
|---|---|---|---|---|
| **Soft** | dời về commit đó | giữ, cộng thêm thay đổi của các commit bị bỏ | giữ | Gộp vài commit cuối thành một: Soft về trước chúng, rồi commit lại |
| **Mixed** | dời về | đặt lại theo commit đó | giữ (thay đổi thành **chưa stage**) | Làm lại vài commit cuối: chọn lại file nào vào commit nào |
| **Hard** | dời về | đặt lại | **đặt lại** | Bỏ hẳn mọi thứ sau commit đó, kể cả thay đổi chưa commit |

- **Hard**:
  - nút Reset chuyển màu đỏ;
  - bấm Reset → hỏi **thêm một lần**: "The uncommitted changes in N file(s) are lost for good."
- **File untracked** không bao giờ bị xóa, kể cả với Hard (giống git).

## Lệnh git

| Việc | Lệnh |
|---|---|
| Commit bị bỏ lại | `git rev-list --count <target>..HEAD` |
| Trong đó chưa push | `git rev-list --count <target>..HEAD --not --remotes` |
| Reset | `git reset --quiet --soft\|--mixed\|--hard <commit id đầy đủ> --` |

### Ghi chú
- **Không dùng được `--end-of-options` với `git reset` của git 2.33** (đã thử: "option '--end-of-options' must come before non-option arguments"). Vì vậy:
  - app đổi target thành **commit id đầy đủ** (40 ký tự hex) trước, rồi mới truyền cho git. Một id như vậy không bao giờ bắt đầu bằng `-`, nên không thể bị hiểu là option;
  - `--` ở cuối: không có đường dẫn file nào theo sau.
- **Target vẫn được kiểm tra như merge/rebase** (`resolve_source`): chỉ ref đang tồn tại hoặc commit id hex.
- **Không reset khi**:
  - đang merge hoặc rebase dở (reset sẽ âm thầm hủy chúng; dùng Abort);
  - có file conflict;
  - HEAD detached.
- **Commit "bị bỏ lại" chưa mất ngay**: git vẫn giữ chúng một thời gian trong **reflog** (`git reflog`), và `ORIG_HEAD` trỏ vào vị trí trước khi reset. Lấy lại được bằng terminal, nhưng không dễ. Vì vậy hộp thoại nói rõ số commit bị bỏ lại.
- **Thay đổi chưa commit mất với Hard thì không lấy lại được**: git không lưu chúng ở đâu cả. Đó là lý do phải hỏi hai lần.

## Luồng dữ liệu

```
History → chuột phải commit → Reset Current Branch to Here…
  └─ branchMenus askReset → getResetPreview(repoId, id) → invoke("get_reset_preview")
       └─ Rust git::reset::preview → rev-parse, rev-list --count (2 lần), read_status → ResetPreview
  └─ dialogs.askReset({ preview, submit }) → <ResetDialog> (Soft / Mixed / Hard)
       └─ (Hard) ask("… lost for good") → submit(mode)
            └─ resetCurrent(repoId, id, mode) → invoke("reset_current")
                 └─ Rust git::reset::reset → kiểm tra → git reset --quiet --<mode> <id> --
            └─ OK → onStatus → RepoView applyStatus → Changes / History tải lại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/reset.rs` (mới) | `preview`, `reset` + 7 test |
| `src-tauri/src/commands/reset.rs` (mới), `lib.rs` | `get_reset_preview`, `reset_current` |
| `src-tauri/src/models.rs` | `ResetMode`, `ResetPreview` |
| `src/types/reset.ts`, `src/api/reset.ts` (mới) | kiểu + wrapper |
| `src/components/ResetDialog.vue` (mới) | hộp thoại 3 lựa chọn, xác nhận lần 2 cho Hard |
| `src/stores/dialogs.ts`, `src/App.vue` | `resetRequest`, gắn `ResetDialog` |
| `src/components/branchMenus.ts` | mục **Reset Current Branch to Here…**, `askReset` |

## Khái niệm mới

### Rust
- **Ép kiểu số bằng `as`**: `(a.len() + b.len()) as u32`. `len()` trả `usize` (cỡ con trỏ, 64 bit), model dùng `u32`. `as` đổi thẳng, giống `(int) x` trong Java; với số file trong repo thì không lo tràn.
- **Hàm test nhận slice của struct** `fn paths(files: &[FileChange]) -> Vec<&str>`: kết quả mượn chuỗi từ `files`. Rust tự hiểu lifetime (chỉ có một tham chiếu đầu vào), không cần ghi `'a`.

### Vue
- **Hỏi thêm ngay trong component**: `ResetDialog` tự gọi `ask(...)` trước khi chạy, vì chỉ nó biết người dùng chọn Hard. Code mở hộp thoại (`branchMenus`) không cần biết chuyện này.
- **Class theo lựa chọn** `:class="{ destructive: mode === 'hard' }"`: nút đổi màu ngay khi chọn Hard, báo hiệu trước khi bấm.

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử (có hộp thoại mới).

Dùng một repo thử (ví dụ `branchy`), tạo vài commit thử trên một branch riêng.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | History → chuột phải commit cách HEAD 2 commit → **Reset Current Branch to Here…** | "Reset "…" to xxxxxxxx", "2 commit(s) are left behind", **Mixed** được chọn |
| 2 | Chọn **Soft** → Reset | History: 2 commit cuối biến mất khỏi branch; Changes: file của chúng **Staged** |
| 3 | Commit lại (một commit gộp) | Một commit thay cho hai |
| 4 | Làm lại bước 1 với **Mixed** | Các file đó ở dạng chưa stage (Modified / Untracked) |
| 5 | Sửa một file → mở lại hộp thoại → chọn **Hard** | Nút Reset đỏ; dòng đỏ "Your uncommitted changes in 1 file(s) are lost." |
| 6 | Reset → hộp thoại thứ hai → **Cancel** | Không có gì đổi |
| 7 | Reset → **Reset and Discard** | File về như commit đó; Changes chỉ còn file untracked (nếu có) |
| 8 | Repo có remote: reset về trước một commit đã push | Dòng cảnh báo "N of them are already pushed…" |
| 9 | Đang có merge dở → Reset | Lỗi "A merge or rebase is in progress…" |
| 10 | Lấy lại commit sau khi reset (terminal): `git reset --hard ORIG_HEAD` | Branch về như trước bước reset gần nhất |
