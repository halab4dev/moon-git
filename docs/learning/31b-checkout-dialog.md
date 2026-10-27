# Bước 31b — Branch → Checkout… (hộp thoại có ô tìm)

*Bạn yêu cầu (2026-10-27), tiếp theo menu bar ở bước 31a.*

## Trạng thái
- Chỉ đổi frontend. `pnpm test`: 38/38 (thêm 3 test cho bộ lọc). `pnpm build`: OK.
- Giao diện: bạn đã thử, OK. Đã commit.

## Giao diện
**Branch → Checkout…** (⇧⌘C):

```
Check Out Branch
┌──────────────────────────────────────────┐
│ fea log                                   │  ← gõ để lọc
├──────────────────────────────────────────┤
│ LOCAL BRANCHES                            │
│ ✓ main                            ↑1 ↓2  │  ← branch hiện tại: đậm, không chọn được
│   feature/login                           │  ← dòng được chọn (nền xanh)
│ REMOTE BRANCHES                           │
│   origin/feature/login                    │
└──────────────────────────────────────────┘
                         [Cancel] [Check Out]
```

- **Lọc**: mỗi từ gõ vào phải có trong tên, không cần đúng thứ tự, không phân biệt hoa thường. `fea log` tìm được `feature/login`, `origin/feature/login`.
- **↑ / ↓** chuyển dòng (bỏ qua branch hiện tại), **Enter** checkout, **Esc** đóng. Bấm chuột một dòng cũng checkout luôn.
- Local trước, rồi remote; mỗi nhóm sắp theo tên như panel Branches (`feature-2` trước `feature-10`). Branch local có ↑↓ / `gone` bên phải.
- **Remote branch**: tạo branch local cùng tên theo dõi nó (như double-click ở panel, bước 19a). Nếu đã có branch local cùng tên theo dõi đúng nó thì chỉ chuyển sang.
- **Lỗi** (vd. có thay đổi chưa commit bị đè: "The branch can't be checked out: it changes 1 file…") hiện **trong hộp thoại**, hộp thoại không đóng.
- Thành công: hộp thoại đóng; panel Branches chuyển ✓, danh sách file cập nhật.
- Danh sách branch lấy theo lần **fetch cuối** (không tự fetch).
- Menu mờ khi đang chạy thao tác khác.

## Luồng dữ liệu

```
⇧⌘C → commands.run("branch.checkout") → RepoView: checkoutOpen = true
  └─ <CheckoutDialog> onMounted → listBranches → invoke("list_branches")   (bước 25)
       gõ → filterBranches(list, query)   (branchFilter.ts, có test)
       Enter → props.submit(fullName) → RepoView.checkoutFromDialog
            └─ checkoutBranch → invoke("checkout_branch") → git switch …   (bước 19a)
            ← lỗi → hiện trong hộp thoại
            ← OK → applyStatus → đóng; panel Branches tự tải lại
```

## File

| File | Việc |
|---|---|
| `src/components/branchFilter.ts` (mới) + `.test.ts` | `filterBranches`: gộp local + remote, lọc theo từ, sắp tên |
| `src/components/CheckoutDialog.vue` (mới) | Hộp thoại: ô tìm, danh sách, phím ↑/↓/Enter/Esc |
| `src/views/RepoView.vue` | `checkoutOpen`, `checkoutFromDialog`, lệnh `branch.checkout` |
| `src/stores/menuCommands.ts`, `src/appMenu.ts` | Lệnh + mục menu Checkout… ⇧⌘C |

## Khái niệm mới
- **`words.every(word => label.includes(word))`**: "mọi từ đều khớp". `every` của mảng giống `allMatch` của Java stream.
- **Tách logic lọc ra file `.ts` thuần**: không dính tới Vue nên test được bằng Vitest mà không cần dựng giao diện. Cùng cách với `laneLayout.ts`, `commitTime.ts`.
- **`do … while` để bỏ qua phần tử**: di chuyển ít nhất một bước, rồi đi tiếp chừng nào còn gặp branch hiện tại.
- **`position: sticky` cho tiêu đề nhóm**: "LOCAL BRANCHES" / "REMOTE BRANCHES" dính ở đầu khung khi cuộn.

## Cách thử

Bấm ⇧⌘R (Reload Interface).

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Branch → Checkout… (hoặc ⇧⌘C) | Hộp thoại, con trỏ ở ô tìm; dòng đầu (không phải branch hiện tại) được chọn |
| 2 | Gõ một phần tên | Danh sách lọc theo |
| 3 | ↑ / ↓ | Dòng chọn di chuyển, bỏ qua branch ✓ |
| 4 | Enter trên một branch local | Checkout; hộp thoại đóng; ✓ trong panel chuyển sang |
| 5 | Chọn một remote branch chưa có bản local | Tạo branch local theo dõi nó |
| 6 | Sửa một file mà branch kia cũng sửa → checkout | Lỗi hiện trong hộp thoại, không đóng |
| 7 | Gõ chữ không khớp gì | "No branch matches …" |
| 8 | Esc / Cancel / bấm ra ngoài | Đóng, không đổi gì |
