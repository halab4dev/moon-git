# Bước 2 — Lớp chạy git + kiểu lỗi chung

## Trạng thái
- `cargo test`: 7/7 test pass. `cargo clippy`: không cảnh báo. `pnpm build`: OK.
- Giao diện: đã thử 4 trường hợp trong bảng cuối file, đều đúng (2026-10-21).

## Mục tiêu
1. Có **một chỗ duy nhất** gọi `git` (`git/runner.rs`), luôn tuân theo quy tắc trong CLAUDE.md: tham số dạng mảng, `GIT_TERMINAL_PROMPT=0`, lỗi git thành thông báo dễ hiểu.
2. Một kiểu lỗi chung `AppError` cho mọi command, serialize được sang frontend.
3. Command thật đầu tiên: `check_repo(path)` — kiểm tra thư mục có phải repo git. Bước 3 sẽ dùng nó khi thêm repo.

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/error.rs` | **Mới.** `ErrorKind` (enum) + `AppError { kind, message }` |
| `src-tauri/src/git/mod.rs` | **Mới.** Khai báo module `runner`, `repo` |
| `src-tauri/src/git/runner.rs` | **Mới.** `run_git(cwd, args)` + 3 test |
| `src-tauri/src/git/repo.rs` | **Mới.** `check_repo(path)` + 4 test |
| `src-tauri/src/commands/repo.rs` | **Mới.** Command mỏng gọi `git::repo::check_repo` |
| `src-tauri/src/models.rs` | Thay `Greeting` bằng `RepoInfo { root, name }` |
| `src-tauri/src/lib.rs` | Khai báo `mod error; mod git;`, đăng ký `check_repo` |
| `src-tauri/Cargo.toml` | Thêm `[dev-dependencies] tempfile` |
| `src/types/error.ts`, `src/types/repo.ts` | **Mới.** Interface khớp Rust |
| `src/api/errors.ts` | **Mới.** `toAppError()` chuẩn hóa lỗi từ `invoke` |
| `src/api/repo.ts` | **Mới.** `checkRepo(path)` |
| `src/views/WelcomeView.vue` | Form demo: nhập đường dẫn → kiểm tra repo |
| `src/styles.css` | Thêm màu `--success`, `--danger` |
| ~~`commands/system.rs`, `api/system.ts`, `types/system.ts`~~ | **Xóa** ví dụ `greet` của bước 1, đã hết vai trò |

### Thư viện mới: `tempfile` (chỉ dùng cho test)
Tạo thư mục tạm và **tự xóa khi biến ra khỏi scope**. Giống `@TempDir` của JUnit 5. Nằm trong `[dev-dependencies]` nên không bị đóng gói vào app (giống Maven `<scope>test</scope>`). Chọn nó vì là crate chuẩn de-facto cho việc này, không tự viết lại phần dọn dẹp.

## Luồng dữ liệu

```
WelcomeView.vue
  onCheck() ─▶ api/repo.ts checkRepo(path)
                 └▶ invoke("check_repo", { path })
                      └▶ commands/repo.rs  check_repo(path: String)
                           └▶ git/repo.rs  check_repo(&Path)
                                └▶ git/runner.rs  run_git(path, ["rev-parse", "--show-toplevel"])
                                     └▶ tiến trình `git` của hệ thống
                                ◀── Ok(stdout) | Err(AppError)
                           ◀── Ok(RepoInfo) | Err(AppError NotARepository …)
  ◀── Promise resolve(RepoInfo)  hoặc  reject({ kind, message })
  try { repo.value = ... } catch (err) { error.value = toAppError(err) }
```

Khi Rust trả `Err(AppError)`, Tauri serialize nó thành JSON và **Promise bị reject** với object đó. Vì TypeScript coi giá trị trong `catch` là `unknown`, ta dùng `toAppError()` để chắc chắn luôn có `kind` và `message`.

## Giải thích code

### `run_git` — vì sao từng dòng
```rust
pub fn run_git(cwd: &Path, args: &[&str]) -> Result<String, AppError> {
```
- `args: &[&str]` — mảng các tham số. `Command::new("git").args(args)` truyền thẳng cho hệ điều hành, **không qua shell**, nên đường dẫn có dấu cách hay ký tự lạ không bị hiểu sai, và không có injection.
- Kiểm tra `cwd.is_dir()` trước: nếu thư mục không tồn tại, việc khởi chạy tiến trình báo `NotFound` — y như khi không có git — dễ báo nhầm "chưa cài git".
- `GIT_TERMINAL_PROMPT=0`: git không bao giờ đứng chờ nhập mật khẩu (app không có terminal → sẽ treo vĩnh viễn).
- `LC_ALL=C`: ép git nói tiếng Anh, để nhận ra lỗi bằng chuỗi như `"not a git repository"` dù máy cài ngôn ngữ khác.
- `.output()` chạy tới khi git kết thúc, gom stdout/stderr/exit code. Phù hợp cho lệnh ngắn. Lệnh dài (clone/fetch) ở bước 5 sẽ dùng cách khác để đọc tiến trình theo thời gian thực.
- `String::from_utf8_lossy`: output git là byte. Byte không hợp lệ UTF-8 (tên file mã hóa lạ) được thay bằng `�` thay vì làm hỏng cả lệnh.

### `check_repo`
Dùng `git rev-parse --show-toplevel`: in đường dẫn gốc của repo chứa thư mục hiện tại. Vì vậy chọn thư mục con của repo cũng được — kết quả là gốc repo.

Chỉ cắt ký tự xuống dòng cuối (`trim_end_matches(['\n', '\r'])`), không dùng `trim()`, vì tên thư mục hợp lệ có thể kết thúc bằng dấu cách.

## Khái niệm Rust mới

### `Result<T, E>` — thay cho exception
Rust **không có exception**. Hàm có thể lỗi trả về `Result`:
- `Ok(giá_trị)` khi thành công, `Err(lỗi)` khi thất bại.
- Giống như mọi hàm Java đều phải khai `throws` và người gọi **bắt buộc** xử lý — compiler cảnh báo nếu bạn bỏ qua một `Result`.

`Result<RepoInfo, AppError>` ≈ Java `RepoInfo checkRepo(...) throws AppError`.

### Toán tử `?`
```rust
.output()
.map_err(spawn_error)?;
```
`?` nghĩa là: nếu là `Ok(x)` thì lấy `x` đi tiếp; nếu là `Err(e)` thì **return Err(e) ngay**. Tương đương Java khi không `catch` mà để exception bay lên người gọi — nhưng ở đây hiện rõ bằng dấu `?`.

`map_err(spawn_error)` đổi kiểu lỗi: `.output()` trả `io::Error`, ta chuyển sang `AppError` trước khi `?`. Giống `catch (IOException e) { throw new AppError(...) }`.

### `enum` + `match`
```rust
pub enum ErrorKind { GitNotFound, PathNotFound, NotARepository, GitFailed, Io }
```
Ở đây giống `enum` Java. (Enum Rust còn mạnh hơn: mỗi biến thể có thể mang dữ liệu — `Result` chính là một enum `Ok(T) | Err(E)`.)

`match` ≈ `switch` của Java nhưng phải **xử lý đủ mọi trường hợp**, thiếu là lỗi compile:
```rust
let stdout = match run_git(...) {
    Ok(out) => out,
    Err(err) if err.kind == ErrorKind::GitFailed && err.message.contains("not a git repository") => {
        return Err(/* NotARepository */);
    }
    Err(err) => return Err(err),
};
```
- `Ok(out) => out` — "mở hộp" lấy giá trị bên trong (pattern matching).
- `if ...` sau pattern gọi là **match guard**: nhánh chỉ khớp khi điều kiện đúng. Ở đây: lỗi git có chứa "not a git repository" → đổi thành lỗi thân thiện hơn; mọi lỗi khác → trả nguyên.
- `match` là biểu thức, nên gán thẳng vào `let stdout = ...`.

### `Option<T>` — thay cho `null`
```rust
let name = match Path::new(&root).file_name() {
    Some(n) => n.to_string_lossy().to_string(),
    None => root.clone(),
};
```
`file_name()` trả `Option`: `Some(giá_trị)` hoặc `None`. Rust **không có null**; giá trị "có thể không có" phải là `Option`, và compiler bắt bạn xử lý `None`. Giống `Optional<T>` của Java nhưng là cách duy nhất, không thể quên.

### `impl` — method cho struct
```rust
impl AppError {
    pub fn new(kind: ErrorKind, message: String) -> AppError { AppError { kind, message } }
}
```
Struct chứa dữ liệu, method viết trong khối `impl` riêng. `AppError::new(...)` gọi giống static method/constructor Java. Rust không có constructor đặc biệt — `new` chỉ là quy ước đặt tên.

### `#[derive(...)]` thêm
- `Clone, Copy`: cho phép sao chép giá trị `ErrorKind` (nhỏ, không chứa heap) thay vì "chuyển quyền sở hữu".
- `PartialEq`: cho phép so sánh `==` (≈ `equals()` tự sinh). Cần cho `err.kind == ErrorKind::GitFailed` và `assert_eq!`.

### `&Path` vs `String`
Command nhận `path: String` (dữ liệu từ JSON, sở hữu). Logic git nhận `&Path` — **mượn** đường dẫn, kiểu dành riêng cho đường dẫn file (≈ `java.nio.file.Path`). `Path::new(&path)` tạo view `&Path` trên chuỗi mà không sao chép.

### Unit test trong Rust
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_plain_folder() { ... assert_eq!(err.kind, ErrorKind::NotARepository); }
}
```
- Test viết **ngay trong file** code, trong module `tests`. `#[cfg(test)]` = chỉ compile khi chạy test.
- `use super::*;` — import mọi thứ của file cha (kể cả hàm private — test được cả hàm không `pub`).
- `#[test]` ≈ `@Test`; `assert!`, `assert_eq!` ≈ `assertTrue`, `assertEquals`.
- `.unwrap()` — "tôi chắc chắn là Ok, lấy giá trị; nếu Err thì panic". Trong test thì OK (panic = test fail). **Không dùng trong code thật.** `.unwrap_err()` ngược lại: chắc chắn là Err.
- Test dùng repo tạm thật (`git init` trong `TempDir`), có trường hợp thư mục con chứa dấu cách.

## Khái niệm TypeScript/Vue mới
- **`ref<RepoInfo | null>(null)`**: `ref` có khai kiểu. `RepoInfo | null` là union type — "hoặc RepoInfo hoặc null".
- **`try / catch / finally` với `await`**: Promise bị reject → nhảy vào `catch`, giống Java. `finally` luôn chạy để tắt trạng thái "đang kiểm tra".
- **`:disabled="..."`**: dấu `:` (viết tắt của `v-bind`) nghĩa là giá trị thuộc tính là **biểu thức JS**, không phải chuỗi cố định.
- **Union string type** `"gitNotFound" | "pathNotFound" | ...`: TS chỉ cho phép đúng các chuỗi này — gần giống enum.

## Cách chạy và kiểm tra

```bash
cd ~/Projects/MoonGit/moon-git/src-tauri
cargo test          # chạy 7 unit test
cd ..
pnpm tauri dev      # nếu chưa chạy
```

Thử trong ô nhập của app:

| Nhập | Kết quả mong đợi |
|---|---|
| `/Users/halab/Projects/MoonGit/moon-git` | Xanh: `Git repository moon-git — /Users/halab/Projects/MoonGit/moon-git` |
| `/Users/halab/Projects/MoonGit/moon-git/src` | Xanh, vẫn ra gốc `moon-git` (thư mục con) |
| `/Users/halab` (không phải repo) | Đỏ: `/Users/halab is not a git repository. (notARepository)` |
| `/khong/ton/tai` | Đỏ: `Folder does not exist: /khong/ton/tai (pathNotFound)` |

Chạy riêng một test: `cargo test resolves_subfolder` (lọc theo tên).

## Ghi chú cho sau này
- Command này là **đồng bộ**: Tauri chạy command không `async` trên luồng chính, nên lệnh chạy lâu sẽ làm đơ giao diện. `rev-parse` chỉ vài mili-giây nên không sao; các lệnh có thể lâu sẽ được chuyển sang chạy nền khi tới bước cần.
- Khi app mở từ Finder (không từ terminal), `PATH` rất ngắn nên `git` sẽ là `/usr/bin/git` (của Command Line Tools) thay vì `/usr/local/bin/git`. Hiện cả hai đều chạy được; sau này có thể cho chọn đường dẫn git trong cài đặt.
