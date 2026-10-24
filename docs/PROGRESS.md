# MoonGit — Kế hoạch và tiến độ

Cập nhật lần cuối: 2026-10-24

**Đang ở:** Giai đoạn 3, bước 23 (cherry-pick) đã code, chờ bạn thử; tiếp theo bước 24 (xử lý conflict). Bước 12–15 tạm hoãn theo yêu cầu của bạn.

Ký hiệu: ✅ xong · 🟡 đang làm / chờ xác nhận · ⏸ tạm hoãn · ⬜ chưa làm

Giải thích chi tiết từng bước: [docs/learning/](learning/README.md)

---

## Giai đoạn 1 — dùng được hằng ngày

| # | Bước | Trạng thái | Ghi chú |
|---|---|---|---|
| 0 | Chuẩn bị môi trường | ✅ | Rust 1.99.1, Node 24, pnpm 10, git 2.33, CLT (SDK 13.1). Còn cần `sudo xcode-select --switch /Library/Developer/CommandLineTools` |
| 1 | Khung Tauri v2 + Vue 3 + TS, layout sidebar \| nội dung, command `greet` | ✅ | Đã xác nhận giao diện (2026-10-21) |
| 2 | Lớp chạy git + kiểu lỗi chung | ✅ | `git/runner.rs`, `error.rs`, command `check_repo`. 7 test pass, clippy sạch. Đã xác nhận 4 trường hợp trên giao diện (2026-10-21) |
| 3 | Danh sách repo + thêm repo local | ✅ | `storage/repos.rs` (`repos.json`, ghi qua file tạm + rename), plugin dialog, Pinia store. 13 test pass, clippy sạch. Đã xác nhận trên giao diện (2026-10-21) |
| 4 | Nhóm / nhóm con repo | ✅ | `storage/repo_list.rs`, `repos.json` v2, menu chuột phải native, `NameDialog`. 23 test pass. Đã xác nhận (2026-10-21), thêm icon folder SVG. Chưa có: kéo-thả, di chuyển nhóm |
| 5 | Clone có tiến trình + hủy | ✅ | `git/operations.rs` (thread worker + Tauri event + hủy cả process group), `git/clone.rs`, `CloneDialog`. 36 test pass. Đã xác nhận (2026-10-21). Fetch/pull/push sẽ dùng lại `operations.rs` |
| 6 | Danh sách file thay đổi | ✅ | `git/status.rs` (porcelain v2 -z, `--no-optional-locks`), bảng `FileTable.vue` (Name/State/Location, sắp xếp theo cột, giống SmartGit), tự làm mới khi focus. 45 test pass. Đã xác nhận (2026-10-21) |
| 7 | Stage / unstage | ✅ | `git/stage.rs` (`--literal-pathspecs`, chia lô 100, repo chưa có commit), chọn dòng kiểu desktop, nút + menu chuột phải. 52 test pass. State đặt tên theo SmartGit. Đã xác nhận (2026-10-21) |
| 8 | Diff side-by-side | ✅ | Monaco (bạn chọn, so với CodeMirror / tự vẽ), tải lười; `git/diff.rs` (HEAD / Index / Working Tree, nhị phân, >5 MB, chặn đường dẫn ngoài repo); bố cục trên/dưới có thanh kéo; double-click → cửa sổ diff riêng (không chặn cửa sổ chính). 61 test pass |
| 9 | Commit + amend | ✅ | Message qua stdin (`-F -`), `--cleanup=whitespace`; kiểm tra trước (chưa stage / conflict / chưa có commit); lỗi thiếu danh tính → `IdentityUnknown`; xác nhận khi amend commit đã push. 71 test pass |
| 10 | Branch hiện tại + ahead/behind + fetch | ✅ | Header `status --branch` (không thêm lệnh git), `fetch --all --progress` có tiến trình/hủy, không `--prune`; fetch không chặn stage/commit; hủy gửi SIGTERM + SIGCONT. 77 test pass |
| 11 | Pull, push | ✅ | Pull = fetch (tiến trình/hủy) + `merge --ff-only` / hỏi Rebase–Merge khi tách nhánh (chọn sẵn theo `pull.rebase`); push không force, branch mới hỏi rồi `--set-upstream`; `Conflict` / `PushRejected`. 90 test pass |
| 12 | Thiết kế cách cấp credential | ⏸ | Chỉ đề xuất `GIT_ASKPASS` hay credential helper, chờ bạn chọn rồi mới code |
| 13 | Account theo server + keyring | ⏸ | Token chỉ lưu trong keyring, gắn account cho từng repo |
| 14 | `user.name` / `user.email` riêng cho từng repo | ⏸ | `git config --local` |
| 15 | Chọn SSH key theo server/repo | ⏸ | `GIT_SSH_COMMAND` |

## Giai đoạn 2 — lịch sử và branch

| # | Tính năng | Trạng thái |
|---|---|---|
| 16a | Lịch sử commit: `git/log.rs` (parser + test), tab History, danh sách ảo hóa tự viết + tải dần theo trang, nhãn branch/tag | ✅ 98 test pass |
| 16b | Graph: thuật toán xếp làn (lane layout) bằng TS, module riêng có test (thêm Vitest), vẽ cột graph | ✅ 11 test Vitest + 98 test Rust |
| 16c | Chọn branch/remote/tag hiển thị trong graph | ✅ Lần đầu = branch hiện tại, sau đó nhớ theo repo. 104 Rust + 15 Vitest |
| 17 | Filter theo hash / message, chế độ highlight (mặc định) và ẩn | ✅ 111 Rust + 15 Vitest |
| 18 | Bấm vào commit: thông tin, danh sách file, diff | ✅ 117 Rust + 15 Vitest |
| 19a | Branch: checkout (local / remote → tạo branch theo dõi), tạo branch mới (từ HEAD hoặc từ commit trong History) | ✅ 123 Rust + 15 Vitest |
| 19b | Branch: đổi tên, xóa local / remote (có xác nhận), đặt / bỏ upstream | ✅ 129 Rust + 17 Vitest |

## Giai đoạn 3 — thao tác nâng cao (sẽ chia nhỏ khi tới nơi)

| # | Tính năng | Trạng thái |
|---|---|---|
| 20a | Stash: tạo (message, tùy chọn gồm untracked), tab Stashes (danh sách + nội dung) | ✅ 136 Rust + 17 Vitest |
| 20b | Stash: apply, pop, drop (xác nhận), conflict khi apply | ✅ 141 Rust + 17 Vitest |
| 21a | Merge giống SmartGit: Fast-Forward / Create Merge-Commit (`--no-ff`) / Merge to Working Tree (`--no-ff --no-commit`) | ✅ 149 Rust + 17 Vitest |
| 21b | Đang merge: dòng báo + Abort (xác nhận), message điền sẵn, chọn merge commit / commit thường khi commit | ✅ 155 Rust + 17 Vitest |
| 22a | Rebase (không interactive) lên branch/tag/commit, xác nhận trước (cảnh báo khi commit đã push) | ✅ 161 Rust + 17 Vitest |
| 22b | Đang rebase: dòng báo, Continue / Skip / Abort | ✅ 166 Rust + 17 Vitest |
| 22c | Reset branch về một commit (History → chuột phải): Soft / Mixed / Hard; Hard xác nhận 2 lần, cảnh báo khi bỏ commit đã push. *Thêm vào lộ trình theo yêu cầu của bạn (2026-10-23), CLAUDE.md không có* | ✅ 173 Rust + 17 Vitest |
| 22d | Discard thay đổi của file (tab Changes): về HEAD, file mới vào Thùng rác (crate `trash`). *Thêm theo yêu cầu của bạn (2026-10-24), CLAUDE.md không có* | ✅ 179 Rust + 17 Vitest |
| 23 | Cherry-pick: Commit hoặc Apply to Working Tree (`--no-commit`); đang chờ sau conflict: Continue / Abort | ✅ 186 Rust + 17 Vitest |
| 24 | Xử lý conflict: phát hiện trạng thái, ours/theirs, công cụ ngoài, Continue/Abort | ⬜ |

---

## Quyết định đã chốt
- Tên app **MoonGit**, identifier `com.github.halab4dev.moongit`.
- Package manager: **pnpm**.
- Bỏ plugin `opener` của template (chưa cần).
- Pinia v4, dùng kiểu "setup store" (`ref`/`computed`/hàm).
- Danh sách repo lưu ở `<app data>/repos.json` (`{ version, repos }`); file hỏng được đổi tên thành `repos.json.broken`.
- Id repo sinh từ thời gian (hex), không dùng crate `uuid`.
- Bỏ repo khỏi danh sách không đụng tới thư mục trên ổ đĩa.
- Nhóm lưu phẳng (`parentId`); xóa nhóm → nội dung lên một cấp. Tên nhóm cùng cha không trùng (không phân biệt hoa thường).
- Thao tác trên sidebar dùng menu native của Tauri; nút × ở bước 3 được thay bằng "Remove from List".
- Thao tác dài: thread worker (`spawn_blocking`) + đọc stderr, thay vì `tokio::process` (dễ đọc hơn, không thêm thư viện). Mỗi thao tác có `opId`; tiến trình qua event `git-progress`.
- Hủy = SIGTERM cho cả process group (crate `rustix`, chỉ unix). Windows tạm chỉ dừng `git`.
- Clone lỗi/hủy → xóa thư mục đích chỉ khi nó không tồn tại trước khi clone.
- Command thao tác trên repo nhận `repo_id` (Rust tự tra đường dẫn), chạy nền qua `run_blocking`.
- Status làm mới khi cửa sổ được focus + nút Refresh (chưa dùng file watcher).
- Stage/unstage luôn dùng `--literal-pathspecs`; file conflict chưa cho stage (để bước 15).
- `vite.config.ts` dịch JS về mức `safari15` / `chrome105` (webview macOS 12 = WebKit hệ thống ≈ Safari 15.6, dù Safari app mới hơn). Kiểm tra bằng `jsc` của hệ thống.
- Fetch = `git fetch --all --progress`, không `--prune` (dọn branch đã xóa trên server sẽ là tùy chọn riêng sau).
- Pull: hỏi Rebase/Merge mỗi lần khi branch tách nhánh (bạn chọn), chọn sẵn theo git config; chỉ thiếu commit thì fast-forward không hỏi. Thực hiện bằng fetch + merge/rebase cục bộ với `@{upstream}`, không dùng `git pull`.
- Push không bao giờ force.
- Lịch sử: `git log --date-order` từ `--branches --remotes --tags HEAD` (không `--all`, để không kéo stash vào), trang 500 commit qua `--skip`; danh sách ảo hóa tự viết (không thêm thư viện).
- Diff dùng Monaco, chỉ nạp lõi + tô màu cú pháp + worker, tải khi mở diff lần đầu. Ba chế độ: Working Tree (Index→WT), Staged (HEAD→Index), Conflict (HEAD→WT).
- Cột State đặt tên theo trạng thái file của SmartGit (Staged, Added, Modified, Missing…); sắp theo mức quan trọng, Conflicted đầu tiên.
- Bỏ ví dụ `greet` ở bước 2, thay bằng form thử `check_repo` (form này sẽ được thay bằng "Add repository" ở bước 3).
- `AppError` là struct `{ kind, message }`; git chạy với `GIT_TERMINAL_PROMPT=0` và `LC_ALL=C`.
- Test dùng crate `tempfile` (chỉ dev-dependency).
- Test TypeScript dùng **Vitest** (devDependency), chạy `pnpm test`.
- Chọn ref cho History: lần đầu = branch hiện tại + upstream (bạn yêu cầu), sau đó lưu theo repo trong localStorage (`null` = tất cả, kể cả branch mới); Rust chỉ nhận ref đang tồn tại + `--end-of-options`.
- Filter lịch sử: một ô cho cả tiền tố id (khi toàn hex) lẫn message đầy đủ, không phân biệt hoa thường; tìm ở Rust trên toàn lịch sử; chế độ Hide viết lại cha về tổ tiên khớp gần nhất.
- Chi tiết commit: merge so với cha thứ nhất (như SmartGit); chỉ nhận commit id dạng hex.
- Branch: dùng `git switch` (không `checkout`), `--no-guess`, không bao giờ `--force`; checkout remote = tạo branch local cùng tên theo dõi nó. Xóa branch chưa merge: hỏi lần 2 rồi mới `--force`. Xóa trên server dùng `refs/heads/<tên>` để không trúng tag.
- Stash: kiểm tra trước khi stash (git báo thành công cả khi không có gì); nhận diện stash bằng id, không bằng số thứ tự; xem nội dung bằng `CommitDetails` (W so với HEAD lúc stash, U cho file untracked). Apply không dùng `--index` (mọi thay đổi về dạng chưa stage); pop bị conflict thì stash được giữ.
- Merge làm giống SmartGit (bạn chọn, 2026-10-23): ba lựa chọn Fast-Forward / Create Merge-Commit / Merge to Working Tree; mặc định luôn là Create Merge-Commit (bạn chọn). Trạng thái merge đọc từ `MERGE_HEAD`/`MERGE_MSG` (qua `--git-path`) trong `read_status`; Simple commit = xóa `MERGE_HEAD` rồi commit. App tự viết message merge (truyền tên ref đầy đủ cho git).
- Rebase: luôn hỏi trước (đếm commit sẽ replay và commit đã push); từ chối khi có thay đổi chưa commit, `--no-autostash`; không merge/rebase/commit khi đang rebase dở. Continue chạy với `GIT_EDITOR=true` (giữ message cũ); Skip / Abort luôn hỏi trước. Thanh trạng thái dùng chung `OperationBanner`.
- Reset được thêm vào lộ trình (bước 22c, bạn yêu cầu 2026-10-23): ba kiểu như SmartGit; Hard luôn hỏi lại lần hai. Mặc định Mixed; truyền commit id đầy đủ cho `git reset` (git 2.33 không nhận `--end-of-options` ở lệnh này).
- Discard (bước 22d, bạn yêu cầu 2026-10-24): làm giống SmartGit, về HEAD cả staged lẫn unstaged; file HEAD không có thì chuyển vào Thùng rác (bạn chọn, crate `trash`), không xóa hẳn. Chỉ xử lý đường dẫn có trong status; không discard file conflict.
- Cherry-pick: một commit từ History; merge commit lấy theo cha thứ nhất (`--mainline 1`); cherry-pick rỗng (đã có) tự hủy và báo; trạng thái chờ đọc từ `CHERRY_PICK_HEAD`.
- Graph: làn không dịch ngang, cột trống được dùng lại; `LaneLayout` tính dần theo trang; vẽ SVG mỗi dòng, tối đa 20 làn hiển thị.

## Việc còn treo
- [ ] Bạn chạy `sudo xcode-select --switch /Library/Developer/CommandLineTools`.
- [x] Commit đầu tiên `dd92830` (bước 1–2).
- [x] Bước 3 đã xác nhận và commit.
- [x] Bước 4, 5 đã xác nhận.
- [x] Bước 6 đã xác nhận và commit.
- [x] Bước 7 đã xác nhận và commit.
- [x] Bước 8 đã xác nhận và commit.
- [x] Bước 9 đã xác nhận và commit.
- [x] Bước 10 đã xác nhận và commit.
- [x] Bước 11 đã xác nhận và commit.
- [x] Bước 16a đã xác nhận và commit.
- [x] Bước 16b đã xác nhận và commit.
- [x] Bước 16c đã xác nhận và commit.
- [x] Bước 17 đã xác nhận và commit.
- [x] Bước 18 đã xác nhận và commit.
- [x] Bước 19a đã xác nhận và commit.
- [x] Bước 19b đã xác nhận và commit.
- [x] Bước 20a đã xác nhận và commit.
- [x] Bước 20b đã xác nhận và commit.
- [x] Bước 21a đã xác nhận và commit.
- [x] Bước 21b đã xác nhận và commit.
- [x] Bước 22a đã xác nhận và commit.
- [x] Bước 22b đã xác nhận và commit.
- [x] Bước 22c đã commit (2026-10-24, khi bạn chuyển sang 22d).
- [x] Bước 22d đã xác nhận và commit.
- [x] Bước 23 đã xác nhận và commit.
- [ ] Bước 12–15 (credential, account + keyring, user.name/email theo repo, SSH key) **tạm hoãn** (2026-10-21), làm lại sau giai đoạn 2 hoặc khi bạn yêu cầu.
