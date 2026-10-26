# Bước 29 — Nút Commit… trên thanh công cụ, hộp thoại Commit (Commit / Commit and Push)

*Bạn yêu cầu (2026-10-26).*

## Trạng thái
- Chỉ đổi frontend. `pnpm test`: 30/30. `pnpm build`: OK.
- Giao diện: chờ bạn thử. Chưa commit.

## Thay đổi
- **Bỏ khung commit ở dưới màn hình** (ô message + Amend + nút Commit, component `CommitBox`). Vùng danh sách file + diff giờ cao thêm.
- **Nút Commit…** (icon chấm tròn trên một đường, như commit trong graph) trên thanh công cụ, ngay sau **Stash…**, chỉ ở tab Changes:

```
Fetch  Pull  Push ▾  Refresh  |  Stage  Unstage  Discard…  Stash…  Commit…
```

- Nút mờ (kèm lý do khi di chuột) khi đang chạy thao tác khác, hoặc khi đang rebase / cherry-pick dở (lúc đó Continue trên thanh màu sẽ commit).
- Bấm vào → **hộp thoại Commit**:

```
Commit
┌─────────────────────────────────────────────┐
│ Commit message (first line = summary)        │
│                                              │
└─────────────────────────────────────────────┘
☐ Amend last commit     2 staged files
                 [Cancel] [Commit and Push] [Commit]
```

- **Commit** (nút chính, hoặc ⌘Enter): commit như trước.
- **Commit and Push**: commit, đóng hộp thoại, rồi **push qua đúng nút Push** của thanh công cụ. Nhờ vậy có đủ: hỏi trước khi push branch mới lên server, dòng tiến trình, nút Cancel, và lỗi hiện như khi bấm Push.
- Đang merge: thay cho Amend là hai lựa chọn **Merge commit / Simple commit** (như bước 21b); nút ghi "Commit Merge" / "Commit Merge and Push".
- Amend: nút ghi "Amend" / "Amend and Push". Nếu commit cũ **đã push**, "Amend and Push" bị mờ: commit đã sửa thay cho commit trên server, push thường sẽ bị từ chối, phải dùng **Force Push** (▾ cạnh Push). Force push vẫn là một bước riêng, có xác nhận.
- HEAD detached: chỉ Commit, "… and Push" mờ.
- Lỗi khi commit (vd. chưa khai báo tên/email) hiện **ngay trong hộp thoại**, hộp thoại không đóng.
- **Bản nháp được giữ**: đóng hộp thoại (Cancel, Esc, bấm ra ngoài) không mất message đã gõ; mở lại vẫn còn. Commit xong thì xóa.

## Luồng dữ liệu

```
Commit… → commitOpen = true → <CommitDialog v-model:message v-model:amend v-model:simple :submit="commitFromDialog">
  Commit / Commit and Push → props.submit(push)
    └─ RepoView.commitFromDialog → commitChanges → invoke("commit_changes") → Rust (như bước 9)
         ← lỗi → trả message → hộp thoại hiện
         ← OK → applyStatus, xóa nháp, đóng hộp thoại
               push? → nextTick → syncButtons.push()  (SyncButtons.defineExpose({ push }))
                         └─ hỏi branch mới? → pushRepo (tiến trình, Cancel) → finished → tải lại status
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src/components/CommitBox.vue` → `CommitDialog.vue` (`git mv`) | Thành hộp thoại: thêm `submit(push)`, lỗi trong hộp thoại, nút "… and Push", tự focus ô message |
| `src/components/SyncButtons.vue` | `defineExpose({ push })` |
| `src/components/AppIcon.vue` | icon `commit` |
| `src/views/RepoView.vue` | nút Commit…, `commitBlocker`, `commitFromDialog`, gắn `CommitDialog`; bỏ khung commit ở dưới |

## Khái niệm mới

### Vue
- **Truyền hàm qua prop thay cho emit** (`:submit="commitFromDialog"`): hộp thoại cần **kết quả** (lỗi hay thành công) để biết có hiện lỗi hay không. `emit` không trả về gì, còn gọi một hàm prop thì `await` được. Cùng ý với các `…Request.submit` trong store `dialogs` (bước 4, 21…).
- **`watch(..., { immediate: true })`**: chạy ngay lần đầu, không chờ giá trị đổi. Cần vì Amend có thể vẫn đang tích từ lần mở trước: hộp thoại mới mở phải tự kiểm tra commit cũ đã push chưa.
- **`nextTick()` trước khi gọi sang component khác**: sau commit, status mới vừa được gán, nhưng Vue chỉ cập nhật props của `SyncButtons` ở lượt render kế tiếp. Chờ `nextTick` để Push thấy branch đã có commit mới.

## Cách thử

Bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Tab Changes | Không còn khung commit ở dưới; thanh công cụ có **Commit…** sau Stash… |
| 2 | Stage một file → Commit… | Hộp thoại, con trỏ ở ô message; "1 staged file" |
| 3 | Gõ message → Esc → mở lại | Message vẫn còn |
| 4 | Commit (hoặc ⌘Enter) | Hộp thoại đóng; file biến mất khỏi danh sách; History có commit mới; branch ↑1 |
| 5 | Stage → Commit… → **Commit and Push** | Commit, rồi thanh công cụ hiện "Pushing…"; xong ↑0 |
| 6 | Trên branch mới chưa có trên server → Commit and Push | Commit, rồi hỏi "Push New Branch" như nút Push |
| 7 | Không stage gì → Commit… | Nút Commit mờ, "Nothing staged: stage files first" |
| 8 | Tích Amend (commit cũ đã push) | Cảnh báo "already pushed"; "Amend and Push" mờ, di chuột thấy lý do |
| 9 | Đang merge (sau Merge to Working Tree) | Hai lựa chọn Merge commit / Simple commit, message điền sẵn |
| 10 | Đang rebase dở | Commit… mờ: "A rebase is in progress…" |
