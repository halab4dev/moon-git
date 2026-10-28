# Bước 13c — Xem các account đã lưu (MoonGit → Accounts…)

*Bạn yêu cầu (2026-10-28).*

## Trạng thái
- `cargo test`: 223/223 (thêm 1 test xóa account). `cargo clippy`: sạch. `pnpm build`: OK. App dev đã build lại.
- **Chưa thử với server thật** (chưa có repo HTTPS đòi đăng nhập). Đã commit theo yêu cầu của bạn (2026-10-28).

## Giao diện
Menu **MoonGit → Accounts…** (chỗ các app macOS thường đặt phần cài đặt chung):

```
Accounts
Logins for HTTPS servers. The password or token of each one is kept in the
system keychain. MoonGit adds one when a server asks you to sign in.

Server        User name               
github.com    halab4dev                     [Remove…]
github.com    work-account (not used)       [Remove…]
gitlab.com    halab                         [Remove…]
                                                 [Close]
```

- Chỉ hiện **server** và **user name**. Token **không bao giờ** hiện: nó nằm trong Keychain, và không command nào trả token về giao diện.
- **"(not used)"**: server có nhiều account thì account đứng đầu (đăng nhập gần nhất) là cái git dùng.
- **Remove…**: hỏi xác nhận, rồi xóa account khỏi `accounts.json` **và** token khỏi Keychain. Lần sau repo trên server đó cần đăng nhập, hộp thoại Sign in (13b) hiện lại.
- Chưa có account nào: dòng giải thích rằng repo SSH và đăng nhập HTTPS mà git trên máy đã lưu sẵn không cần account trong MoonGit.
- Thêm account mới: không có nút riêng. Account tự được thêm khi server đòi đăng nhập (13b).

## File

| File | Việc |
|---|---|
| `src-tauri/src/storage/accounts.rs` | `AccountStore::remove` + 1 test |
| `src-tauri/src/commands/account.rs`, `lib.rs` | command `remove_account` |
| `src/api/account.ts` | `removeAccount` |
| `src/components/AccountsDialog.vue` (mới) | Hộp thoại danh sách |
| `src/stores/dialogs.ts`, `src/App.vue`, `src/appMenu.ts` | Mở / đóng; mục menu **Accounts…** |

## Khái niệm
- **`<table>` HTML**: `thead` / `tbody` / `tr` / `th` / `td`, kẻ dòng bằng `border-bottom`, `border-collapse: collapse` để các đường kẻ liền nhau.
- **`tabindex="-1"` + `focus()`**: cho một `div` nhận được phím (Esc để đóng) mà không thêm nó vào vòng Tab.

## Cách thử

| # | Làm | Mong đợi |
|---|---|---|
| 1 | MoonGit → Accounts… (chưa đăng nhập lần nào) | "No saved accounts…" |
| 2 | Sau khi đăng nhập ở 13b → mở lại | Một dòng `github.com  <user name>` |
| 3 | Remove… → Remove | Dòng biến mất; *Keychain Access* không còn mục đó |
| 4 | Fetch repo HTTPS đó | Hộp thoại Sign in hiện lại |
| 5 | Esc / Close / bấm ra ngoài | Đóng |
