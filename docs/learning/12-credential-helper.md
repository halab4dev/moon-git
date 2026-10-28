# Bước 12 — MoonGit làm "credential helper" cho git

*Theo lộ trình CLAUDE.md (Giai đoạn 1, mục 8), tạm hoãn từ 2026-10-21, làm lại 2026-10-28. Bạn đã chọn: cách B (credential helper riêng), tự dùng account cùng server, thiếu thông tin thì hỏi bằng hộp thoại (bước 13).*

## Trạng thái
- `cargo test`: 213/213 pass (thêm 5 test). `cargo clippy`: sạch.
- Đã chạy thử chương trình thật: `moongit --git-credential get` trả lời rỗng ngay, **không mở cửa sổ**; git rồi báo "terminal prompts disabled" như trước.
- **Chưa có gì thay đổi trên giao diện**: chưa có account nào được lưu (bước 13), nên helper chưa bao giờ trả lời. Mọi thứ chạy y như trước. Bạn không có repo HTTPS để thử; đã commit (2026-10-28).

## Credential helper là gì
Khi git cần đăng nhập vào một server HTTPS, git không tự hỏi người dùng mà hỏi lần lượt các **credential helper** đã cấu hình (trên macOS thường là `osxkeychain`). Mỗi helper là một chương trình. Git chạy nó và nói chuyện qua stdin/stdout bằng các dòng `key=value`:

```
git → helper (stdin):   protocol=https
                        host=github.com
                        (dòng trống)
helper → git (stdout):  username=halab4dev
                        password=ghp_…
```

Helper nào trả lời đủ username + password trước thì git dùng cái đó. Không ai trả lời thì git hỏi trên terminal; app đã tắt việc này (`GIT_TERMINAL_PROMPT=0`), nên git báo lỗi và app hiện "The server asked for a username and password…".

## MoonGit làm gì
Với **mọi lệnh git cần mạng** (clone, fetch, pull, push, force push, xóa branch trên server), MoonGit thêm vào đầu lệnh:

```
git -c credential.helper=                                      ← xóa danh sách helper đang có…
    -c credential.helper=!'<đường dẫn MoonGit>' --git-credential   ← …MoonGit đứng đầu
    -c credential.helper=osxkeychain                           ← rồi helper cũ của bạn, đúng thứ tự cũ
    fetch --all --progress
```

- `-c` chỉ có tác dụng **cho đúng lệnh đó**. App không ghi gì vào `~/.gitconfig` hay `.git/config`, nên git trong terminal không bị ảnh hưởng.
- Danh sách helper cũ đọc bằng `git config --get-all credential.helper` rồi gắn lại **sau** MoonGit. Nên nếu MoonGit không có gì (như hiện tại), osxkeychain vẫn được hỏi như trước.
- Git chạy helper `!…` qua shell, nên đường dẫn MoonGit được **đặt trong nháy đơn** (có test với thư mục chứa dấu cách).

Khi git cần đăng nhập, nó chạy **chính chương trình MoonGit**: `moongit --git-credential get`. `main.rs` thấy tham số này thì **không mở cửa sổ**, chỉ:
1. đọc yêu cầu từ stdin (`protocol`, `host`, `path`, `username`);
2. hỏi "nguồn credential" (`CredentialSource`). Bước 12 chưa có account nào (`NoSavedAccounts`), bước 13 sẽ đọc account + keyring;
3. in `username=…` / `password=…`, hoặc không in gì.

Bảo mật:
- Token chỉ đi qua **đường ống stdout** của helper sang git. Không nằm trong tham số, biến môi trường, file hay log.
- Giá trị có xuống dòng bị **từ chối**: nếu không, một "mật khẩu" như `x\nhost=other.com` có thể tự thêm dòng vào câu trả lời.
- `store` (git báo đăng nhập thành công) và `erase` (server từ chối) hiện chỉ được nhận rồi bỏ qua; bước 13 sẽ quyết định có dùng hay không.

## Luồng dữ liệu

```
Fetch → run_git_with_progress (operations.rs)
          ├─ credential_options(cwd) → credential::helper_options(cwd, current_exe)
          │      git config --get-all credential.helper  → ["osxkeychain"]
          └─ git -c credential.helper= -c credential.helper=!'…moongit' --git-credential -c credential.helper=osxkeychain fetch …
                 │  (chỉ khi server HTTPS đòi đăng nhập)
                 ▼
             moongit --git-credential get   (main.rs → credential_helper → helper_main)
                 stdin → parse_request → source.find(…) → stdout   (bước 12: rỗng)
```

## File

| File | Việc |
|---|---|
| `src-tauri/src/git/credential.rs` (mới) | Chế độ helper (`helper_main`, `answer`, `parse_request`), `CredentialSource`, `helper_options`, `shell_quote` + 5 test |
| `src-tauri/src/git/operations.rs` | `run_git_with_progress` thêm `-c credential.helper…` trước mọi lệnh cần mạng |
| `src-tauri/src/main.rs` | Thấy `--git-credential` thì chạy helper rồi thoát, không mở app |
| `src-tauri/src/lib.rs` | `CREDENTIAL_HELPER_ARG`, `credential_helper` cho `main.rs` |

## Test (không cần mạng)
- Trả lời đúng `get` cho server đã biết; không trả lời server lạ, `store` / `erase`, đầu vào rỗng, hay giá trị có xuống dòng.
- Đặt nháy cho đường dẫn (dấu cách, dấu nháy).
- **Thứ tự helper với git thật** (`git credential fill`), dùng hai helper giả bằng shell script trong thư mục có dấu cách:
  - helper "của MoonGit" trả lời → git dùng nó, **không hỏi** helper của người dùng;
  - helper "của MoonGit" im lặng → helper của người dùng vẫn được dùng.
- Không có helper nào cấu hình: MoonGit vẫn đứng đầu, ngay sau dòng xóa danh sách.

## Khái niệm mới

### Rust
- **`trait` làm "cổng cắm"** (`CredentialSource`): `answer` chỉ biết "có ai đó tìm được credential", không biết đó là keyring thật hay đồ giả trong test. Giống `interface` trong Java + truyền implementation vào (dependency injection). `&dyn CredentialSource` là "tham chiếu tới bất kỳ kiểu nào cài trait này", như biến kiểu interface trong Java.
- **Một chương trình, hai vai**: `main` đọc `std::env::args()` và rẽ nhánh trước khi khởi động Tauri. `std::process::exit(code)` thoát ngay với mã trả về.
- **`std::env::current_exe()`**: đường dẫn của chính chương trình đang chạy (để git gọi lại nó).
- **`.chain(...)`**: nối hai iterator (`-c …` của helper + tham số gốc) mà không tạo mảng trung gian, giống `Stream.concat` của Java.
- **`contains(['\n', '\r', '\0'])`**: tìm một trong nhiều ký tự trong chuỗi.
- **`r"…"` cho chuỗi có `\`**: `r"'\''"` là đúng 4 ký tự `'\''`, cách shell viết một dấu nháy bên trong chuỗi nháy đơn.

## Cách thử
Chưa có gì để thử trên giao diện: chưa lưu account thì helper chưa trả lời. Kiểm tra là **không có gì thay đổi**:

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Fetch / Pull / Push repo `moon-git` (SSH) | Như trước |
| 2 | Fetch một repo HTTPS mà máy đã lưu đăng nhập (osxkeychain) | Như trước: không hỏi gì |
| 3 | Terminal: `printf 'protocol=https\nhost=github.com\n\n' \| src-tauri/target/debug/moongit --git-credential get` | Không in gì, thoát ngay, không mở cửa sổ |
