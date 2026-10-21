# Bước 5 — Clone có tiến trình + hủy

## Trạng thái
- `cargo test`: 36/36 pass (thêm 13 test: đọc dòng tiến trình, parse %, clone từ repo local, dọn thư mục, kiểm tra đầu vào, **hủy dừng được cả tiến trình con**). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21).

## Mục tiêu
- Menu **+** / chuột phải nhóm → **Clone Repository…** → nhập URL, chọn thư mục cha, tên thư mục (tự gợi ý từ URL).
- Trong khi clone: thanh tiến trình + giai đoạn ("Receiving objects 45%"), nút **Cancel Clone**.
- Xong: repo tự vào danh sách (đúng nhóm đã chọn) và được chọn.
- Hủy hoặc lỗi: thư mục dở dang bị xóa (chỉ khi chính lần clone này tạo ra nó).
- Hạ tầng (`git/operations.rs`) dùng lại cho fetch / pull / push ở bước 10–11.

## Thiết kế

### Chạy ở đâu: thread riêng, không chặn giao diện
Command `clone_repo` là `async`. Bên trong, `spawn_blocking` đẩy công việc sang một **thread worker**. Thread này chạy `git clone --progress` và đọc stderr từng dòng cho tới khi git kết thúc. Thread giao diện vẫn rảnh, nên bấm Cancel được.

**Vì sao không dùng `tokio::process` như CLAUDE.md gợi ý:** với tokio, việc vừa đọc output vừa chờ tín hiệu hủy cần macro `tokio::select!` và thêm khái niệm async stream. Cách đang dùng (một thread đọc stream bình thường + một bảng tiến trình có khóa để lệnh Cancel tìm và dừng) giống `Thread` + `synchronized` bên Java, dễ đọc hơn, và không cần thêm thư viện. Mỗi lần clone chỉ tốn một thread — không đáng kể với app desktop.

### Tiến trình: Tauri event
```
git (stderr) ─▶ read_lines ─▶ on_line(line) ─▶ app.emit("git-progress", GitProgress{opId, line, phase, percent})
                                                     │
frontend: listen("git-progress") ◀───────────────────┘   lọc theo opId của mình
```
- git viết tiến trình ra **stderr**. Mỗi lần cập nhật nó kết thúc bằng `\r` (để terminal ghi đè cùng một dòng); thông báo bình thường kết thúc bằng `\n`. `read_lines` tách theo cả hai.
- `--progress` bắt git in tiến trình dù không chạy trong terminal.
- `parse_percent` đọc `"Receiving objects:  45% (450/1000)…"` thành `("Receiving objects", 45)`.
- Mỗi lần clone có một **`opId`** (UUID do frontend tạo). Mọi event đều gắn `opId`, nên sau này nhiều thao tác chạy song song không lẫn tiến trình của nhau.
- **Event** (Rust đẩy về, không cần frontend hỏi) khác **command** (frontend hỏi, Rust trả lời một lần). Ở đây dùng cả hai: command `clone_repo` trả kết quả cuối cùng (Promise), event báo tiến trình ở giữa.

### Hủy: dừng cả nhóm tiến trình
- `Operations` = bảng `opId → tiến trình git đang chạy` (có `Mutex`), do Tauri giữ như `RepoStore`.
- Bấm Cancel → command `cancel_operation(opId)` → tìm trong bảng → đánh dấu `cancelled = true` → gửi tín hiệu dừng.
- **Vấn đề:** `git clone` tự sinh tiến trình con (`git-remote-https`, `ssh`, `index-pack`). Chỉ dừng `git` thì tiến trình con có thể vẫn chạy — ví dụ đang chờ kết nối tới server chậm (có thể tới vài phút) — và vẫn giữ pipe stderr, nên thread đọc không kết thúc.
- **Cách giải:** khởi động git trong một **process group** mới (`process_group(0)`), khi hủy thì gửi tín hiệu cho **cả nhóm** (`kill_process_group`). Tất cả cùng dừng.
- Gửi **SIGTERM** (lịch sự, "hãy dừng lại") chứ không phải SIGKILL (giết ngay): git kịp tự dọn dẹp.
- Test `cancel_stops_git_and_its_children` chạy một lệnh git có tiến trình con `sleep 30`, hủy sau 0,3 giây, và kiểm tra nó kết thúc ngay chứ không phải chờ 30 giây.
- **Windows** (chưa làm): process group của Windows khác hẳn; hiện code Windows chỉ dừng `git`. Phần phụ thuộc hệ điều hành gói gọn trong 2 hàm có `#[cfg(unix)]` / `#[cfg(not(unix))]`.

### Dọn dẹp
- Thư mục đích **đã có và không rỗng** → từ chối ngay, không chạy git.
- Thư mục đích **chưa có** → nếu clone lỗi hoặc bị hủy, xóa thư mục dở dang (`remove_dir_all`). An toàn vì đã kiểm tra nó **không tồn tại trước khi clone** — chỉ xóa đúng thứ chính mình vừa tạo.
- Thư mục đích **có sẵn nhưng rỗng** → được clone vào; nếu lỗi thì giữ lại thư mục (không phải mình tạo).

### Bảo mật
- Tham số vẫn dạng mảng; thêm `--` trước URL để một URL bắt đầu bằng `-` không bị hiểu là tùy chọn của git.
- `GIT_TERMINAL_PROMPT=0` + stdin rỗng: repo cần đăng nhập sẽ **báo lỗi ngay** thay vì treo chờ gõ mật khẩu. Lỗi này được đổi thành `AuthRequired` với thông báo dễ hiểu. Đăng nhập từ app là việc của bước 12–13.
- Nếu máy bạn đã có credential helper (macOS: `osxkeychain`) hoặc SSH key, git vẫn tự dùng chúng — clone repo private vẫn có thể thành công.

## File đã tạo / sửa

**Rust**
| File | Việc |
|---|---|
| `git/runner.rs` | Tách `git_command()` dùng chung (cwd, env `GIT_TERMINAL_PROMPT`, `LC_ALL`) |
| `git/operations.rs` | **Mới.** `Operations` (bảng tiến trình + cancel), `run_git_with_progress`, `read_lines`, `parse_percent`, xử lý process group. 7 test |
| `git/clone.rs` | **Mới.** `clone_repo` (kiểm tra đầu vào, chạy, dọn dẹp), `folder_name_from_url`. 6 test |
| `commands/clone.rs` | **Mới.** `clone_repo` (async), `cancel_operation`, `suggest_folder_name` |
| `models.rs` | Thêm `GitProgress` (payload của event) |
| `error.rs` | Thêm `Cancelled`, `AuthRequired` |
| `lib.rs` | `.manage(Operations::new())`, đăng ký 3 command |
| `Cargo.toml` | Thêm `rustix` (chỉ macOS/Linux) |

**Frontend**
| File | Việc |
|---|---|
| `types/progress.ts` | `GitProgress` |
| `api/clone.ts` | `cloneRepo`, `cancelOperation`, `suggestFolderName`, `onGitProgress` (lắng nghe event) |
| `stores/repos.ts` | Action `cloneRepo`; tách `insertRepo` dùng chung với `addRepo` |
| `stores/dialogs.ts` | `cloneRequest`, `openClone`, `closeClone` |
| `components/CloneDialog.vue` | **Mới.** Form + tiến trình + hủy |
| `components/sidebarMenus.ts` | Mục *Clone Repository…* / *Clone Repository Here…* |
| `App.vue` | Gắn `CloneDialog` |

### Thư viện mới: `rustix`
- Dùng để gửi tín hiệu dừng cho **cả nhóm tiến trình** (`kill_process_group`). Thư viện chuẩn của Rust chỉ có `Child::kill()` — dừng đúng một tiến trình, và luôn là SIGKILL.
- Vì sao chọn: đã có sẵn trong cây dependency (tempfile dùng nó), API **an toàn**. Lựa chọn khác là `libc` (cũng có sẵn) nhưng phải gọi hàm C trực tiếp trong khối `unsafe` — nên tránh khi có lựa chọn an toàn.
- Khai báo dưới `[target.'cfg(unix)'.dependencies]` → chỉ biên dịch trên macOS/Linux, Windows không kéo nó vào.

## Luồng dữ liệu đầy đủ
```
CloneDialog start()
  id = crypto.randomUUID()
  unlisten = await onGitProgress(p => if p.opId === id → cập nhật thanh tiến trình)   ← nghe trước
  store.cloneRepo(id, url, parentDir, folderName, groupId)
    invoke("clone_repo", {...})
      commands::clone::clone_repo (async)
        spawn_blocking(move || {                                  ← sang thread worker
          git::clone::clone_repo(ops, id, url, …, send_progress)
            kiểm tra đầu vào, thư mục đích
            run_git_with_progress(["clone","--progress","--",url,name])
              spawn git (nhóm tiến trình riêng) → ops.start(id, child)
              read_lines(stderr) → mỗi dòng: send_progress → app.emit("git-progress") ──▶ frontend
              EOF → ops.finish(id) → chờ exit code
              cancelled? → Err(Cancelled) ; exit ≠ 0 → Err(GitFailed / AuthRequired)
            lỗi + thư mục do mình tạo → xóa
          check_repo(target) → RepoStore.add_repo(info, groupId)
        })
      ← Ok(RepoEntry) | Err(AppError)
    insertRepo(entry) → sidebar có repo mới, được chọn
  finally: unlisten()

(song song) Bấm Cancel Clone
  cancelOperation(id) → invoke("cancel_operation") → ops.cancel(id)
    cancelled = true; kill_process_group(SIGTERM) → git + con dừng → stderr đóng → thread đọc kết thúc
```

## Khái niệm Rust mới

### `async fn` command + `spawn_blocking`
```rust
#[tauri::command]
pub async fn clone_repo(app: AppHandle, ...) -> Result<RepoEntry, AppError> {
    let task = tauri::async_runtime::spawn_blocking(move || { ... });
    match task.await { Ok(result) => result, Err(err) => Err(...) }
}
```
- Command thường (không `async`) chạy trên **thread chính** — chạy lâu là giao diện đơ. Command `async` chạy trên runtime nền của Tauri.
- Nhưng đọc stderr là việc **chặn** (blocking): thread đứng chờ dữ liệu. Không nên chặn thread của runtime async → `spawn_blocking` chuyển việc đó sang một thread riêng dành cho việc chặn.
- `task.await` ≈ `future.get()` của Java `CompletableFuture`, nhưng **không chặn** thread — runtime làm việc khác trong lúc chờ.
- `move ||` — closure **lấy quyền sở hữu** các biến nó dùng (`url`, `op_id`, `app`…), vì nó chạy trên thread khác và có thể sống lâu hơn hàm gọi nó.
- `Err(err)` ở `match` là khi chính thread worker bị panic — hiếm, nhưng vẫn đổi thành `AppError` thay vì bỏ qua.

### `AppHandle`, `Manager`, `Emitter`
- `AppHandle` = tay cầm tới app, gửi qua thread được. Từ nó lấy state (`app.state::<Operations>()`) và gửi event (`app.emit(...)`).
- `Manager`, `Emitter` là **trait** — giống interface Java. Phải `use` trait thì mới gọi được method của nó (`state`, `emit`). Đây là chỗ Rust khác Java: method của interface chỉ "nhìn thấy" khi interface được import.
- Command async không nhận `State<'_, T>` vào trong closure `move` được (vì `State` là tham chiếu mượn, không được mang sang thread khác) → nhận `AppHandle` rồi lấy state bên trong thread.

### `FnMut` vs `FnOnce`
- `on_line: impl FnMut(&str)` — closure được gọi **nhiều lần** và được phép **sửa** biến bên ngoài (ví dụ `lines.push(...)` trong test).
- `FnOnce` (bước 4) — chỉ gọi một lần.
- (Còn `Fn` — gọi nhiều lần, không sửa gì.) Rust bắt khai rõ để kiểm tra an toàn khi dùng chung.

### `HashMap`
`HashMap<String, RunningOp>` ≈ `HashMap<String, RunningOp>` của Java. `get_mut` lấy tham chiếu sửa được; `remove` lấy hẳn giá trị ra khỏi map (trả `Option`).

### `let ... else`
```rust
let Some(op) = running.get_mut(op_id) else {
    return Ok(());
};
// từ đây `op` dùng được
```
"Nếu khớp mẫu thì gán, không thì chạy khối `else` (bắt buộc phải thoát: `return`, `break`…)". Gọn hơn `match` khi chỉ quan tâm trường hợp tốt — giống guard clause `if (x == null) return;` bên Java.

### `#[cfg(unix)]` — code theo hệ điều hành
```rust
#[cfg(unix)]
fn terminate(child: &mut Child) -> io::Result<()> { /* process group */ }
#[cfg(not(unix))]
fn terminate(child: &mut Child) -> io::Result<()> { child.kill() }
```
Compiler chỉ biên dịch phiên bản đúng với hệ điều hành đang build. Phần còn lại của code gọi `terminate()` mà không cần biết OS nào. Giữ phần phụ thuộc OS nhỏ và ở một chỗ.

### `pub(crate)`
`git_command`, `spawn_error` là `pub(crate)`: dùng được ở mọi file trong crate này nhưng không phải API công khai. ≈ package-private của Java nhưng phạm vi là cả crate.

### Byte, `&[u8]`, `b"..."`
stderr đọc ra là **byte** (`u8`). `b'\r'` là một byte, `b"..."` là chuỗi byte. Chỉ chuyển sang `String` (`from_utf8_lossy`) khi đã có trọn một dòng — tránh cắt đôi ký tự UTF-8 nhiều byte giữa hai lần đọc.

### `std::thread::scope` (trong test)
```rust
std::thread::scope(|scope| {
    let worker = scope.spawn(|| run_git_with_progress(&ops, ...));
    ...
    ops.cancel("slow").unwrap();
    worker.join().unwrap()
});
```
Thread "có phạm vi": được **mượn** biến của hàm (`&ops`) vì Rust đảm bảo mọi thread trong scope kết thúc trước khi scope kết thúc. Thread thường (`std::thread::spawn`) thì không được mượn, phải `move` hoặc dùng `Arc`.

## Khái niệm Vue / TS / web mới

### Lắng nghe event của Tauri
```ts
const unlisten = await onGitProgress((p) => { ... });
try { await store.cloneRepo(...) } finally { unlisten(); }
```
- `listen()` đăng ký listener và trả về hàm **hủy đăng ký**. Luôn gọi nó trong `finally`, nếu không mỗi lần clone lại thêm một listener (rò bộ nhớ).
- Đăng ký **trước** khi gọi command, để không lỡ những dòng tiến trình đầu tiên.

### `crypto.randomUUID()`
API có sẵn của trình duyệt, tạo id ngẫu nhiên không trùng — dùng làm `opId`.

### `localStorage`
Lưu thư mục cha lần trước để lần sau điền sẵn. Chỉ là tiện ích nhỏ, nên bọc `try/catch` — nếu không có thì vẫn chạy bình thường. Thông tin quan trọng (danh sách repo) vẫn lưu ở Rust.

### `watch` với hàm async — chống trả lời cũ
```ts
watch(url, async (value) => {
  const suggestion = await suggestFolderName(value);
  if (url.value === value && !folderEdited.value) folderName.value = suggestion;
});
```
Gõ nhanh → nhiều lời gọi chạy cùng lúc, có thể trả về **không theo thứ tự**. Kiểm tra `url.value === value` để chỉ dùng câu trả lời của lần gõ mới nhất.

### `<progress>`
Thẻ HTML có sẵn. Có `value` → thanh phần trăm; không có `value` → hiệu ứng "đang chạy" (dùng khi git chưa báo %). `:value="percent ?? undefined"` — `undefined` làm Vue bỏ hẳn thuộc tính.

## Cách chạy và kiểm tra
```bash
pnpm tauri dev
cd src-tauri && cargo test      # 36 test
```

| Thao tác | Kết quả mong đợi |
|---|---|
| **+** → *Clone Repository…* | Hộp thoại Clone, con trỏ ở ô URL |
| Gõ `https://github.com/tauri-apps/tauri.git` | Ô *Folder name* tự thành `tauri` |
| *Browse…* chọn thư mục cha, bấm **Clone** | Thanh tiến trình chạy: "Counting objects…", "Receiving objects 37%"…, "Resolving deltas…" |
| Trong khi chạy, bấm **Cancel Clone** | Dừng sau khoảng 1 giây, quay lại form không báo lỗi; thư mục `tauri` **không còn** trên ổ đĩa |
| Clone repo nhỏ, ví dụ `https://github.com/octocat/Hello-World.git` | Xong → hộp thoại đóng, `Hello-World` xuất hiện trong sidebar và được chọn |
| Chuột phải một nhóm → *Clone Repository Here…* | Dòng "· group …" hiện tên nhóm; xong thì repo nằm trong nhóm đó |
| Clone lại vào cùng thư mục cha, cùng tên | Chữ đỏ: `… already exists and is not an empty folder.` |
| URL sai, ví dụ `https://github.com/khong/ton-tai-abc.git` | Chữ đỏ: thông báo cần đăng nhập (GitHub trả lời như vậy với repo không tồn tại) |
| URL linh tinh, ví dụ `abc` | Chữ đỏ `fatal: repository 'abc' does not exist` |
| Mở lại hộp thoại | Thư mục cha lần trước được điền sẵn |
| Esc / bấm nền tối khi đang clone | Không đóng (phải Cancel trước) |

`tauri` là repo lớn (vài trăm MB) — tiện để thử Cancel. Nếu mạng nhanh, thử Cancel ngay khi vừa bắt đầu.

## Ghi chú cho sau này
- Đóng app khi đang clone: tiến trình git có thể tiếp tục chạy nền tới khi xong. Sẽ xử lý khi làm phần thoát app.
- Tên thư mục hiển thị ghép bằng `/`; Windows sẽ cần dấu `\` (chỉ ảnh hưởng dòng hiển thị, Rust tự ghép đường dẫn đúng).
- SSH hỏi xác nhận host key lần đầu (`Are you sure you want to continue connecting?`) sẽ thất bại vì không có terminal. Xử lý ở bước 15 (SSH).
