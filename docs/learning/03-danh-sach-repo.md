# Bước 3 — Danh sách repo + thêm repo local

## Trạng thái
- `cargo test`: 13/13 pass (thêm 6 test cho storage). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã thử theo bảng cuối file, đều đúng (2026-10-21).

## Mục tiêu
- Bấm **+** ở sidebar → hộp thoại chọn thư mục của hệ điều hành → kiểm tra là repo git → lưu vào danh sách.
- Danh sách được **lưu xuống file JSON**, mở lại app vẫn còn.
- Bấm vào repo để chọn; nút **×** để bỏ repo khỏi danh sách (không xóa gì trên ổ đĩa).

## Thư viện mới

| Thư viện | Dùng để làm gì | Vì sao chọn |
|---|---|---|
| `pinia` (npm, v4) | Store chung: danh sách repo + repo đang chọn, dùng bởi cả sidebar và màn hình chính | CLAUDE.md đã chọn; thư viện state chính thức của Vue |
| `tauri-plugin-dialog` (Rust) + `@tauri-apps/plugin-dialog` (npm) | Mở hộp thoại chọn thư mục native | Plugin chính thức của Tauri. Webview không tự mở được hộp thoại thư mục native. Plugin có hai nửa: Rust (đăng ký bằng `.plugin(...)`) và JS (hàm `open()`) |

Không thêm crate `uuid`: id được sinh từ thời gian (nano giây, dạng hex) — đủ cho id tạo bằng cú click.

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/storage/mod.rs` | **Mới.** Module lưu cấu hình (ghi chú: không bao giờ chứa secret) |
| `src-tauri/src/storage/repos.rs` | **Mới.** `RepoStore`: đọc/ghi `repos.json`, thêm/xóa, 6 test |
| `src-tauri/src/models.rs` | Thêm `RepoEntry { id, name, path }` (có `Deserialize` vì cũng là định dạng file) |
| `src-tauri/src/error.rs` | Thêm `RepoAlreadyAdded`, `Storage` |
| `src-tauri/src/commands/repo.rs` | Thay `check_repo` bằng `list_repos`, `add_repo`, `remove_repo` |
| `src-tauri/src/lib.rs` | Đăng ký plugin dialog; `setup` tạo `RepoStore` và giao cho Tauri quản lý |
| `src-tauri/capabilities/default.json` | Thêm quyền `dialog:allow-open` |
| `src/main.ts` | `app.use(createPinia())` |
| `src/stores/repos.ts` | **Mới.** Pinia store `useRepoStore` |
| `src/api/repo.ts` | `listRepos`, `addRepo`, `removeRepo` |
| `src/types/repo.ts`, `src/types/error.ts` | `RepoEntry`; thêm 2 loại lỗi |
| `src/components/AppSidebar.vue` | Nút +, danh sách repo, nút ×, hộp báo lỗi |
| `src/views/RepoView.vue` | **Mới.** Màn hình repo đang chọn (tạm chỉ hiện tên + đường dẫn) |
| `src/views/WelcomeView.vue` | Bỏ form thử của bước 2 |
| `src/App.vue` | Gọi `store.load()` khi mở app; chọn RepoView hay WelcomeView |

## File dữ liệu

Vị trí (Tauri tính theo identifier của app):
- macOS: `~/Library/Application Support/com.github.halab4dev.moongit/repos.json`
- Ubuntu: `~/.local/share/com.github.halab4dev.moongit/repos.json`

Nội dung:
```json
{
  "version": 1,
  "repos": [
    { "id": "18a3f2c9b4e7d000", "name": "moon-git", "path": "/Users/halab/Projects/MoonGit/moon-git" }
  ]
}
```
`version` để sau này đổi định dạng (bước 4 thêm nhóm) mà vẫn đọc được file cũ.

**Ghi an toàn:** ghi ra `repos.json.tmp` trước, rồi `rename` đè lên `repos.json`. `rename` là thao tác nguyên tử của hệ điều hành, nên nếu app sập giữa chừng, file cũ vẫn nguyên vẹn — không bao giờ có file ghi dở.

**File hỏng** (ví dụ bạn sửa tay sai cú pháp): app không từ chối khởi động, mà đổi tên file thành `repos.json.broken` (để bạn lấy lại dữ liệu) và bắt đầu với danh sách rỗng.

## Luồng dữ liệu

### Khi mở app
```
main.rs → lib.rs run()
  setup(): app_data_dir() → RepoStore::open(".../repos.json")  → đọc file vào bộ nhớ
           app.manage(store)                                      → Tauri giữ hộ 1 bản duy nhất
App.vue onMounted → store.load() → api listRepos() → invoke("list_repos")
  → commands::list_repos(store: State<RepoStore>) → store.list() → Vec<RepoEntry> → JSON
  ← repos.value = [...] → sidebar tự vẽ lại
```

### Khi bấm +
```
AppSidebar onAdd()
  open({ directory: true })                   ← plugin dialog: hộp thoại native, trả đường dẫn hoặc null
  store.add(path) → api addRepo(path) → invoke("add_repo", { path })
    → commands::add_repo
        git::repo::check_repo(path)?          ← tái dùng bước 2: phải là repo git, lấy thư mục gốc
        store.add(info)
           trùng đường dẫn? → Err(RepoAlreadyAdded)
           ghi file mới → rồi mới cập nhật bộ nhớ
    ← RepoEntry
  repos.value.push(entry); selectedId = entry.id   → sidebar thêm dòng (đã chọn), màn hình chính đổi sang RepoView
```
Lỗi ở bất kỳ đâu (không phải repo, trùng, không ghi được file) → Promise reject → `run()` trong store bắt lại → `store.error` → hộp đỏ trong sidebar.

## Khái niệm Rust mới

### State dùng chung: `app.manage()` + `State<'_, T>`
```rust
app.manage(RepoStore::open(...));                      // đăng ký 1 lần lúc khởi động
pub fn list_repos(store: State<'_, RepoStore>) -> ...  // Tauri tự "tiêm" vào command
```
Giống **Spring bean singleton + dependency injection**: đăng ký một instance, Tauri truyền nó vào command nào khai báo tham số kiểu `State<RepoStore>`. Tham số `State` không đến từ frontend — frontend vẫn chỉ gửi `{ path }`.

`'_` là **lifetime** (thời gian sống của tham chiếu). Ở đây chỉ có nghĩa: "`store` là tham chiếu mượn, đừng giữ nó lâu hơn lời gọi command". Viết `'_` để compiler tự suy ra — bạn chưa cần hiểu sâu hơn.

### `Mutex` — khóa cho dữ liệu dùng chung
```rust
repos: Mutex<Vec<RepoEntry>>
let mut repos = self.lock()?;   // khóa
...                              // tự mở khóa khi `repos` ra khỏi scope
```
Command có thể chạy trên nhiều thread cùng lúc. `Mutex` ≈ `synchronized` / `ReentrantLock` của Java, nhưng **dữ liệu nằm bên trong khóa**: không khóa thì không lấy được `Vec` ra — compiler cấm luôn việc quên khóa.

`lock()` trả về `MutexGuard`. Khi guard bị hủy (hết scope), khóa tự mở — như `try-with-resources` nhưng tự động, không cần viết `finally { unlock() }`.

"Poisoned" lock: nếu một thread panic khi đang giữ khóa, Rust đánh dấu khóa là "nhiễm độc" vì dữ liệu có thể đang dở dang. Ta đổi trường hợp này thành `AppError` thay vì `unwrap()` (sẽ panic theo).

### `*repos = updated`
`repos` là guard trỏ tới `Vec` bên trong khóa. `*` = "giá trị mà nó trỏ tới" → thay cả danh sách bên trong khóa bằng danh sách mới.

### Closure và iterator (≈ lambda + Stream của Java)
```rust
repos.iter().find(|r| r.path == info.root)
repos.iter().filter(|r| r.id != id).cloned().collect()
```
- `|r| r.path == info.root` là **closure** ≈ lambda Java `r -> r.path.equals(root)`.
- `.iter().filter(...).cloned().collect()` ≈ `list.stream().filter(...).map(copy).collect(toList())`.
- `.cloned()`: iterator cho ra *tham chiếu*; `cloned` tạo bản sao để vector mới sở hữu chúng.

### `if let`
```rust
if let Some(existing) = repos.iter().find(...) { return Err(...); }
```
Viết tắt của `match` khi chỉ quan tâm một trường hợp: "nếu tìm thấy (Some) thì lấy ra `existing` và làm...".

### `Clone` và `.clone()`
`#[derive(Clone)]` cho phép tạo bản sao độc lập (sâu). Trong Java, gán đối tượng chỉ copy tham chiếu; trong Rust, muốn có hai bản thì phải `clone()` rõ ràng — nhìn code là biết chỗ nào tốn bộ nhớ.

### `Deserialize`
`RepoEntry` có thêm `Deserialize` vì giờ phải **đọc** từ file JSON (`serde_json::from_str`). ≈ `objectMapper.readValue(text, RepoFile.class)`.

### `setup` và `?` trong closure
`.setup(|app| { ...; Ok(()) })` chạy một lần khi app khởi động. `app_data_dir()?` — nếu không xác định được thư mục dữ liệu thì app báo lỗi và dừng, vì không có chỗ lưu cấu hình.

## Khái niệm Vue mới

### Pinia store
```ts
export const useRepoStore = defineStore("repos", () => {
  const repos = ref<RepoEntry[]>([]);        // state
  const selected = computed(() => ...);       // giá trị suy ra
  async function add(path: string) { ... }   // action
  return { repos, selected, add, ... };
});
// trong component:
const store = useRepoStore();
store.repos, store.add(...)
```
- Store = **một nơi duy nhất giữ state dùng chung** cho nhiều component. Giống một singleton service bên Spring giữ dữ liệu, component nào cũng inject được.
- Gọi `useRepoStore()` ở đâu cũng nhận về **cùng một** store.
- Không dùng store thì phải truyền danh sách repo từ App xuống Sidebar bằng props, rồi báo ngược lên bằng event — rắc rối khi nhiều component cần.

### `computed`
```ts
const selected = computed(() => repos.value.find((r) => r.id === selectedId.value) ?? null);
```
Giá trị **tự tính lại** khi `repos` hoặc `selectedId` đổi, và được cache (không tính lại nếu không có gì đổi). Giống một getter có cache tự làm mới. `sortedRepos` cũng là computed — danh sách luôn được sắp theo tên.

`??` = "nếu bên trái là null/undefined thì lấy bên phải".

### Props: `defineProps`
```vue
<!-- App.vue -->
<RepoView :repo="store.selected" />
<!-- RepoView.vue -->
defineProps<{ repo: RepoEntry }>();
```
Props = tham số truyền **từ component cha xuống con**, giống tham số constructor. Con chỉ đọc, không sửa props. Kiểu được khai bằng TypeScript nên truyền sai kiểu là lỗi compile.

### `onMounted`
Chạy một lần khi component đã hiển thị lên màn hình — chỗ hợp lý để tải dữ liệu ban đầu. ≈ `@PostConstruct`.

### `v-for` + `:key`
```vue
<li v-for="repo in store.sortedRepos" :key="repo.id">
```
Lặp để vẽ danh sách. `:key` giúp Vue nhận ra dòng nào là dòng nào khi danh sách thay đổi (thêm/xóa/sắp xếp lại), để chỉ cập nhật đúng dòng đó.

`:key` trên `<RepoView>` trong App.vue: khi đổi repo, Vue tạo RepoView **mới hoàn toàn** thay vì tái dùng cái cũ — sau này RepoView có state riêng (file đang chọn, diff…) sẽ không bị lẫn giữa các repo.

### `:class="{ selected: ... }"` và `@click.stop`
- `:class` với object: thêm class `selected` khi điều kiện đúng.
- `.stop` = `event.stopPropagation()`: bấm × không đồng thời "chọn" dòng đó.

## Cách chạy và kiểm tra

```bash
cd ~/Projects/MoonGit/moon-git
pnpm tauri dev
cd src-tauri && cargo test     # 13 test
```

| Thao tác | Kết quả mong đợi |
|---|---|
| Bấm **+**, chọn `moon-git` | Hiện `moon-git` trong sidebar, được chọn (nền xanh); màn hình chính hiện tên + đường dẫn |
| Bấm **+**, chọn `moon-git/src` | Hộp đỏ: `moon-git is already in the list.` (thư mục con → gốc → trùng) |
| Bấm **+**, chọn thư mục không phải repo (vd. `~/Documents`) | Hộp đỏ: `… is not a git repository.` |
| Bấm **+** rồi Cancel | Không có gì xảy ra |
| Thêm vài repo khác | Danh sách sắp theo tên (không phân biệt hoa thường) |
| Đóng app, mở lại | Danh sách vẫn còn |
| Rê chuột lên một repo, bấm **×** | Repo biến mất khỏi danh sách; thư mục trên ổ đĩa không bị đụng tới |

Xem file dữ liệu:
```bash
cat ~/Library/Application\ Support/com.github.halab4dev.moongit/repos.json
```

## Ghi chú cho sau này
- Đường dẫn được so sánh nguyên văn để phát hiện trùng. Hai đường dẫn khác chữ hoa/thường trên ổ macOS không phân biệt hoa thường có thể lọt qua — hiếm, để sau.
- Repo bị xóa/di chuyển trên ổ đĩa vẫn nằm trong danh sách; bước 6 (đọc status) sẽ báo lỗi rõ khi mở repo đó.
