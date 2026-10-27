# Bước 28 — Bỏ dòng branch dưới tên repo, sắp lại thanh công cụ, thêm icon (thanh công cụ + sidebar)

*Bạn yêu cầu (2026-10-26): panel Branches ở sidebar đã hiện branch, không cần lặp lại ở header.*

## Trạng thái
- Chỉ đổi frontend. `pnpm test`: 30/30. `pnpm build`: OK (kiểm tra kiểu: không còn biến/import thừa).
- Giao diện: bạn đã xem, OK. Đã commit.

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

## Thanh công cụ: nút server sang trái (bạn yêu cầu thêm)
Trước: `Stage  Unstage  Discard…  Stash…  ……  Fetch  Pull  Push ▾  Refresh` (nút server dồn sang phải).

Giờ:

```
Fetch  Pull  Push ▾  Refresh  |  Stage  Unstage  Discard…  Stash…
```

- Dấu **|** là đường kẻ dọc mảnh (class `separator` đã có, dùng như chỗ ngăn nút Resolve… ở bước 24).
- Tab **History** không có nút thao tác file nên không có dấu |. Tab **Stashes** còn nút Stash…, nên vẫn có dấu |.
- Dòng ghi chú ngắn của Fetch/Pull/Push (vd. "Already up to date", "Force pushed") chuyển ra **sau** các nút. Nếu để trước như cũ, các nút sẽ bị đẩy sang phải mỗi khi ghi chú hiện ra.
- Lúc Fetch/Pull/Push đang chạy, cụm bên trái thành dòng tiến trình + Cancel. Cụm bên phải có thể xê dịch một chút (dòng tiến trình tối đa 280px).

## Icon cho các nút (bạn yêu cầu thêm)
Bản đầu là icon nét nhỏ 14px, cùng màu chữ, đứng trước chữ. Bạn muốn **to hơn, có màu**, nên giờ thanh công cụ giống SmartGit / IDE: **icon màu 24px ở trên, chữ nhỏ ở dưới**. Nút phẳng (không viền), chỉ hiện nền và viền khi di chuột.

| Nút | Icon | Màu |
|---|---|---|
| Fetch | đám mây, mũi tên xuống (chỉ tải về, file không đổi) | xanh dương |
| Pull | mũi tên xuống vào khay | xanh lá |
| Push | mũi tên lên ra khỏi khay | cam |
| Refresh | mũi tên vòng | xanh ngọc |
| Stage / Unstage | hình tròn đặc có dấu + / − trắng | xanh lá / vàng cam |
| Discard… | mũi tên quay lại (undo) | đỏ |
| Stash… | hộp lưu trữ | tím |
| Resolve… | cây bút | vàng đậm |
| Mark Resolved | hình tròn đặc có ✓ trắng | xanh lá |
| Apply (tab Stashes) | **hộp stash** (giống icon Stash…), mũi tên **xuống** đi ra từ hộp: thay đổi được lấy ra, hộp vẫn còn | tím |
| Apply and Drop (tab Stashes) | y hệt Apply + **dấu ✕ đỏ** ở góc: lấy ra rồi xóa stash | tím + đỏ |
| Drop… (tab Stashes) | thùng rác | đỏ |

- **Kiểu "duotone"**: nét đậm màu chính, phía sau là vùng tô cùng màu nhưng trong suốt 82% (`fill-opacity: 0.18`). Trông có màu mà không chói, và hợp cả nền sáng lẫn nền tối (đã xem thử cả hai).
- **Không thêm thư viện icon**: 10 icon tự vẽ bằng SVG trong `AppIcon.vue`. Mỗi icon = một màu + danh sách hình, mỗi hình thuộc một kiểu: `fill` (vùng tô nhạt), `line` (nét), `solid` (tô đặc), `white` (nét trắng trên hình đặc).
- Màu gán qua `style="color: …"` trên `<svg>`, các hình dùng `currentColor`. Một chỗ đổi màu là đổi cả icon.
- Nút bị disabled vẫn mờ cả icon lẫn chữ (CSS `opacity: 0.5` đã có cho mọi nút).
- Kiểu nút mới là class dùng chung **`button.tool`** trong `styles.css`, vì nút nằm ở hai component (`RepoView` và `SyncButtons`). Nút ▾ cạnh Push là một `tool` hẹp, cao bằng các nút khác.
- `<AppIcon name="push" />` chỉ nhận đúng các tên đã khai báo (`IconName`); gõ sai tên thì `pnpm build` báo lỗi.

## Tab Stashes: Apply / Apply and Drop / Drop lên thanh công cụ (bạn yêu cầu thêm)
- Trước: ba nút Apply / Pop / Drop… nằm trong tab Stashes, ngay trên danh sách, không có icon. Giờ chúng nằm trên thanh công cụ chung, có icon:

```
Fetch  Pull  Push ▾  Refresh  |  Apply  Apply and Drop  Drop…
```

- **Pop đổi tên thành "Apply and Drop"** (bạn yêu cầu), cả trên nút lẫn trong menu chuột phải: nói thẳng nó làm gì. Lệnh git bên dưới vẫn là `git stash pop` (nếu có conflict thì stash được giữ lại).
- **Bỏ nút Stash… khỏi tab Stashes** (bạn yêu cầu): tạo stash chỉ còn ở tab Changes. Dòng "No stashes" giờ ghi rõ: dùng "Stash…" ở tab Changes.
- Ba nút mới tác động lên stash **đang chọn** trong danh sách; chuột phải một stash vẫn có menu Apply / Pop / Drop… như cũ.
- Nút ở `RepoView`, còn danh sách và việc chạy lệnh vẫn ở `StashView`. `StashView` "mở" ra hai thứ bằng `defineExpose`: `hasSelection` (có stash đang chọn không) và `run(action)`. `RepoView` giữ một ref tới nó (`useTemplateRef("stashView")`) và gọi `stashView?.run('pop')` cho nút Apply and Drop. Cùng cách đã dùng cho Conflict Solver ở bước 24b.

Bản đầu dùng bảng kẹp giấy (Apply, mũi tên xuống) và hộp có mũi tên **lên** (Pop). Bạn thấy không rõ nghĩa và hai mũi tên ngược chiều, mâu thuẫn nhau. Bản hiện tại theo một quy tắc: **cùng một hộp, cùng chiều mũi tên**, khác nhau đúng một chi tiết là việc stash có bị xóa hay không (dấu ✕). Màu đỏ của dấu ✕ được cố định (kiểu hình `danger`), không theo màu chính của icon.

## Icon trong danh sách repo ở sidebar (bạn yêu cầu thêm)
- **Repo**: trước là ký tự `◆`. Giờ là ô vuông bo góc **màu cam** (màu quen thuộc của git) với **nhánh trắng**: một đường thẳng + một nhánh rẽ, ba chấm tròn.
- **Nhóm (folder)**: to hơn (15px → 18px) và **hai tông vàng tươi**: thân sau vàng đậm, nắp trước vàng sáng. Gập: folder đóng; mở: folder mở (nắp nghiêng), như cũ.
- Icon repo đứng thẳng hàng với icon folder cùng cấp: dưới cột ▸/▾ của nhóm có một ô trống cho repo.
- Cả ba icon nằm trong `AppIcon.vue` như các icon khác (`folder`, `folderOpen`, `repo`). Để vẽ được icon hai màu, mỗi hình giờ có thể mang **màu riêng** (`paint(d, màu)`), không bắt buộc dùng màu chính của icon. Thêm kiểu `whiteSolid` cho các chấm trắng.
- Màu folder giờ cố định, nên biến CSS `--folder` (màu folder cũ cho chế độ sáng/tối) bị bỏ. Đã xem thử trên nền sáng, nền tối và dòng đang chọn (nền xanh): đều rõ.

## File đã sửa

| File | Việc |
|---|---|
| `src/components/BranchStatus.vue` | Xóa |
| `src/views/RepoView.vue` | Bỏ `<BranchStatus>`, `openBranchMenu`, `branchCallbacks`; thêm dòng cảnh báo detached |
| `src/views/RepoView.vue` (thanh công cụ) | `SyncButtons` + Refresh lên đầu, rồi `separator`; bỏ `spacer`; icon cho các nút |
| `src/components/SyncButtons.vue` | ghi chú chuyển ra sau các nút; icon Fetch/Pull/Push |
| `src/components/AppIcon.vue` (mới) | icon SVG màu theo tên (gồm apply / pop / drop) |
| `src/views/StashView.vue` | bỏ hàng nút riêng; `defineExpose({ hasSelection, run })` |
| `src/styles.css` | kiểu nút `button.tool` (icon trên, chữ dưới); cỡ icon 18px trong cây sidebar; bỏ `.folder-icon`, `--folder` |
| `src/components/GroupItem.vue`, `RepoItem.vue` | icon folder / repo bằng `AppIcon` |
| `src/components/branchMenus.ts` | Bỏ `showBranchMenu`, `mergeMenu`, `rebaseMenu` (chỉ menu header dùng); `showBranchItemMenu` thêm "Upstream of …" cho branch local (đọc `listRefs` để biết upstream hiện tại và các remote branch) |

## Khái niệm
- **`currentColor` trong SVG**: màu nét/tô = thuộc tính `color` của phần tử cha. Đặt `color` một lần trên `<svg>`, mọi hình bên trong theo màu đó.
- **Hàm nhỏ tạo object** (`const line = (d) => ({ d, kind: "line" })`): bảng icon đọc gọn như một danh sách, thay vì lặp `{ d: …, kind: … }` khắp nơi.
- **Union type làm danh sách tên hợp lệ** (`type IconName = "fetch" | "pull" | …`) + `Record<IconName, string[]>`: TS bắt buộc bảng `PATHS` có đủ mọi tên, và prop `name` chỉ nhận các tên đó. Giống một `enum` + `EnumMap` của Java.

Ghi chú thêm: `vue-tsc` (trong `pnpm build`) báo lỗi **biến khai báo mà không dùng** (`noUnusedLocals` của TS strict). Nhờ vậy, sau khi xóa một component, mọi code chỉ phục vụ nó đều lộ ra để dọn.

## Cách thử
Bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Chọn một repo | Dưới tên repo chỉ còn đường dẫn, rồi đến các tab |
| 2 | Chuột phải một branch local trong panel | Có **Upstream of "…"** ▸: các remote branch (✓ ở upstream hiện tại) + Stop Tracking |
| 3 | Chọn một remote branch khác làm upstream | Di chuột lên branch: `→` remote mới; ↑↓ tính theo remote mới |
| 4 | History → chuột phải một commit → checkout ở trạng thái detached (terminal: `git checkout <id>`) → ↻ | Dòng cảnh báo "HEAD detached at …" màu vàng/cam |
| 5 | Checkout lại một branch | Dòng cảnh báo biến mất |
| 6 | Tab Changes | `Fetch Pull Push ▾ Refresh \| Stage Unstage Discard… Stash…` |
| 7 | Tab History | Chỉ `Fetch Pull Push ▾ Refresh`, không có dấu \| |
| 8 | Bấm Fetch khi đã cập nhật | Nút không bị đẩy đi; ghi chú hiện sau ▾ |
| 9 | Nhìn các nút | Icon màu 24px trên, chữ dưới; di chuột thì nút có nền + viền; nút mờ thì icon cũng mờ |
| 10 | Chọn file conflict | Resolve… nền xanh, icon bút vàng |
| 11 | Đổi macOS sang Dark mode | Icon vẫn rõ, vùng tô nhạt không chói |
| 12 | Tab Stashes | Thanh công cụ: `… Refresh \| Apply  Apply and Drop  Drop…`; không có Stash…; không còn hàng nút trên danh sách |
| 13 | Chưa có stash nào | Apply / Apply and Drop / Drop… mờ; dòng "No stashes" chỉ sang tab Changes |
| 14 | Chọn một stash → Apply and Drop (nút, hoặc chuột phải) | Thay đổi về working tree, stash biến mất khỏi danh sách |
| 15 | Drop… | Hỏi xác nhận trước khi xóa |
| 16 | Sidebar, danh sách repo | Mỗi repo có ô cam với nhánh trắng trước tên; nhóm có folder vàng to hơn, đóng/mở theo trạng thái gập |
| 17 | Chọn một repo (nền xanh) | Icon vẫn rõ |
