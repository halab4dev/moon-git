# Bước 15a — SSH key và cách đăng nhập HTTPS cho từng repo

*Lộ trình CLAUDE.md mục 8: "SSH: dùng ssh của hệ thống; cho phép chọn key cho từng server/repo". Bạn chọn: theo repo (và nhóm ở 15b), ghi vào `.git/config`; thêm lựa chọn "dùng của máy" cho cả HTTPS.*

## Trạng thái
- `cargo test`: 238/238 (thêm 9 test). `cargo clippy`: sạch. `pnpm test`: 38/38. `pnpm build`: OK.
- Giao diện: bạn đã xem, OK. Đã commit (2026-11-01).

## Giao diện
**Repository → Login (SSH & HTTPS)…**, hoặc chuột phải repo → **Login (SSH & HTTPS)…**:

```
Login · moon-git
How this repository logs in to its servers, for each kind of remote URL.
SSH (remote URLs like git@github.com:…)
Used by: origin
◉ Use this computer's SSH settings
   ~/.ssh/config, the keys in ssh-agent, ~/.ssh/id_ed25519…
○ Use this key (only this one is offered to the server)
     [ /Users/halab/.ssh/id_work          ] [Choose…]
HTTPS (remote URLs like https://github.com/…)
No remote of this repository uses HTTPS now (applies if one is added).
◉ MoonGit's accounts
   Asks you to sign in when needed; saved in MoonGit → Accounts….
○ Use this computer's credentials
   git's own credential helpers (e.g. the macOS keychain), as in a terminal.
                                              [Cancel] [Save]
```

### Một repo dùng cả SSH lẫn HTTPS?
Có, nên hộp thoại luôn có cả hai phần (bạn hỏi):
- nhiều remote khác giao thức, vd. `origin = git@github.com:me/x.git` (SSH, fork của bạn) và `upstream = https://github.com/goc/x.git` (HTTPS, repo gốc); Fetch chạy `--all` nên dùng cả hai;
- một remote có **URL push riêng** (`pushurl`), vd. fetch qua HTTPS, push qua SSH: ghi là "origin (push)".

Mỗi phần ghi **remote nào dùng giao thức đó** ("Used by: origin, upstream"), hoặc "No remote of this repository uses … now". Nhận dạng URL: `https://` / `http://` → HTTPS; `ssh://` hoặc dạng `git@host:path` → SSH; thư mục, `file://`, `git://` → không cần đăng nhập.

Tên menu ban đầu là "SSH Key & Login…", bạn thấy không rõ là có cả HTTPS, nên đổi thành **"Login (SSH & HTTPS)…"**.

### SSH
- **Use this computer's SSH settings** (mặc định): như git thường làm. `ssh` tự tìm key trong `~/.ssh/config` (khối `Host` khớp server, như alias `halab4dev.github.com` của bạn), trong ssh-agent, rồi các key mặc định. Chọn lại mục này rồi Save = **bỏ** key đã đặt.
- **Use this key**: **Choose…** mở thư mục `~/.ssh`, hoặc gõ đường dẫn (gõ `~/…` cũng được). Phải là **khóa riêng** (file không có `.pub`), và file phải tồn tại.
- Nếu repo đã có một lệnh SSH **tự đặt bằng tay** (`core.sshCommand` khác dạng MoonGit ghi): hộp thoại hiện nó và cảnh báo Save sẽ thay thế.

### HTTPS
- **MoonGit's accounts** (mặc định): như bước 13. Thiếu đăng nhập thì hiện hộp thoại Sign in.
- **Use this computer's credentials**: MoonGit **đứng ngoài**, không gắn helper của mình. Git dùng helper sẵn có (osxkeychain…) y như trong terminal. Nếu server vẫn đòi đăng nhập, MoonGit **không** hiện hộp thoại Sign in (account lưu ở đó repo này không dùng); câu báo lỗi chỉ tới hộp thoại này.

## Lưu ở đâu
Cả hai đều ghi vào `.git/config` của repo, nên git trong terminal làm y hệt:

| Lựa chọn | `.git/config` |
|---|---|
| Use this key | `core.sshCommand = ssh -i '/Users/halab/.ssh/id_work' -o IdentitiesOnly=yes` |
| Use this computer's SSH settings | (không có `core.sshCommand`) |
| Use this computer's credentials | `moongit.credentials = system` |
| MoonGit's accounts | (không có `moongit.credentials`) |

- **`core.sshCommand`**: lệnh git dùng thay cho `ssh` (git 2.10+). Git chạy nó **qua shell**, nên đường dẫn được đặt trong nháy đơn (có test với thư mục có dấu cách và dấu nháy).
- **`-o IdentitiesOnly=yes`**: ssh **chỉ** đưa đúng key này, không thử thêm các key trong agent. Không có nó, server có thể nhận nhầm một key khác trong agent trước (vd. tài khoản cá nhân thay vì công ty).
- **`moongit.credentials`**: một khóa riêng của MoonGit (git bỏ qua các khóa nó không biết). Đặt trong repo để mỗi repo tự nhớ lựa chọn của mình.
- Không bao giờ sửa `~/.ssh/config` hay `~/.gitconfig`.

## Luồng dữ liệu

```
SSH Key & Login… → <AccessDialog> → getRemoteAccess → invoke("get_remote_access") → git::access::read
Save → setRemoteAccess(repoId, key | null, systemCredentials) → invoke("set_remote_access") → git::access::set
         ├─ key:  git config --local -- core.sshCommand "ssh -i '<key>' -o IdentitiesOnly=yes"
         ├─ máy:  git config --local --unset-all core.sshCommand
         └─ HTTPS: git config --local -- moongit.credentials system   /   --unset-all
Fetch / Push … → run_git_with_progress
         ├─ credential::helper_options: moongit.credentials = system → không gắn helper MoonGit
         └─ lỗi đăng nhập + system → bỏ `server` khỏi lỗi → không hiện Sign in
```

## File

| File | Việc |
|---|---|
| `src-tauri/src/git/access.rs` (mới) | `read`, `set`, `uses_system_credentials`, đọc lại key từ lệnh, kiểm tra file key (+ `~`), remote theo giao thức + 8 test |
| `src-tauri/src/git/credential.rs` | Không gắn helper khi repo dùng của máy (+ 1 test); `shell_quote` dùng chung |
| `src-tauri/src/git/operations.rs` | Lỗi đăng nhập của repo dùng của máy: không mời Sign in |
| `src-tauri/src/models.rs` | `RemoteAccess` |
| `src-tauri/src/commands/access.rs` (mới), `lib.rs` | `get_remote_access`, `set_remote_access` |
| `src/types/access.ts`, `src/api/access.ts` (mới) | Kiểu + hàm gọi |
| `src/components/AccessDialog.vue` (mới) | Hộp thoại |
| `src/stores/dialogs.ts`, `src/App.vue`, `src/appMenu.ts`, `src/stores/menuCommands.ts`, `src/components/sidebarMenus.ts` | Mở từ menu Repository và menu chuột phải |

## Khái niệm mới

### Rust
- **Đảo ngược một phép biến đổi để kiểm tra** (`key_in_command`): bỏ nháy, rồi `shell_quote` lại và so với bản gốc. Chỉ nhận đúng thứ MoonGit đã ghi; mọi lệnh khác coi là "tự đặt bằng tay".
- **Biến khai báo trước, gán sau để mượn** (`let expanded; … expanded.as_str()`): giá trị phải sống đủ lâu cho tham chiếu `&str` dùng sau `match`. Rust bắt lỗi nếu sống không đủ (lifetime).
- **`then_some`**: `điều_kiện.then_some(giá_trị)` = `Some(giá_trị)` nếu đúng, `None` nếu sai.

### Tauri / Vue
- **`homeDir()` + `join()`** (`@tauri-apps/api/path`): đường dẫn thư mục home, ghép `.ssh`, để hộp thoại chọn file mở sẵn ở đó.
- **`open({ defaultPath })`** của plugin dialog: chọn một file.

## Cách thử
App dev bạn đang chạy tự build lại phần Rust; bấm ⇧⌘R nếu giao diện chưa đổi.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | `moon-git` → Repository → Login (SSH & HTTPS)… | SSH: "Used by: origin", "Use this computer's SSH settings"; HTTPS: "No remote … uses HTTPS", "MoonGit's accounts" |
| 2 | Use this key → Choose… | Hộp thoại chọn file mở ở `~/.ssh` |
| 3 | Chọn file `.pub` → Save | Lỗi "public half…", không lưu |
| 4 | Chọn đúng khóa riêng mà GitHub nhận → Save → Fetch | Fetch thành công; terminal: `git config --local core.sshCommand` ra lệnh có key đó |
| 5 | Chọn một khóa GitHub **không** nhận → Fetch | Lỗi "Permission denied (publickey)" (chứng tỏ đúng key đã được dùng) |
| 6 | Mở lại → Use this computer's SSH settings → Save → Fetch | Như cũ (dùng `~/.ssh/config`); `core.sshCommand` đã bị xóa |
| 7 | HTTPS → Use this computer's credentials → Save | `git config --local moongit.credentials` = `system` |
