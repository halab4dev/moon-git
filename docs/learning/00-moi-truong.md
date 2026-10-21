# Bước 0 — Chuẩn bị môi trường

## Máy hiện tại
macOS 12.7.5 (Monterey), Intel x86_64.

| Thành phần | Trạng thái lúc kiểm tra | Ghi chú |
|---|---|---|
| Node.js | v24.15.0 (qua nvm), npm 11, pnpm 10 | Đủ dùng |
| git | 2.33.0 (`/usr/local/bin/git`) | Đủ. `--porcelain=v2` có từ git 2.11 |
| Homebrew | 5.1.1 | |
| Xcode Command Line Tools | Hỏng: `xcode-select` trỏ tới `/Applications/Xcode.app` không tồn tại; bản CLT còn sót lại rất cũ (clang 11, SDK 10.15) | Cần cài lại |
| Rust | Chưa có | Cần cài |
| WebView | Có sẵn | macOS dùng WKWebView (engine Safari) |

## Các thứ cần cài và để làm gì

### Xcode Command Line Tools
Bộ công cụ biên dịch của Apple: `clang` (trình biên dịch C), `ld` (linker), **macOS SDK** (header + thư viện hệ điều hành).

Vì sao cần: Rust tự compile code của nó, nhưng bước cuối cần linker hệ thống để tạo file chạy. Tauri gọi API macOS (AppKit cho cửa sổ, WebKit cho webview) nên cần SDK. Hình dung nó như "JDK của hệ điều hành".

```bash
sudo rm -rf /Library/Developer/CommandLineTools   # xóa bản cũ bị lệch
xcode-select --install                            # bấm Install trong hộp thoại
xcode-select -p   # kết quả đúng: /Library/Developer/CommandLineTools
```

Nếu `xcode-select -p` vẫn in ra đường dẫn Xcode.app cũ (cài lại CLT không tự sửa đường dẫn này), chỉ lại bằng tay:
```bash
sudo xcode-select --switch /Library/Developer/CommandLineTools
```
Cách tạm không cần sudo: `export DEVELOPER_DIR=/Library/Developer/CommandLineTools`. Biến này chỉ có tác dụng trong terminal hiện tại; `xcrun`/`clang` đọc nó trước cấu hình của `xcode-select`.

> Thiếu CLT thì cả `/usr/bin/python3` và `/usr/bin/git` của macOS cũng không chạy (chúng chỉ là "shim" gọi vào CLT). git của bạn chạy được vì là bản riêng ở `/usr/local/bin`.

### Rust (qua rustup)
- `rustup`: quản lý phiên bản Rust — như SDKMAN/jenv (Java) hay nvm (Node).
- `rustc`: trình biên dịch — như `javac`.
- `cargo`: build + quản lý thư viện — gộp vai trò Maven/Gradle và npm. Thư viện gọi là **crate**, khai báo trong `Cargo.toml` (như `pom.xml`/`package.json`).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# chọn "1) Proceed with standard installation", mở terminal mới
rustc --version && cargo --version
```
Dùng rustup thay vì `brew install rust` vì cập nhật toolchain dễ hơn và là cách Tauri khuyến nghị.

### Tauri CLI
Không cài global. Nó là devDependency npm `@tauri-apps/cli` trong dự án, chạy bằng `pnpm tauri ...` — giống khai báo plugin trong `pom.xml` thay vì cài tay, phiên bản gắn với dự án.

### Ubuntu (sau này)
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```
Trên Ubuntu, webview là WebKitGTK. `libsecret` (Secret Service cho keyring) thường có sẵn.

## Tóm tắt kiến trúc dự án
- Giao diện: Vue 3 + TypeScript chạy trong webview.
- Backend: Rust mỏng, gọi **git CLI hệ thống** với tham số dạng mảng, parse output máy đọc (`--porcelain=v2 -z`, `%x1f`/`%x1e`), trả struct serialize bằng `serde`. Không dùng libgit2.
- Luồng chuẩn: component Vue → Pinia store → `src/api/*.ts` (`invoke` có kiểu) → `#[tauri::command]` trong `commands/` → `git/*.rs` chạy git + parse → `Result<Model, AppError>` → Promise resolve/reject bên TS.
- Thao tác dài (clone/fetch/pull/push): bất đồng bộ, gửi tiến trình qua Tauri event, hủy được.
- Token chỉ nằm trong keyring hệ điều hành; thao tác phá hủy luôn có hộp thoại xác nhận.

## Lộ trình Giai đoạn 1 (đã thống nhất)

| # | Bước | Nội dung |
|---|---|---|
| 1 | Khung dự án | Tauri v2 + Vue 3 + TS strict, layout sidebar \| nội dung, command ví dụ `greet` |
| 2 | Lớp chạy git + kiểu lỗi | `git/runner.rs`, `GIT_TERMINAL_PROMPT=0`, `AppError`, `check_repo`, unit test repo tạm |
| 3 | Danh sách repo + thêm repo local | `storage/` JSON trong app data, chọn thư mục, Pinia store |
| 4 | Nhóm / nhóm con | Cây sidebar, tạo/đổi tên/xóa, di chuyển repo |
| 5 | Clone có tiến trình + hủy | `tokio::process`, Tauri event, Cancel |
| 6 | Danh sách file thay đổi | Parser `status --porcelain=v2 -z` có test |
| 7 | Stage / unstage | |
| 8 | Diff side-by-side | Monaco diff editor |
| 9 | Commit + amend | |
| 10 | Branch hiện tại + ahead/behind + fetch | |
| 11 | Pull, push | |
| 12 | Thiết kế credential | Chỉ đề xuất `GIT_ASKPASS` vs credential helper, chờ chọn |
| 13 | Account + keyring | |
| 14 | user.name / email từng repo | |
| 15 | Chọn SSH key | `GIT_SSH_COMMAND` |

Quyết định đã chốt: tên **MoonGit**, identifier `com.github.halab4dev.moongit`, package manager **pnpm**.
