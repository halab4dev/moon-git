# Bước 28 — Bỏ dòng branch dưới tên repo

*Bạn yêu cầu (2026-10-26): panel Branches ở sidebar đã hiện branch, không cần lặp lại ở header.*

## Trạng thái
- Chỉ đổi frontend. `pnpm test`: 30/30. `pnpm build`: OK (kiểm tra kiểu: không còn biến/import thừa).
- Giao diện: chờ bạn thử. Chưa commit.

## Đã bỏ
Dòng `⎇ main ▾ → origin/main ↑1 ↓2` dưới tên repo (component `BranchStatus.vue`, đã xóa) và menu của nó (`showBranchMenu`). Mọi thông tin của dòng đó đã có ở panel Branches:

| Dòng cũ | Giờ ở đâu |
|---|---|
| Branch hiện tại | ✓ + chữ đậm trong Local Branches |
| `→ origin/main` | Di chuột lên branch |
| ↑ahead ↓behind, `(gone)` | Bên phải mỗi branch local |
| Menu: checkout, remote branches | Double-click / chuột phải trong panel |
| Menu: New / Rename / Merge / Rebase / Delete | Chuột phải trong panel |
| Menu: **Upstream of "main" ▸** | **Mới chuyển sang** chuột phải một branch local trong panel (trước đây chỉ có ở menu header) |

## Giữ lại một trường hợp: HEAD detached
Khi HEAD detached (checkout một commit, hoặc đang rebase), không có branch nào được ✓ trong panel. Commit mới lúc đó không thuộc branch nào, và rất dễ mất. Vì vậy chỉ trong trường hợp này, dưới tên repo hiện một dòng màu cảnh báo:

> HEAD detached at 1a2b3c4: new commits here belong to no branch. Check out a branch, or create one from History.

## File đã sửa

| File | Việc |
|---|---|
| `src/components/BranchStatus.vue` | Xóa |
| `src/views/RepoView.vue` | Bỏ `<BranchStatus>`, `openBranchMenu`, `branchCallbacks`; thêm dòng cảnh báo detached |
| `src/components/branchMenus.ts` | Bỏ `showBranchMenu`, `mergeMenu`, `rebaseMenu` (chỉ menu header dùng); `showBranchItemMenu` thêm "Upstream of …" cho branch local (đọc `listRefs` để biết upstream hiện tại và các remote branch) |

## Khái niệm
Không có khái niệm mới. Ghi chú: `vue-tsc` (trong `pnpm build`) báo lỗi **biến khai báo mà không dùng** (`noUnusedLocals` của TS strict). Nhờ vậy, sau khi xóa một component, mọi code chỉ phục vụ nó đều lộ ra để dọn.

## Cách thử
Bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Chọn một repo | Dưới tên repo chỉ còn đường dẫn, rồi đến các tab |
| 2 | Chuột phải một branch local trong panel | Có **Upstream of "…"** ▸: các remote branch (✓ ở upstream hiện tại) + Stop Tracking |
| 3 | Chọn một remote branch khác làm upstream | Di chuột lên branch: `→` remote mới; ↑↓ tính theo remote mới |
| 4 | History → chuột phải một commit → checkout ở trạng thái detached (terminal: `git checkout <id>`) → ↻ | Dòng cảnh báo "HEAD detached at …" màu vàng/cam |
| 5 | Checkout lại một branch | Dòng cảnh báo biến mất |
