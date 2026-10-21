# Bước 9 — Commit + amend

## Trạng thái
- `cargo test`: 71/71 pass (thêm 10 test: commit chỉ file đã stage, message nhiều dòng / có dòng `#` / bắt đầu bằng `--`, amend thêm file, amend chỉ đổi message, lỗi đầu vào, đang có conflict, `head_commit` trước commit đầu, phát hiện commit đã push, nhận diện lỗi thiếu danh tính, ghi stdin). `cargo clippy`: sạch. `pnpm build`: OK.
- Giao diện: đã xác nhận (2026-10-21). Đã commit.

## Mục tiêu
- Ô **commit message** ở cuối màn hình repo, nút **Commit**, phím tắt ⌘+Enter (Ctrl+Enter trên Ubuntu).
- Ô **Amend last commit**: tick vào thì message cũ được điền sẵn (nếu ô đang trống); bỏ tick khi chưa sửa gì thì message được xóa đi.
- Amend một commit **đã push** → hiện cảnh báo, và khi bấm Amend thì có hộp thoại xác nhận.
- Lỗi dễ hiểu: chưa nhập message, chưa stage gì, chưa có commit để amend, còn conflict, git chưa biết tên/email của bạn.

## Lệnh git

| Việc | Lệnh | Ghi chú |
|---|---|---|
| Commit | `git commit --quiet --cleanup=whitespace -F -` | message gửi qua **stdin** |
| Amend | như trên + `--amend` | thay commit cuối = thay đổi cũ + những gì đang stage, message mới |
| Message commit cuối | `git log -1 --no-show-signature --format=%B HEAD` | `%B` = message gốc (tiêu đề + thân) |
| Commit cuối đã push chưa? | `git branch --remotes --contains HEAD` | có dòng nào → đã nằm trên một remote branch |
| Repo đã có commit chưa? | `git rev-parse --verify --quiet HEAD` | gom thành hàm `has_commits` trong `git/repo.rs` (stage.rs dùng lại) |

### Vì sao gửi message qua stdin (`-F -`) thay vì `-m "<message>"`
- `-m` cũng an toàn với mảng args (không qua shell), nhưng stdin còn tốt hơn: nội dung **không bao giờ** bị hiểu nhầm là option, và không vướng giới hạn độ dài dòng lệnh.
- Test `keeps_multi_line_message_and_hash_lines` commit message `--amend is not an option here…` để kiểm tra điều này.
- Runner có thêm hàm `run_git_with_input(cwd, args, input)`. Phần kiểm tra exit code được tách ra thành `check_output`, dùng chung với `run_git_bytes`.

### `--cleanup=whitespace`
Khi bạn commit bằng editor, git mặc định **xóa mọi dòng bắt đầu bằng `#`** (coi là comment). Trong app không có editor nên điều đó chỉ gây hại: message `#123 fix login` sẽ mất dòng đầu. `whitespace` chỉ cắt khoảng trắng và dòng trống thừa.

### Kiểm tra trước khi gọi git
`git commit` khi chưa stage gì in "nothing to commit" ra **stdout** và stderr thì **trống**, nên lỗi chỉ là "git commit failed (exit status: 1)", rất khó hiểu. Vì vậy `commit()` đọc status trước:
- Còn file conflict → báo "Resolve them before committing" (xử lý conflict là bước 24).
- Không amend mà chưa stage gì → "Nothing is staged…".
- Amend khi repo chưa có commit → "There is no commit to amend yet."

### Lỗi thiếu danh tính
Nếu máy chưa cấu hình `user.name`/`user.email`, git in một đoạn dài "Please tell me who you are… Run git config --global…". `explain_commit_error` nhận ra đoạn này và đổi thành `ErrorKind::IdentityUnknown` (kind mới) kèm câu ngắn gọn. Có kind riêng là để bước 14 (user.name/email theo repo) có thể mở thẳng màn hình cài đặt khi gặp lỗi này.

### Amend commit đã push — vì sao hỏi lại
Amend không phá dữ liệu trên máy bạn: commit cũ vẫn còn trong reflog. Nhưng nếu commit đó đã lên server thì lần push sau sẽ bị từ chối, phải `push --force`, và ai đã pull commit cũ sẽ phải tự sửa lại. CLAUDE.md yêu cầu thao tác kiểu này phải có xác nhận, nên app hỏi bằng hộp thoại `ask` (plugin dialog đã có từ bước 3).

`branch --contains` phải duyệt lịch sử, có thể chậm ở repo lớn. Vì vậy app **chỉ gọi khi bạn tick Amend**, không gọi ở mỗi lần làm mới status.

> Thông tin "đã push" dựa trên remote-tracking branch **lần fetch gần nhất**. Fetch là bước 10, nên trước đó thông tin này có thể chưa cập nhật.

## Luồng dữ liệu

```
CommitBox.vue: gõ message, bấm Commit (hoặc ⌘+Enter)
  └─ nếu amend + đã push → ask() xác nhận
  └─ emit("commit")
RepoView.vue: commitStaged()
  └─ run(() => commitChanges(repoId, message, amend))      ← busy guard như stage/unstage
       └─ invoke("commit_changes", { repoId, message, amend })
            └─ Rust commands/commit.rs → run_blocking
                 ├─ git::commit::commit()  → read_status (kiểm tra) → git commit -F - (stdin)
                 └─ git::status::read_status() → RepoStatus mới
  └─ thành công: xóa message, bỏ tick Amend; bảng file + diff tự cập nhật (statusVersion++)

Tick "Amend last commit":
CommitBox watch(amend) → getHeadCommit(repoId) → invoke("get_head_commit")
  └─ git log -1 --format=%B  +  git branch --remotes --contains HEAD
  └─ null (chưa có commit) → bỏ tick, hiện ghi chú; có → điền message, nhớ "pushed"
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/runner.rs` | `run_git_with_input` (ghi stdin), tách `check_output` |
| `src-tauri/src/git/repo.rs` | `has_commits` dùng chung |
| `src-tauri/src/git/stage.rs` | dùng `has_commits` |
| `src-tauri/src/git/commit.rs` (mới) | `commit`, `head_commit`, `explain_commit_error` + 9 test |
| `src-tauri/src/commands/commit.rs` (mới) | `commit_changes`, `get_head_commit` |
| `src-tauri/src/models.rs` | `HeadCommit { message, pushed }` |
| `src-tauri/src/error.rs`, `src/types/error.ts` | `IdentityUnknown` |
| `src-tauri/src/lib.rs`, `git/mod.rs`, `commands/mod.rs` | đăng ký module + command |
| `src/types/commit.ts`, `src/api/commit.ts` (mới) | kiểu + wrapper invoke |
| `src/components/CommitBox.vue` (mới) | ô message, Amend, nút Commit |
| `src/views/RepoView.vue` | giữ `commitMessage`/`amend`, `commitStaged()`; `run()` trả về `true/false` |
| `src/styles.css` | textarea dùng chung style với input |

## Khái niệm mới

### Rust
- **`Stdio::piped()` + `child.stdin.take()`**: mở một "ống" tới stdin của tiến trình con. `take()` lấy giá trị ra khỏi `Option` và để lại `None`, giống chuyển quyền sở hữu: sau đó `child` không còn giữ pipe nữa. Khi biến `stdin` hết phạm vi (cuối khối `if let`), pipe tự **đóng**, và git hiểu là input đã hết. So với Java `Process.getOutputStream()` thì ở Java bạn phải tự gọi `close()`; Rust đóng tự động khi giá trị bị drop (giống try-with-resources nhưng không cần viết gì).
- **`wait_with_output()`**: chờ tiến trình kết thúc và đọc hết stdout/stderr. Cách này tránh treo khi output lớn làm đầy bộ đệm của pipe.
- **`map_err(explain_commit_error)`**: truyền thẳng tên hàm thay cho closure `|e| explain_commit_error(e)`, giống method reference `this::explain` trong Java.
- **`vec![...]` rồi `push`**: dựng mảng args có điều kiện (thêm `--amend` khi cần). Vẫn là mảng args, không ghép chuỗi.

### Vue
- **Hai `v-model` trên một component**: `v-model:message` và `v-model:amend` gắn với hai `defineModel` khác tên. Cha (RepoView) giữ giá trị vì cha là nơi xóa chúng sau khi commit thành công; con (CommitBox) đọc và ghi như biến bình thường.
- **`watch` một `defineModel`**: khi `amend` đổi (dù do bạn tick hay do cha đặt lại `false`), con phản ứng: điền hoặc xóa message, tải thông tin HEAD. Sau `await` phải kiểm tra lại `amend.value`, vì bạn có thể đã bỏ tick trong lúc chờ (cùng ý tưởng chống trả lời cũ ở bước 5).
- **`@keydown.meta.enter` / `@keydown.ctrl.enter`**: phím tắt ghép, giống `@keydown.meta.a` ở bước 7.

## Cách thử

App dev đang chạy và đã tự build lại. Nên thử trên một repo thử nghiệm, ví dụ tạo nhanh:

```bash
mkdir -p ~/tmp/try-commit && cd ~/tmp/try-commit && git init && echo a > a.txt
```

rồi **Add Repository** thư mục đó.

| # | Làm | Kết quả mong đợi |
|---|---|---|
| 1 | Chưa stage gì, gõ message | Nút **Commit** mờ, dòng chữ "Nothing staged" |
| 2 | Tick **Amend** khi repo chưa có commit nào | Tự bỏ tick, ghi chú "There is no commit to amend yet." |
| 3 | Stage `a.txt`, gõ `First commit`, bấm ⌘+Enter | Bảng file trống ("working tree clean"), ô message trống; `git log` trong terminal thấy commit |
| 4 | Tick **Amend** | Ô message được điền `First commit`, nút đổi thành **Amend**, dòng chữ "Nothing staged: only the message changes" |
| 5 | Bỏ tick Amend (chưa sửa gì) | Ô message trống trở lại |
| 6 | Tạo `b.txt`, stage, tick Amend, sửa message thành `First commit + b`, bấm Amend | `git log --stat` chỉ có **1** commit, gồm cả `a.txt` và `b.txt` |
| 7 | Message nhiều dòng, có dòng `#123 note` | `git log -1` giữ nguyên dòng `#123 note` |
| 8 | (Tùy chọn) repo đã push lên remote: tick Amend | Dòng vàng "This commit is already pushed."; bấm Amend → hộp thoại xác nhận, bấm Cancel thì không có gì thay đổi |
