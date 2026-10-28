# Bước 13a — Lưu account: accounts.json + keyring

*Lộ trình CLAUDE.md mục 8, "Account theo server: HTTPS token lưu trong keyring". Bước 13 chia ba: **13a** (phần này, Rust), **13b** hộp thoại Sign in + chạy lại, **13c** màn hình quản lý account.*

## Trạng thái
- `cargo test`: 220/220 pass (thêm 7 test). `cargo clippy`: sạch. `pnpm build`: OK.
- **Chưa thử với Keychain thật**: test dùng một "keyring giả" trong bộ nhớ. Ghi/đọc Keychain thật sẽ thử ở 13b, khi bạn đăng nhập qua hộp thoại.
- Chưa có gì mới trên giao diện. Đã commit (2026-10-28), làm tiếp 13b.

## Thư viện mới: crate `keyring` (4.2.0)
- Đã chọn sẵn trong CLAUDE.md. Một API chung (`Entry::new(service, user)`, `set_password`, `get_password`, `delete_credential`) cho kho bí mật của từng hệ điều hành:
  - macOS: **Keychain** (xem được trong app *Keychain Access*, tìm "com.github.halab4dev.moongit");
  - Linux: **Secret Service** (GNOME Keyring / KWallet), qua `zbus`, viết thuần Rust, không cần cài thêm thư viện hệ thống;
  - Windows: Credential Manager.
- Mặc định (feature `v1`) tự chọn đúng kho cho từng hệ điều hành.

## Account được lưu thế nào

| Ở đâu | Chứa gì |
|---|---|
| `accounts.json` (thư mục app data, cạnh `repos.json`) | `id`, server (`github.com`), user name. **Không có bí mật** |
| Keyring: service `com.github.halab4dev.moongit`, user = `id` | Token / mật khẩu |

```json
{ "version": 1, "accounts": [ { "id": "18b2…", "host": "github.com", "username": "halab4dev" } ] }
```

- Cùng server + cùng user name = **cùng account** (đăng nhập lại chỉ thay token).
- Account đăng nhập **gần nhất đứng đầu** danh sách, và là account git dùng cho server đó. Nếu URL có user name (`https://me@github.com/…`), dùng đúng account của user đó.

## "Thử rồi mới giữ": begin / finish
Git chỉ dùng được thứ đã lưu (helper là một chương trình khác, đọc keyring). Nên khi bạn nhập token ở hộp thoại (13b):

```
begin_sign_in(host, user, token)  → lưu ngay (keyring + accounts.json), nhớ trong RAM cách hoàn tác
chạy lại lệnh git
finish_sign_in(id, accepted)
   ├─ server nhận   → giữ
   └─ server từ chối → hoàn tác: account mới thì xóa; account cũ thì trả lại token trước đó
```

Token cũ (để hoàn tác) chỉ nằm trong bộ nhớ của app, trong vài giây đó.

## Helper đọc account
- Mọi lệnh git cần mạng giờ có thêm biến môi trường **`MOONGIT_DATA_DIR`**: chỉ là **đường dẫn thư mục** app data, không phải bí mật.
- Helper (`moongit --git-credential get`) đọc `accounts.json` ở đó, tìm account theo server, rồi đọc token trong keyring. Chỉ trả lời cho `https`/`http`.
- Thiếu biến này (git được chạy từ terminal, không qua MoonGit) thì helper không trả lời gì.

## Lỗi đăng nhập kèm tên server
- `AppError` có thêm trường **`server`** (chỉ có khi `kind = authRequired`). Phần giao diện (13b) dùng nó để biết hỏi đăng nhập cho server nào.
- Tên server lấy từ lỗi của git:
  - `could not read Username for 'https://github.com': terminal prompts disabled` → `github.com`, câu báo: "github.com needs a user name and a password or token."
  - `Authentication failed for 'https://me@git.example.com:8443/…'` → `git.example.com:8443`, câu báo: "… refused the user name or password / token."

## Command mới (cho 13b)

| Command | Việc |
|---|---|
| `list_accounts` | Danh sách account (không có token) |
| `find_account(host)` | Account dùng cho server đó (điền sẵn user name) |
| `begin_sign_in(host, username, token)` | Lưu trước khi thử, trả về id |
| `finish_sign_in(id, accepted)` | Giữ hoặc hoàn tác |

**Không command nào trả token về giao diện.** Đọc keyring có thể chậm hoặc chờ bạn trả lời "Allow access?" của macOS, nên mọi command chạy nền (`run_blocking`).

## File

| File | Việc |
|---|---|
| `src-tauri/src/storage/accounts.rs` (mới) | `AccountStore` (begin / finish sign-in), `SecretStore` + `Keyring`, `SavedAccounts` cho helper, đọc/ghi `accounts.json` + 6 test |
| `src-tauri/src/git/credential.rs` | Helper dùng `SavedAccounts` + `Keyring`; `MOONGIT_DATA_DIR`; `server_in_messages` + 1 test |
| `src-tauri/src/git/operations.rs` | Đặt `MOONGIT_DATA_DIR`; lỗi đăng nhập kèm server |
| `src-tauri/src/error.rs` | `AppError.server`, `with_server` |
| `src-tauri/src/models.rs` | `Account` |
| `src-tauri/src/commands/account.rs` (mới), `lib.rs` | 4 command; `AccountStore` được quản lý bởi Tauri |
| `src/types/account.ts`, `src/api/account.ts` (mới), `src/types/error.ts` | Kiểu + hàm gọi command; `server` trong `AppError` |

## Khái niệm mới

### Rust
- **Trait với `Send + Sync` làm "supertrait"** (`trait SecretStore: Send + Sync`): mọi kiểu cài `SecretStore` phải dùng được an toàn từ nhiều thread, vì Tauri gọi command từ thread nền.
- **`Box<dyn SecretStore>`**: giữ "một kiểu bất kỳ cài trait" trong struct, chọn lúc chạy (Keychain thật / bộ nhớ trong test). Giống field kiểu interface trong Java.
- **Generic với ràng buộc trait** (`SavedAccounts<S: SecretStore>`): cách khác cho cùng mục đích, chọn kiểu lúc biên dịch. Giống `class SavedAccounts<S extends SecretStore>` trong Java.
- **`OnceLock`**: biến toàn cục chỉ gán **một lần** (thư mục app data, lúc khởi động), đọc từ mọi nơi an toàn.
- **`ok()??`**: `ok()` đổi `Result` thành `Option`; mỗi `?` bóc một lớp `Option`. Không tìm được thì trả `None` ngay.
- **`#[serde(skip_serializing_if = "Option::is_none")]`**: bỏ hẳn trường khỏi JSON khi không có giá trị.
- **`Arc<Mutex<…>>` trong test**: hai bản `MemorySecrets` dùng chung một kho, giống app và helper dùng chung một keyring thật.

## Cách thử
Chưa có gì để thử trên giao diện. Phần nhìn thấy được (hộp thoại Sign in) là bước 13b.
