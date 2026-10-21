# Bước 4 — Nhóm / nhóm con repo

## Trạng thái
- `cargo test`: 23/23 pass (thêm 10 test cho logic nhóm, 1 test đọc file version 1). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21).

## Mục tiêu
- Sidebar thành **cây**: nhóm → nhóm con → repo, bấm vào nhóm để thu gọn/mở rộng.
- Tạo / đổi tên / xóa nhóm; thêm repo thẳng vào một nhóm; chuyển repo sang nhóm khác.
- Lưu vào cùng file `repos.json` (lên **version 2**), file version 1 cũ vẫn đọc được.
- Kéo-thả: để sau (CLAUDE.md cho phép).

## Quyết định thiết kế

| Vấn đề | Chọn | Vì sao |
|---|---|---|
| Lưu cây thế nào | **Danh sách phẳng** + `parentId` / `groupId` | Di chuyển = đổi một trường, không phải bóc ra cắm vào cây lồng nhau. Frontend tự dựng cây để vẽ |
| Xóa nhóm thì nội dung đi đâu | Nhóm con và repo bên trong **lên một cấp** (vào nhóm cha) | Không mất gì; không có gì bị xóa khỏi ổ đĩa. Vẫn hỏi xác nhận vì thao tác đổi cấu trúc cây |
| Tên trùng | Hai nhóm **cùng cha** không được trùng tên (không phân biệt hoa thường); khác cha thì được | Tránh hai nhóm "Work" cạnh nhau gây nhầm |
| Menu thao tác | **Menu chuột phải native** (`@tauri-apps/api/menu`, có sẵn, không cần thêm thư viện) | Trông và hoạt động như app desktop thật; không phải tự viết component menu |
| Nhập tên | Hộp thoại tự viết `NameDialog.vue` | `window.prompt()` không dùng được trong webview của Tauri |
| Hỏi xác nhận xóa | `ask()` của plugin dialog (đã có từ bước 3) | Hộp thoại native, chỉ cần thêm quyền `dialog:allow-ask` |
| Command trả về gì | Thao tác thay đổi trả về **toàn bộ danh sách mới** (`RepoList`) | Frontend không phải lặp lại luật (ví dụ "xóa nhóm thì con lên một cấp") — Rust là nơi duy nhất giữ luật |
| Trạng thái thu gọn | Chỉ trong bộ nhớ, mở app là mở rộng hết | Đơn giản; có thể lưu sau nếu cần |

## Định dạng file mới (version 2)
```json
{
  "version": 2,
  "groups": [
    { "id": "18e0…a1", "name": "Work", "parentId": null },
    { "id": "18e0…b2", "name": "Client A", "parentId": "18e0…a1" }
  ],
  "repos": [
    { "id": "18e0…88", "name": "moon-git", "path": "/Users/halab/Projects/MoonGit/moon-git", "groupId": "18e0…b2" }
  ]
}
```
File version 1 (không có `groups`, repo không có `groupId`) được đọc như: không có nhóm, mọi repo ở cấp cao nhất. Lần ghi tiếp theo sẽ ghi thành version 2. Không cần viết code chuyển đổi riêng — nhờ `#[serde(default)]` (xem dưới).

## File đã tạo / sửa

**Rust**
| File | Việc |
|---|---|
| `models.rs` | Thêm `RepoGroup`, `RepoList`; `RepoEntry` thêm `group_id` |
| `storage/repo_list.rs` | **Mới.** Luật thay đổi danh sách (`impl RepoList`): thêm/xóa/chuyển repo, tạo/đổi tên/xóa nhóm, kiểm tra tên. Không đụng file → test thuần, nhanh. 10 test |
| `storage/repos.rs` | `RepoStore` giờ giữ `RepoList`; mọi thay đổi đi qua một hàm `update()` chung. File version 2 |
| `commands/repo.rs` | `add_repo` nhận thêm `group_id`; thêm `move_repo` |
| `commands/group.rs` | **Mới.** `create_group`, `rename_group`, `delete_group` |
| `error.rs` | Thêm `InvalidInput`, `NotFound` |
| `capabilities/default.json` | Thêm `dialog:allow-ask` |

**Frontend**
| File | Việc |
|---|---|
| `types/repo.ts` | `RepoGroup`, `RepoList`, `groupId` |
| `api/repo.ts`, `api/group.ts` | Wrapper cho các command mới |
| `stores/repoTree.ts` | **Mới.** `buildTree()` dựng cây từ danh sách phẳng; `listGroupOptions()` tạo danh sách "Work / Client A" cho menu Move To |
| `stores/repos.ts` | Thêm groups, tree, trạng thái thu gọn, các action nhóm |
| `stores/dialogs.ts` | **Mới.** Store mở hộp thoại nhập tên từ bất kỳ đâu |
| `components/NameDialog.vue` | **Mới.** Hộp thoại nhập tên, hiện lỗi ngay trong hộp thoại |
| `components/sidebarMenus.ts` | **Mới.** Các menu native và hành động của chúng |
| `components/GroupItem.vue` | **Mới.** Một dòng nhóm + nội dung bên trong (đệ quy) |
| `components/RepoItem.vue` | **Mới.** Một dòng repo |
| `components/AppSidebar.vue` | Vẽ cây; nút + mở menu |
| `App.vue` | Gắn `NameDialog` một lần cho cả app |
| `styles.css` | Style chung: nút, ô nhập, dòng cây, khung hộp thoại |

Nút **×** trên dòng repo ở bước 3 đã được thay bằng mục **Remove from List** trong menu chuột phải, để mọi thao tác nằm một chỗ.

## Luồng dữ liệu: tạo nhóm con
```
Chuột phải vào nhóm "Work"
  GroupItem @contextmenu → showGroupMenu(group)       (sidebarMenus.ts)
    Menu.new({...}).popup()                            ← menu native của hệ điều hành
  Chọn "New Subgroup…" → newGroup("work-id")
    dialogs.askName({ title, submit })                 ← lưu yêu cầu vào store
  NameDialog thấy nameRequest ≠ null → hiện ra
  Gõ "Client A", Enter → request.submit("Client A")
    store.createGroup("Client A", "work-id")
      invoke("create_group", { name, parentId })
        commands::group::create_group(name, parent_id, store)
          RepoStore::create_group → update(|list| list.create_group(...))
            khóa → copy → RepoList::create_group (kiểm tra cha tồn tại, tên hợp lệ, không trùng)
                 → ghi file → thay bản trong bộ nhớ
      ← RepoList mới
    applyList(list) → groups/repos đổi → computed `tree` tính lại → sidebar vẽ lại
    revealGroup("work-id") → mở rộng "Work" để thấy nhóm mới
  submit trả null → hộp thoại đóng
  (Nếu Rust trả lỗi, ví dụ tên trùng → submit trả message → hộp thoại giữ nguyên, hiện chữ đỏ)
```

## Khái niệm Rust mới

### `impl` ở file khác với `struct`
`RepoList` khai báo trong `models.rs`, nhưng các method của nó viết trong `storage/repo_list.rs`. Rust cho phép viết khối `impl` ở bất kỳ module nào trong cùng crate. Nhờ vậy `models.rs` chỉ chứa dữ liệu, còn luật nghiệp vụ nằm riêng. (Java bắt buộc method nằm trong class; gần nhất là tách ra một `RepoListService`.)

### `&mut self` và `iter_mut()`
- `fn create_group(&mut self, ...)` — method **được sửa** đối tượng. `&self` chỉ được đọc. Compiler kiểm tra: không thể gọi method `&mut self` trên một giá trị chỉ được mượn để đọc. Java không phân biệt hai loại này.
- `self.groups.iter_mut().find(...)` — duyệt và lấy **tham chiếu sửa được** tới phần tử, để `group.name = name` sửa thẳng trong vector.

### Generic function + closure làm tham số: `update`
```rust
fn update<T>(&self, change: impl FnOnce(&mut RepoList) -> Result<T, AppError>) -> Result<T, AppError> {
    let mut data = self.lock()?;
    let mut updated = data.clone();
    let result = change(&mut updated)?;   // chạy thay đổi trên bản sao
    write_file(&self.file, &updated)?;    // ghi được mới…
    *data = updated;                      // …thay bản thật
    Ok(result)
}
// dùng:
self.update(|list| list.add_repo(info, group_id))
```
- `<T>`: kiểu trả về tùy thao tác (`RepoEntry` cho thêm repo, `RepoList` cho thao tác khác) — giống generic method Java `<T> T update(Function<RepoList, T> change)`.
- `impl FnOnce(&mut RepoList) -> Result<T, AppError>`: "một closure nhận danh sách sửa được, trả về Result". `FnOnce` = closure chỉ được gọi một lần (cho phép nó "lấy luôn" biến như `info`).
- Lợi ích: 6 thao tác dùng chung một quy trình **khóa → sao chép → sửa → ghi → thay**. Thất bại ở bước nào thì cả bộ nhớ lẫn file đều không đổi (có test `failed_change_leaves_list_untouched`).

### `Option` trong thực tế
- `ok_or_else(|| not_found("Group"))` — đổi `Option` thành `Result`: `Some(x)` → `Ok(x)`, `None` → `Err(...)`. Rồi dùng `?` được.
- `g.parent_id.as_deref() == Some(id)` — `parent_id` là `Option<String>`, `id` là `&str`. `as_deref()` đổi thành `Option<&str>` để so sánh được mà không phải tạo `String` mới.
- `#[serde(default)]` — khi đọc JSON mà thiếu trường đó, dùng giá trị mặc định (`Vec` rỗng, `None`). Đây là cách đọc file version 1.
- `#[derive(Default)]` trên `RepoList` → `RepoList::default()` = danh sách rỗng. ≈ constructor không tham số.

### `retain` và `any`
- `self.groups.retain(|g| g.id != id)` — giữ lại các phần tử thỏa điều kiện, xóa phần còn lại, sửa tại chỗ. ≈ `list.removeIf(g -> g.id.equals(id))` (điều kiện ngược).
- `.iter().any(|g| ...)` ≈ `stream().anyMatch(...)`.

### `cargo fmt`
Công cụ định dạng code chuẩn của Rust (≈ Prettier / google-java-format). Đã chạy một lần; nó chỉ đổi xuống dòng/thụt lề.

## Khái niệm Vue / TS mới

### Component đệ quy
`GroupItem.vue` dùng `<GroupItem>` bên trong chính nó cho các nhóm con. Trong `<script setup>`, component tự gọi được mình bằng tên file. Mỗi cấp truyền `depth + 1` để thụt lề sâu hơn. Đây là cách vẽ cây có độ sâu bất kỳ.

### `watch`
```ts
watch(() => dialogs.nameRequest, async (request) => { ... });
```
Theo dõi một giá trị; mỗi khi nó **đổi**, chạy hàm. Khác `computed` (tính ra giá trị mới), `watch` dùng để **làm việc gì đó** (reset form, focus ô nhập). ≈ listener `PropertyChangeListener` trên một thuộc tính.

### Template ref + `nextTick`
```ts
const input = ref<HTMLInputElement | null>(null);   // <input ref="input">
await nextTick();     // chờ Vue vẽ xong DOM
input.value?.focus();
```
- `ref="input"` trong template: Vue gán phần tử DOM thật vào biến `input`. Dùng khi cần gọi API của DOM (focus, select).
- `nextTick()`: Vue cập nhật DOM theo lô, không ngay lập tức. Ô nhập nằm trong `v-if`, nên phải chờ một nhịp mới tồn tại.
- `?.` (optional chaining): chỉ gọi nếu không null.

### Store gọi từ file `.ts` thường
`sidebarMenus.ts` không phải component nhưng vẫn gọi `useRepoStore()` được — store dùng được ở bất kỳ đâu sau khi `app.use(createPinia())` đã chạy. Nhờ đó hành động của menu native (chạy ngoài Vue) vẫn cập nhật được giao diện.

### Hàm lưu trong state
`NameRequest.submit` là một **hàm** nằm trong store. Code mở hộp thoại truyền vào "khi người dùng bấm OK thì làm gì" — giống truyền callback/lambda. Hộp thoại không cần biết đang tạo hay đổi tên nhóm.

### `@contextmenu.prevent.stop`, `@mousedown.self`, `@keydown.esc`
- `contextmenu` = chuột phải. `.prevent` chặn menu mặc định của webview; `.stop` để chuột phải vào repo không kích hoạt luôn menu của vùng trống bên dưới.
- `.self`: chỉ khi bấm đúng vào phần nền tối (không phải bên trong hộp thoại) → đóng.
- `.esc`: chỉ phản ứng phím Escape.

### `Set` reactive
`collapsed = ref(new Set<string>())` — Vue 3 theo dõi được cả `add`/`delete` trên `Set`, nên `isCollapsed()` trong template tự cập nhật.

### Hàm generic TS
`async function call<T>(promise: Promise<T>): Promise<T>` — giống generic Java: nhận Promise kiểu gì trả về đúng kiểu đó, chỉ đổi lỗi thành `AppError`.

## Cách chạy và kiểm tra
```bash
pnpm tauri dev
cd src-tauri && cargo test     # 23 test
```

| Thao tác | Kết quả mong đợi |
|---|---|
| Mở app | Repo `moon-git` (nếu còn) vẫn ở cấp cao nhất — file version 1 đọc được |
| Bấm **+** | Menu: *Add Local Repository…*, *New Group…* |
| *New Group…*, gõ `Work`, Enter | Nhóm **Work** xuất hiện (chữ đậm, có ▾) |
| Tạo thêm nhóm `work` ở cấp cao nhất | Hộp thoại giữ nguyên, chữ đỏ: `A group named "work" already exists here.` |
| Chuột phải **Work** → *New Subgroup…* → `Client A` | Client A nằm trong Work, thụt vào |
| Chuột phải **Client A** → *Add Local Repository Here…* | Repo mới nằm trong Client A |
| Chuột phải một repo → *Move To* | Danh sách *Top Level*, `Work`, `Work / Client A`; mục hiện tại bị mờ |
| Bấm vào dòng **Work** | Thu gọn (▸), bấm lần nữa mở lại (▾) |
| Chuột phải **Client A** → *Rename…* | Ô nhập có sẵn tên cũ, đã bôi đen |
| Chuột phải **Work** → *Delete Group…* | Hộp thoại native hỏi xác nhận. Đồng ý → Client A và các repo lên cấp cao nhất |
| Chuột phải vùng trống trong sidebar | Cùng menu với nút + |
| Chuột phải repo → *Remove from List* | Repo biến mất |
| Đóng app, mở lại | Cây giữ nguyên (nhưng các nhóm đều mở rộng) |

Xem file: `cat ~/Library/Application\ Support/com.github.halab4dev.moongit/repos.json` — giờ là `"version": 2`.

## Bổ sung: icon folder cho nhóm
- `GroupItem.vue` vẽ icon bằng **SVG nhúng thẳng** (`<svg><path d="..."/></svg>`), đổi giữa folder **đóng** và **mở** theo trạng thái thu gọn (`v-if` / `v-else` trên hai `<path>`).
- Vì sao không dùng emoji 📁: emoji vẽ bằng font của hệ điều hành → màu và kích thước khác nhau giữa macOS và Ubuntu. Vì sao không thêm thư viện icon: chỉ cần 2 hình.
- Màu lấy từ biến CSS `--folder` (khai trong `styles.css`, có giá trị riêng cho dark mode) qua `fill: var(--folder)` — đổi màu folder chỉ cần sửa một chỗ.
- `aria-hidden="true"`: icon chỉ để trang trí, trình đọc màn hình bỏ qua nó (tên nhóm đã đủ nghĩa).
- `d="M1.5 3.5 ..."`: chuỗi lệnh vẽ của SVG (M = di chuyển bút, a = cung tròn, h/v = đường ngang/dọc, z = đóng hình). Không cần hiểu chi tiết — thường copy từ bộ icon hoặc vẽ bằng công cụ.

## Ghi chú cho sau này
- Chưa di chuyển được **nhóm** sang nhóm khác (CLAUDE.md chỉ yêu cầu di chuyển repo). Khi thêm cần kiểm tra không cho chuyển nhóm vào chính con của nó (vòng lặp).
- `buildTree()` chưa có unit test vì dự án chưa có công cụ test cho TypeScript. Sẽ thêm (Vitest) khi làm thuật toán xếp làn ở Giai đoạn 2, lúc đó bổ sung test cho hàm này.
