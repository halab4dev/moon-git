# Bước 13b — Hộp thoại "Sign in", tự chạy lại

*Tiếp 13a. Bạn đã chọn: thiếu thông tin đăng nhập thì hỏi bằng hộp thoại, lưu theo server để dùng cho các repo sau.*

## Trạng thái
- `cargo test`: 222/222 (thêm 2 test). `cargo clippy`: sạch. `pnpm test`: 38/38. `pnpm build`: OK. App dev đã build lại.
- **Chưa thử với server thật** (chưa có repo HTTPS đòi đăng nhập). Đã commit theo yêu cầu của bạn (2026-10-28).

## Giao diện
Khi một thao tác cần mạng bị server đòi đăng nhập (hoặc từ chối token đã lưu):

```
Sign in to github.com
github.com needs a user name and a password or token.
User name          [ halab4dev               ]
Password or token  [ ••••••••••••••••        ]
GitHub, GitLab and Bitbucket need a personal access token here, not your
account password. It is saved in the system keychain for github.com and used
for its other repositories too.
                                        [Cancel] [Sign In]
```

- Áp dụng cho: **Fetch, Pull** (phần tải về), **Push, Force Push, Clone, xóa branch trên server**. Commit and Push cũng vậy, vì nó push qua nút Push.
- **Sign In** → lưu (Keychain) → **tự chạy lại** thao tác:
  - server nhận: hộp thoại đóng, thao tác hoàn tất như bình thường;
  - server từ chối: thông tin vừa nhập bị **hoàn tác** (không lưu), hộp thoại vẫn mở với câu báo, ô token được xóa để gõ lại;
  - lỗi khác (vd. push bị từ chối vì server có commit mới): đăng nhập vẫn được giữ (server không chê nó), hộp thoại đóng, lỗi hiện như trước.
- **Cancel** (hoặc Esc, bấm ra ngoài): như trước bước 13b, lỗi gốc hiện ra.
- Token cũ bị từ chối (hết hạn, bị thu hồi): hộp thoại điền sẵn **user name** của account đã lưu, chỉ cần dán token mới.
- Lần sau, mọi repo trên cùng server dùng account này, **không hỏi lại**.
- Ô token là ô mật khẩu (`type="password"`). Token chỉ nằm trong hộp thoại lúc đang mở, rồi đi thẳng sang Rust → Keychain. Không ghi ra console/log.
- Clone: hộp thoại Sign in hiện **đè lên** hộp thoại Clone; đăng nhập xong thì clone chạy lại với tiến trình như thường.

## Sửa thêm ở phía Rust: không đụng vào helper cũ của bạn
Sau mỗi lần đăng nhập, git báo kết quả cho **tất cả** helper: `store` khi được nhận, `erase` khi bị từ chối. Nếu MoonGit đứng đầu và osxkeychain của bạn đứng sau, một token sai trong MoonGit có thể làm git ra lệnh cho osxkeychain **xóa** thông tin đăng nhập vẫn đang dùng tốt trong terminal.

Nên giờ:
- MoonGit **có account** cho server của lệnh (theo URL khi clone, theo các remote của repo khi fetch/pull/push): **chỉ** helper của MoonGit được dùng. Helper của bạn không được hỏi, không nhận `store` / `erase`.
- MoonGit **chưa có account** cho server đó: như bước 12, MoonGit đứng đầu (không trả lời), helper của bạn theo sau.

## Luồng dữ liệu

```
Push → SyncButtons.push → withSignIn(() => withProgress("Pushing…", pushRepo))     (signIn.ts)
  └─ lỗi authRequired, server = "github.com"
       findAccount("github.com") → điền sẵn user name
       dialogs.askSignIn({ server, username, reason, submit, cancel }) → <SignInDialog>
         Sign In → submit(user, token)
            beginSignIn → invoke("begin_sign_in") → AccountStore: Keychain + accounts.json
            chạy lại pushRepo   (git -c credential.helper= -c credential.helper=!'moongit' --git-credential push)
                └─ git → moongit --git-credential get → SavedAccounts → Keychain → token → git
            ├─ OK          → finishSignIn(id, true)  → đóng hộp thoại
            └─ từ chối lại → finishSignIn(id, false) → hoàn tác → câu báo trong hộp thoại
         Cancel → lỗi gốc
```

## File

| File | Việc |
|---|---|
| `src/components/signIn.ts` (mới) | `withSignIn(action)`: chạy, hỏi đăng nhập khi cần, chạy lại |
| `src/components/SignInDialog.vue` (mới) | Hộp thoại |
| `src/stores/dialogs.ts`, `src/App.vue` | `SignInRequest`, gắn hộp thoại (sau cùng, để đè lên Clone) |
| `src/components/SyncButtons.vue` | Fetch, Pull, Push, Force Push qua `withSignIn` |
| `src/components/CloneDialog.vue`, `branchMenus.ts` | Clone, xóa branch trên server qua `withSignIn` |
| `src-tauri/src/git/credential.rs` | `http_host`, `has_account_for`, `helper_options(…, only_moongit)` + 2 test |
| `src-tauri/src/git/operations.rs` | Chọn `only_moongit` cho từng lệnh |

## Khái niệm mới
- **Bọc một hàm bằng hàm khác** (`withSignIn(() => pushRepo(…))`): thêm hành vi (hỏi đăng nhập, chạy lại) quanh **bất kỳ** thao tác nào mà không sửa từng thao tác. Giống decorator / AOP "around advice" trong Java.
- **Promise chờ người dùng** (`new Promise((resolve, reject) => …)`, giữ `resolve` / `reject` trong request của hộp thoại): `withSignIn` "đứng chờ" cho tới khi bạn đăng nhập xong hay bấm Cancel. Cùng ý với hộp thoại Pull (bước 11).
- **Generic TS `<T>`**: `withSignIn` trả về đúng kiểu mà thao tác bên trong trả về (`boolean` của `withProgress`, `RepoStatus` của xóa branch…).

## Cách thử
Cần một repo **HTTPS** đòi đăng nhập. Cách dễ nhất với GitHub:
1. Tạo **personal access token**: GitHub → Settings → Developer settings → Personal access tokens → *Fine-grained tokens* → Generate (chọn repo, quyền *Contents: Read and write*).
2. Trong MoonGit: **Repository → Clone Repository…** với URL HTTPS của một repo **private** của bạn, ví dụ `https://github.com/halab4dev/<repo-private>.git`. Repo public thì clone không cần đăng nhập, nhưng push thì có.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Clone repo private qua HTTPS | Hộp thoại "Sign in to github.com" đè lên Clone |
| 2 | Nhập user name + **token sai** → Sign In | "github.com refused …", hộp thoại vẫn mở, ô token trống |
| 3 | Nhập token đúng → Sign In | Clone chạy tiếp, xong; repo xuất hiện trong sidebar |
| 4 | Mở *Keychain Access*, tìm `com.github.halab4dev.moongit` | Có một mục; `accounts.json` (cạnh `repos.json`) không chứa token |
| 5 | Sửa, commit, **Push** repo vừa clone | Không hỏi gì nữa |
| 6 | Clone một repo private khác cùng github.com | Không hỏi gì |
| 7 | Thu hồi token trên GitHub → Fetch | Hộp thoại hiện lại, user name điền sẵn |
| 8 | Bấm Cancel | Lỗi hiện như trước (không có gì thay đổi) |

Lần đầu helper đọc Keychain, macOS có thể hỏi "moongit wants to use … keychain": chọn **Always Allow**. Bản dev chưa ký nên có thể hỏi lại sau mỗi lần build.
