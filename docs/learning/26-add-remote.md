# Bước 26 — Thêm remote, hiện và sửa URL

*Bạn yêu cầu thêm (2026-10-25), CLAUDE.md không có.*

## Trạng thái
- `cargo test`: 203/203 pass. Thêm 7 test (repo tạm):
  - thêm remote `upstream` (bỏ khoảng trắng thừa); remote hiện ngay trong danh sách branch (chưa có branch), sau `fetch --all` có `upstream/main`;
  - tên đã có (`origin`), rỗng, bắt đầu bằng `-`, có dấu cách, có `..` → báo lỗi, không thêm gì;
  - URL rỗng, bắt đầu bằng `-`, có xuống dòng → báo lỗi;
  - tên có `/` (`team/main`) vẫn được;
  - đọc URL của mọi remote (tên có dấu chấm `team.a`, URL có dấu cách); repo không có remote → rỗng; danh sách branch mang URL;
  - sửa URL `origin` (bỏ khoảng trắng thừa); remote không tồn tại → `NotFound`; URL bắt đầu bằng `-` → từ chối, URL cũ giữ nguyên.
- `cargo clippy`: sạch. `pnpm test`: 30/30 (thêm 2 test cho `displayUrl`). `pnpm build`: OK.
- Giao diện: bạn đã thử, OK. Đã commit.

## Remote là gì
Remote là một **tên ngắn trỏ tới URL** của một repo khác, lưu trong `.git/config`:

```
[remote "upstream"]
    url = https://github.com/goc/repo.git
    fetch = +refs/heads/*:refs/remotes/upstream/*
```

- `origin`: tên git tự đặt cho repo bạn clone về.
- Thường thêm remote thứ hai khi làm việc với **fork**: `origin` = fork của bạn, `upstream` = repo gốc.
- Dòng `fetch = …` nghĩa là: khi fetch, branch `x` trên server được lưu thành `refs/remotes/upstream/x`. Vì vậy panel Branches có nhóm riêng cho mỗi remote.

## Giao diện
Ở panel **Branches**:
- Nút **+** trên dòng **Remotes**, hoặc chuột phải dòng **Remotes** → **Add Remote…**.
- Repo chưa có remote nào: dòng "No remotes." có link **Add Remote…**.

Hộp thoại có hai ô **Name** và **URL**:
- Repo chưa có remote: tên điền sẵn `origin`, con trỏ đặt ở ô URL.
- Đã có remote: tên để trống (phải đặt tên khác, ví dụ `upstream`).
- Lỗi (tên trùng, tên sai, URL rỗng…) hiện ngay trong hộp thoại, hộp thoại không đóng.

Sau khi thêm, remote mới hiện ngay với dòng "No branches (not fetched yet?)". Bấm **Fetch** ở header: fetch đã chạy `git fetch --all` (bước 7), nên lấy luôn remote mới.

### Hiện URL và sửa URL (bạn yêu cầu thêm)
- Mỗi nhóm remote hiện **URL bên cạnh tên**, chữ nhỏ và mờ hơn. Sidebar hẹp thì URL bị cắt bằng "…"; di chuột lên để xem đủ.
- **Mật khẩu / token nằm trong URL bị che**: `https://me:ghp_xxx@github.com/…` hiện là `https://me:***@github.com/…` (hàm `displayUrl`, có test). Danh sách nằm trên màn hình suốt, mà CLAUDE.md không cho lộ secret. Cách đúng là không ghi token vào URL; phần account (bước 12–15) sẽ lo việc đó.
- **Chuột phải một remote** → **Edit URL of origin…**: dùng lại hộp thoại Add Remote, tên khóa (không sửa được), URL điền sẵn và được chọn sẵn → **Save**.
  - Các branch đã fetch vẫn giữ nguyên; lần Fetch sau dùng URL mới.
  - Nếu ai đó đã đặt riêng `remote.<name>.pushurl` bằng tay, URL push đó **không đổi** (git `set-url` chỉ đổi URL chính).
- Remote đã bị xóa nhưng còn sót ref (không có URL) thì không có menu sửa.

## Lệnh git

| Việc | Lệnh |
|---|---|
| Kiểm tra tên | `git check-ref-format refs/remotes/<name>/probe` |
| Thêm | `git remote add -- <name> <url>` |
| URL của mọi remote | `git config -z --get-regexp ^remote\..*\.url$` |
| Sửa URL | `git remote set-url -- <name> <url>` |

### Vì sao đọc URL bằng `git config` thay vì `git remote -v`
- `git remote -v` hiện URL **sau khi** áp dụng `url.<base>.insteadOf` (luật đổi URL trong config). Sửa theo giá trị đó sẽ ghi đè giá trị gốc bằng giá trị đã đổi. `git config` trả về đúng giá trị ghi trong file.
- `-z`: mỗi mục là `remote.<name>.url` + xuống dòng + giá trị + byte `\0`. Tên remote có thể chứa dấu chấm (`team.a`), nên tên = phần giữa `remote.` và `.url`.
- Không có remote nào → `--get-regexp` thoát với mã 1 (không tìm thấy), nên kiểm tra `git remote` trước và trả về rỗng.
- Một remote có thể có nhiều dòng `url`; lấy dòng đầu (`entry().or_insert_with`).

### Ghi chú an toàn
- **`--`**: mọi thứ sau nó là tham số thường, không phải option. Tên hay URL bắt đầu bằng `-` không bao giờ bị git hiểu thành option.
- Tuy git cho phép tên `-x`, app vẫn **chặn tên bắt đầu bằng `-`**: các lệnh sau như `git push -x main` sẽ hiểu nhầm nó là option.
- **URL bắt đầu bằng `-`** bị chặn. Ví dụ `-oProxyCommand=…` có thể bị ssh đọc thành option; git cũng tự chặn, đây là lớp thứ hai.
- **Ký tự điều khiển** (xuống dòng…) trong URL bị chặn: `.git/config` lưu mỗi giá trị một dòng.
- **Tên remote** theo đúng luật tên ref (không dấu cách, `..`, `~`, `^`, `:`…), vì branch của nó được lưu thành `refs/remotes/<name>/…`. App nhờ `git check-ref-format` kiểm tra, giống cách git tự kiểm tra.
- Tên trùng được kiểm tra trước (`git remote`), để báo câu dễ hiểu thay cho "error: remote origin already exists.".
- **Không** kiểm tra URL có kết nối được không: việc đó cần đăng nhập server, mà phần account (bước 12–15) đang tạm hoãn. URL sai sẽ báo lỗi ở lần Fetch.

## Luồng dữ liệu

```
listBranches → Rust list_branches → remote::urls (git config) → RemoteGroup { name, url, branches }
  └─ BranchPanel: tên + displayUrl(url) (mờ, che token)
Chuột phải remote → Edit URL… → askEditUrl → dialogs.askRemote({ existing: { name, url } })
  └─ Save → setRemoteUrl → invoke("set_remote_url") → remote::set_url → git remote set-url -- name url
+ (Remotes) → BranchPanel.askAddRemote → dialogs.askRemote({ initialName, submit })
  └─ AddRemoteDialog (gắn một lần trong App.vue) → Add → submit(name, url)
       └─ addRemote → invoke("add_remote") → Rust git::remote::add
            check_name / check_url → git remote add -- name url
       ← lỗi: AppError.message hiện trong hộp thoại
       ← OK: BranchPanel.load() → nhóm remote mới hiện ra, hộp thoại đóng
```

Cả hai lệnh (`add_remote`, `set_remote_url`) trả về `()` (không có dữ liệu), không trả `RepoStatus`: thêm remote không đổi file hay branch hiện tại, chỉ danh sách branch cần tải lại.

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/remote.rs` (mới) | `add`, `set_url`, `urls`, `check_name`, `check_url` + 7 test |
| `src-tauri/src/models.rs`, `git/branches.rs` | `RemoteGroup.url` |
| `src/components/remoteUrl.ts` (+ test, mới) | `displayUrl`: che mật khẩu trong URL |
| `src-tauri/src/git/mod.rs` | `pub mod remote` |
| `src-tauri/src/commands/branch.rs`, `lib.rs` | command `add_remote`, `set_remote_url` |
| `src/api/branch.ts`, `src/types/branches.ts` | `addRemote`, `setRemoteUrl`, trường `url` |
| `src/stores/dialogs.ts` | `RemoteRequest`, `askRemote` / `closeRemote` |
| `src/components/AddRemoteDialog.vue` (mới) | hộp thoại hai ô Name / URL, cũng dùng để sửa (tên `readonly`) |
| `src/App.vue` | gắn `<AddRemoteDialog />` |
| `src/components/BranchPanel.vue` | URL mờ cạnh tên remote; nút **+**, menu chuột phải trên "Remotes" và trên từng remote, link khi chưa có remote |

## Khái niệm mới

### Rust
- **Hàm trả về `&str` mượn từ tham số** (`fn check_name<'a>(…, name: &'a str) -> Result<&'a str, …>`): `trim()` không tạo chuỗi mới, chỉ trả về một "lát" của chuỗi gốc. Lifetime `'a` báo cho Rust biết kết quả sống lâu bằng `name`. Java không có khái niệm này: `trim()` của Java luôn trả về một `String` (có thể là object mới).
- **`chars().any(|c| c.is_control())`**: duyệt từng ký tự; `any` dừng ngay ở ký tự đầu tiên khớp, giống `stream().anyMatch(...)`.
- **`HashMap::entry(k).or_insert_with(...)`**: chỉ thêm khi khóa chưa có, giống `map.computeIfAbsent(k, ...)` của Java.
- **`HashMap::remove(name)`** trả về `Option` giá trị: "lấy ra" URL để chuyển (move) vào `RemoteGroup`, không cần `clone`.
- **Chuỗi thô `r"…"`**: dấu `\` không phải escape, tiện cho regex (`r"^remote\..*\.url$"`), giống text block của Java nhưng một dòng.
- **`unwrap_err()`** trong test: lấy lỗi ra từ `Err`, và panic nếu kết quả lại là `Ok`. Là mặt ngược của `unwrap()`.

### Vue
- **Chọn template ref theo điều kiện**: `(request.initialName ? urlInput : nameInput).value?.focus()`. Hai ref là hai biến bình thường, nên chọn một trong hai bằng biểu thức `? :`.
- **Một hộp thoại cho hai việc**: request có `existing: {…} | null`; template đổi tiêu đề, nút (Add / Save) và `:readonly` theo nó.
- **`replace` với regex và `$1`** (`displayUrl`): giữ lại nhóm thứ nhất, thay phần mật khẩu bằng `***`. Giống `String.replaceAll` của Java.
- **`@click.stop` trên nút nằm trong dòng có `@click`**: bấm **+** không làm nhóm Remotes gập lại, vì `.stop` chặn sự kiện lan lên phần tử cha (giống `event.stopPropagation()`).

## Cách thử

App dev đã build lại và đang chạy. Bấm ⌘R trước khi thử.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo không có remote (`branchy`) | Dòng "No remotes. Add Remote…" |
| 2 | Bấm **Add Remote…** | Hộp thoại, tên sẵn `origin`, con trỏ ở ô URL |
| 3 | Nhập URL (ví dụ đường dẫn tới một repo local khác, hoặc URL GitHub) → Add | Nhóm `origin` hiện ra: "No branches (not fetched yet?)" |
| 4 | Bấm **Fetch** ở header | Các branch của `origin` hiện ra |
| 5 | Repo đã có `origin` → **+** trên dòng Remotes | Tên để trống |
| 6 | Nhập tên `origin` → Add | "A remote named "origin" already exists.", hộp thoại vẫn mở |
| 7 | Tên `my fork` | Báo tên không hợp lệ |
| 8 | Chuột phải dòng Remotes | Menu **Add Remote…** |
| 9 | Esc / Cancel / bấm ra ngoài | Hộp thoại đóng, không thêm gì |
| 10 | Nhìn các nhóm remote | URL mờ cạnh tên; kéo sidebar hẹp lại → URL bị cắt "…", di chuột → thấy đủ |
| 11 | Chuột phải `origin` → **Edit URL of origin…** | Tên khóa, URL điền sẵn; sửa → Save → URL mới hiện ra |
| 12 | Sửa URL thành rỗng | Nút Save bị mờ |
| 13 | Terminal: `git remote set-url origin https://me:secret@example.com/x.git` → ↻ | Hiện `https://me:***@example.com/x.git` (đặt lại URL cũ sau khi thử) |
