# Bước 31a — Menu bar của MoonGit

*Bạn yêu cầu (2026-10-27). Bước 31b sẽ thêm Branch → Checkout… (hộp thoại có ô tìm).*

## Trạng thái
- `cargo test`: 208/208, `cargo clippy`: sạch (Rust chỉ thêm plugin clipboard). `pnpm test`: 35/35. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK. Đã commit.

## Menu

| Menu | Mục (phím tắt) |
|---|---|
| **MoonGit** | About MoonGit · Services · Hide MoonGit ⌘H · Hide Others ⌥⌘H · Show All · Quit ⌘Q |
| **Repository** | Add Local Repository… ⌘O · Clone Repository… ⇧⌘O · New Group… · Remove from List · Close Window ⌘W |
| **Edit** | Undo ⌘Z · Redo ⇧⌘Z · Cut ⌘X · Copy ⌘C · Paste ⌘V · Select All ⌘A · **Copy Repository Path** ⌥⌘C |
| **View** | Changes ⌘1 · History ⌘2 · Stashes ⌘3 · Refresh ⌘R · Theme ▸ (System ✓, Light / Dark mờ) · Enter Full Screen ⌃⌘F · *Reload Interface (dev)* ⇧⌘R |
| **Remote** | Fetch ⇧⌘F · Pull ⇧⌘L · Push ⇧⌘P · Force Push… · Add Remote… |
| **Local** | Commit… ⌘K · Stage · Unstage · Discard… · Stash… ⇧⌘S · Apply Stash · Apply and Drop Stash · Drop Stash… |
| **Branch** | New Branch… ⇧⌘N · Rename Current Branch… |
| **Window** | Minimize ⌘M · Zoom · Bring All to Front · (danh sách cửa sổ đang mở, do macOS thêm) |

- **Tên app trong menu MoonGit**: nhãn mặc định của macOS lấy theo tên file chương trình (`moongit`), nên ra "Quit Moongit". Giờ ba mục About / Hide / Quit ghi rõ **MoonGit** (bạn yêu cầu). About vẫn hiện phiên bản (đọc bằng `getVersion()`).
  - Riêng **chữ đậm cạnh logo Apple** (tên menu ứng dụng) do macOS lấy từ tên app: khi chạy dev (chương trình chưa đóng gói) nó vẫn là "moongit"; bản build (`MoonGit.app`) sẽ là "MoonGit".
- **Repository** = menu File cũ (bạn đổi tên). Add / Clone đưa repo mới vào **cùng nhóm với repo đang chọn** (chưa chọn → cấp ngoài cùng).
- **Remove from List**: như menu chuột phải ở sidebar, chỉ bỏ khỏi danh sách, không xóa thư mục, nên không hỏi.
- **Theme**: chỗ đặt sẵn cho dark mode sau này. Hiện chỉ có System (theo macOS, như trước giờ).
- **Reload Interface (dev)**: chỉ có khi chạy `pnpm tauri dev`. ⌘R giờ là Refresh, nên nạp lại giao diện dùng **⇧⌘R**.
- **Remote / Local** tách như SmartGit. Apply / Apply and Drop / Drop chỉ bật ở tab Stashes khi đang chọn một stash; Stage / Unstage / Discard chỉ bật ở tab Changes khi có file phù hợp đang chọn.
- Mục nào **không làm được lúc đó thì mờ**, theo đúng điều kiện của nút tương ứng (chưa chọn repo, đang chạy thao tác khác, không có gì để push, đang rebase dở…).

## Cách làm: menu "bấm hộ" nút

```
macOS menu bar ─ click / phím tắt ─▶ commands.run("remote.push")      (appMenu.ts)
                                          │
                       store menuCommands: Map<CommandId, { run, enabled }>
                                          │  (SyncButtons đã đăng ký "remote.push")
                                          ▼
                              SyncButtons.push()  ← cùng hàm với nút Push
```

1. **`src/stores/menuCommands.ts`** (store mới): bảng `tên lệnh → { run, enabled }`.
   - Component nào làm được việc gì thì **tự đăng ký** khi xuất hiện, **tự gỡ** khi biến mất: `useMenuHandlers({...})`.
   - Ví dụ: `SyncButtons` đăng ký Fetch/Pull/Push/Force Push; `RepoView` đăng ký tab, Refresh, Commit, Stage…, stash, branch; `BranchPanel` đăng ký Add Remote.
2. **`src/appMenu.ts`**: dựng menu bằng API JS của Tauri (`Menu.new`, `Submenu.new`, `MenuItem.new`, `PredefinedMenuItem.new`) rồi `setAsAppMenu()`. Mỗi mục của MoonGit chỉ gọi `commands.run(id)`.
3. **Bật / mờ tự động**: một `watchEffect` đọc `commands.isEnabled(id)` cho mọi mục. Khi trạng thái đổi (chọn file, chuyển tab, bắt đầu push…), Vue chạy lại nó và gọi `item.setEnabled(...)`, chỉ cho mục thực sự thay đổi.
4. Menu được cài một lần, trong `App.vue` (cửa sổ chính). Cửa sổ diff / Conflict Solver dùng chung menu bar (trên macOS menu bar thuộc về app, không thuộc từng cửa sổ). Copy / Paste / Close Window tác động lên cửa sổ đang focus.

**Không sửa logic git nào**: mọi thao tác vẫn đi qua đúng hàm của nút, nên vẫn có đủ hỏi xác nhận, tiến trình, Cancel, hiện lỗi.

## Thư viện mới: plugin clipboard của Tauri
- **`@tauri-apps/plugin-clipboard-manager`** (JS) + **`tauri-plugin-clipboard-manager`** (Rust): plugin **chính thức** của Tauri để ghi chữ vào clipboard của hệ điều hành. Dùng cho **Copy Repository Path**.
- Vì sao không dùng `navigator.clipboard` của trình duyệt: WebKit chỉ cho ghi clipboard ngay sau một cú click/phím **trong trang web**. Bấm menu bar là sự kiện của macOS, không phải của trang, nên `navigator.clipboard.writeText` có thể bị từ chối. Plugin ghi thẳng từ Rust, không bị giới hạn này.
- Quyền: chỉ thêm `clipboard-manager:allow-write-text` (ghi chữ) trong `capabilities/default.json`. App **không** có quyền đọc clipboard.

## File

| File | Việc |
|---|---|
| `src/stores/menuCommands.ts` (mới) | Bảng lệnh: `register`, `isEnabled`, `run`; `useMenuHandlers` cho component |
| `src/appMenu.ts` (mới) | Dựng menu bar, đồng bộ bật/mờ; lệnh chung (Add / Clone / New Group / Remove, Copy Repository Path) |
| `src/App.vue` | Cài menu khi mở cửa sổ chính |
| `src/views/RepoView.vue` | Đăng ký lệnh View / Local / Branch |
| `src/components/SyncButtons.vue` | Đăng ký lệnh Remote (Fetch, Pull, Push, Force Push) |
| `src/components/BranchPanel.vue` | Đăng ký Add Remote |
| `src/components/sidebarMenus.ts`, `branchMenus.ts` | `export` `addLocalRepo`, `newGroup`, `askRename` để menu dùng lại |
| `src-tauri/src/lib.rs`, `Cargo.toml`, `package.json`, `capabilities/default.json` | Plugin clipboard |

## Khái niệm mới

### Vue / TS
- **Đăng ký / gỡ theo vòng đời component** (`useMenuHandlers` gọi `register`, rồi `onUnmounted` gỡ): giống đăng ký listener rồi `removeListener` trong Java, nhưng tự động theo component.
- **`shallowReactive(new Map())`**: Vue theo dõi việc thêm/xóa khóa của `Map` (đủ để menu biết lệnh nào đang có người làm), không đi sâu vào từng object bên trong.
- **`watchEffect`**: như `watch`, nhưng không cần khai báo nguồn: Vue tự ghi nhận những giá trị reactive được đọc bên trong và chạy lại khi chúng đổi. Hợp khi một hàm đọc rất nhiều thứ (mọi điều kiện của mọi mục menu).
- **`import.meta.env.DEV`**: `true` khi chạy dev, `false` khi build; Vite bỏ hẳn phần code dev khỏi bản build.

### Tauri
- **Menu bar từ JavaScript**: `menu.setAsAppMenu()`. Mục "predefined" (Copy, Hide, Quit…) do macOS xử lý, không cần `action`.
- **Accelerator** (`"CmdOrCtrl+Shift+F"`): phím tắt của mục menu. `CmdOrCtrl` = ⌘ trên macOS, Ctrl trên Linux/Windows.

## Cách thử

App dev cần build lại phần Rust (plugin mới). Nếu bạn đang chạy `pnpm tauri dev`, nó tự build lại. Nếu không thấy menu mới, tắt rồi chạy lại.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Nhìn menu bar | MoonGit · Repository · Edit · View · Remote · Local · Branch · Window |
| 2 | Chưa chọn repo | Remove from List, Copy Repository Path, Remote, Local, Branch mờ |
| 3 | Chọn repo → ⌥⌘C → dán vào đâu đó | Đường dẫn thư mục repo |
| 4 | ⌘2, ⌘3, ⌘1 | Chuyển tab History, Stashes, Changes |
| 5 | Sửa một file → ⌘R | Danh sách file cập nhật |
| 6 | Chọn file chưa stage → Local → Stage | File được stage; ở tab History thì mục mờ |
| 7 | ⌘K | Hộp thoại Commit |
| 8 | ⇧⌘F | Fetch chạy, có tiến trình như bấm nút |
| 9 | Tab Stashes, chọn một stash → Local → Apply and Drop Stash | Như nút Apply and Drop |
| 10 | Branch → New Branch… | Hộp thoại tạo branch |
| 11 | View → Theme | System ✓; Light / Dark mờ |
| 12 | ⇧⌘R (dev) | Giao diện nạp lại |
| 13 | Gõ chữ trong ô commit message: ⌘C / ⌘V / ⌘Z | Vẫn hoạt động |
