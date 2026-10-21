# GitDesk — Git GUI client (Tauri + Rust + Vue/TypeScript)

## Mục tiêu
Xây một Git client desktop thay thế SmartGit cho nhu cầu cá nhân, chạy trên macOS và Ubuntu (Windows hỗ trợ sau, nên tránh code phụ thuộc hệ điều hành ngay từ đầu).

## Về người dùng — RẤT QUAN TRỌNG
- Tôi quen nhất **Java**, code được **NodeJS**. **Rust, Vue, Tauri là mới** với tôi.
- Tôi không chỉ muốn app chạy được, tôi muốn **hiểu code làm gì**. Với Rust và Vue chỉ cần hiểu ở mức cơ bản.
- Vì vậy, hãy làm việc ở **chế độ học tập**:
    1. Làm từng bước nhỏ, mỗi bước chỉ một tính năng hoặc một phần rõ ràng.
    2. Sau mỗi bước, giải thích **bằng tiếng Việt**: đã tạo/sửa file nào, vì sao, luồng dữ liệu đi thế nào (Vue → invoke → Rust → git → trả về).
    3. Khi gặp khái niệm Rust lần đầu (ownership, borrowing, `Result`, `?`, `Option`, trait, `async`, lifetime…), giải thích ngắn gọn và **so sánh với Java** khi có thể.
    4. Khi gặp khái niệm Vue lần đầu (`ref`, `computed`, `watch`, props/emit, store…), giải thích ngắn và so sánh với cách làm quen thuộc nếu có.
    5. Không thêm thư viện mới mà không nói nó dùng để làm gì và vì sao chọn nó.
    6. Kết thúc mỗi bước: nói tôi cách chạy và kiểm tra, rồi **dừng lại chờ tôi xác nhận** trước khi sang bước tiếp theo.
- Ưu tiên code dễ đọc hơn code "khéo". Tránh macro phức tạp, generic lồng nhau, hay trick khó hiểu khi không thật sự cần.
- Comment trong code viết bằng tiếng Anh, ngắn gọn, giải thích "vì sao" ở những chỗ không hiển nhiên.

## Công nghệ
- **Tauri v2**
- **Backend:** Rust (stable). Dùng `serde` để chuyển dữ liệu sang frontend.
- **Frontend:** Vue 3 (Composition API, `<script setup lang="ts">`), TypeScript strict mode, Vite, Pinia cho state.
- **Thao tác Git:** gọi **git CLI** qua `std::process::Command` (hoặc `tokio::process` cho thao tác dài). **Không** dùng `git2-rs`/libgit2, để phần Rust mỏng và dễ hiểu, đồng thời hành vi giống hệt git của hệ thống.
- **Diff viewer:** Monaco diff editor (hoặc lựa chọn nhẹ hơn nếu có lý do, nhưng phải giải thích).
- **Lưu thông tin bảo mật:** crate `keyring` (macOS Keychain, Secret Service trên Ubuntu, Windows Credential Manager).

## Kiến trúc
```
src-tauri/src/
  main.rs / lib.rs     # đăng ký các Tauri command
  commands/            # các hàm #[tauri::command], mỏng, chỉ gọi xuống git/
  git/                 # chạy lệnh git + parse output, mỗi file một nhóm (status.rs, log.rs, branch.rs…)
  models.rs            # struct dữ liệu trả về frontend (derive Serialize)
  error.rs             # kiểu lỗi chung, serialize được sang frontend
  storage/             # cấu hình app: danh sách repo, nhóm, account (không chứa secret)
src/
  api/                 # wrapper có kiểu cho invoke(), mỗi command một hàm TS
  types/               # interface TS khớp với models.rs
  stores/              # Pinia stores
  components/          # component Vue
  views/
```

## Quy tắc khi làm việc với git
- Luôn truyền tham số dạng **mảng args**, **không bao giờ** ghép chuỗi lệnh shell (tránh lỗi đường dẫn có dấu cách và lỗi injection).
- Dùng định dạng output dành cho máy đọc: `git status --porcelain=v2 -z`, `git log` với `--format` và ký tự phân cách rõ ràng (ví dụ `%x1f`, `%x1e`), `-z` khi có thể.
- Đặt `GIT_TERMINAL_PROMPT=0` để git không treo khi chờ nhập mật khẩu.
- Thao tác dài (clone, fetch, push, pull) chạy bất đồng bộ, gửi **tiến trình** về giao diện qua Tauri event, và cho phép hủy.
- Lỗi từ git (stderr, exit code) phải được chuyển thành thông báo dễ hiểu trên giao diện, không nuốt lỗi.
- Parser output git phải có **unit test Rust**, dùng repo tạm tạo trong test.

## Tính năng và lộ trình

### Giai đoạn 1 — dùng được hằng ngày
1. Khung dự án Tauri + Vue + TS chạy được, layout cơ bản (sidebar repo | nội dung chính).
2. Thêm repo local (chọn thư mục, kiểm tra là repo git) và **clone** repo remote (có tiến trình).
3. **Nhóm / nhóm con** repo: cây trong sidebar, tạo/đổi tên/xóa nhóm, di chuyển repo giữa các nhóm (kéo-thả làm sau cũng được). Lưu dạng JSON trong thư mục app data.
4. Danh sách file thay đổi (staged / unstaged / untracked), stage/unstage file.
5. Xem diff từng file (side-by-side).
6. Commit (message, amend).
7. Fetch, pull, push; hiển thị ahead/behind của branch hiện tại.
8. **Quản lý account:**
    - Account theo server (GitHub, GitLab, server tự host…): HTTPS token lưu trong keyring, **không bao giờ** lưu secret dạng text.
    - Gắn account cho từng repo, và `user.name` / `user.email` riêng cho từng repo.
    - SSH: dùng ssh của hệ thống; cho phép chọn key cho từng server/repo.
    - Cung cấp credential cho git một cách an toàn (ví dụ qua `GIT_ASKPASS` trỏ về app hoặc credential helper). Đề xuất phương án và giải thích trước khi làm.

### Giai đoạn 2 — lịch sử và branch
9. **Graph lịch sử commit:**
    - Chọn những branch (local/remote/tag) muốn hiển thị.
    - Filter theo **commit id** (prefix hash) và theo **commit message**.
    - Có hai chế độ filter: **highlight** (giữ nguyên graph, làm mờ commit không khớp; mặc định) và **ẩn** commit không khớp.
    - Thuật toán xếp làn (lane layout) viết bằng TypeScript, tách thành module riêng có test, kèm giải thích cách hoạt động.
    - Danh sách ảo hóa (chỉ render phần đang hiển thị) và tải dần, để mượt với repo hàng chục nghìn commit, kể cả trên WebKitGTK (Ubuntu).
    - Bấm vào commit: xem thông tin, danh sách file và diff của commit đó.
10. **Branch:** tạo, checkout, đổi tên, xóa (local/remote), tracking branch.

### Giai đoạn 3 — thao tác nâng cao
11. **Stash:** tạo (kèm message, tùy chọn gồm untracked), danh sách, apply, pop, drop.
12. **Merge:** hai lựa chọn: tạo merge commit (`--no-ff`), hoặc chỉ đưa thay đổi vào working tree (`--squash` / `--no-commit`).
13. **Rebase** (không interactive) lên branch đã chọn.
14. **Cherry-pick:** hai lựa chọn: giữ nguyên commit, hoặc chỉ copy thay đổi vào working tree (`-n`).
15. **Xử lý conflict** (bắt buộc đi kèm 11–14): phát hiện trạng thái đang merge/rebase/cherry-pick, danh sách file conflict, mở công cụ so sánh ngoài hoặc chọn ours/theirs, đánh dấu đã giải quyết, nút Continue / Abort.

## Những điều cần tránh
- Không làm nhiều tính năng cùng lúc trong một bước.
- Không tự ý refactor lớn mà không hỏi trước.
- Không lưu token/mật khẩu ra file, log, hay console.
- Không chạy lệnh git có tính phá hủy (`reset --hard`, `push --force`, `clean`, xóa branch…) mà không có hộp thoại xác nhận trên giao diện.