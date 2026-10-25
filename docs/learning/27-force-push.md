# Bước 27 — Force push (an toàn, "with lease")

*Bạn yêu cầu thêm (2026-10-26), CLAUDE.md không có.*

## Trạng thái
- `cargo test`: 208/208 pass. Thêm 5 test (server bare + hai bản clone tạm):
  - preview liệt kê đúng commit sẽ mất trên server (`theirs.txt`), số commit local, id server lúc fetch;
  - force push thay branch trên server bằng branch local; sau đó ahead/behind = 0/0;
  - **có người push sau lần fetch cuối** → git từ chối (`PushRejected`, "since your last fetch"), commit của họ vẫn còn;
  - `expected` không phải id đầy đủ (`""`, `abc123`, `--force`, `refs/heads/main`) → từ chối trước khi chạy git;
  - không có upstream / không có gì khác server → báo lỗi.
- `cargo clippy`: sạch. `pnpm test`: 30/30. `pnpm build`: OK.
- Giao diện: bạn đã thử, OK. Đã commit.

## Khi nào cần force push
Push thường chỉ **thêm** commit lên server. Khi bạn đã **viết lại lịch sử** của branch đã push (rebase, amend, reset), branch local và branch trên server tách nhau. Push thường bị từ chối ("Pull first"). Force push nghĩa là: "server hãy bỏ branch của mình, lấy đúng branch local của tôi".

Nguy hiểm: commit chỉ có trên server sẽ **mất khỏi branch đó**, kể cả commit của người khác.

## `--force` và `--force-with-lease`
| | `--force` | `--force-with-lease=<ref>:<id>` (app dùng) |
|---|---|---|
| Ghi đè server | luôn luôn | chỉ khi server **vẫn đúng ở `<id>`** |
| Có người push sau lần fetch cuối của bạn | commit của họ mất, bạn không hề biết | git từ chối (`stale info`), không mất gì |

- App lấy `<id>` = vị trí `origin/main` lúc fetch cuối, đúng cái bạn **đã thấy** trong hộp xác nhận.
- Nên lệnh chỉ xóa đúng những commit đã liệt kê cho bạn, không hơn.
- `<id>` được truyền **rõ ràng**, không dùng dạng `--force-with-lease` trống. Dạng trống sẽ so với `origin/main` **lúc chạy**, mà nếu có tool khác fetch ngầm thì lớp bảo vệ mất tác dụng.

## Giao diện
- Bên cạnh nút **Push** có nút **▾** → menu **Force Push…**.
  - Mờ kèm lý do khi: HEAD detached; branch chưa có trên server; upstream đã bị xóa trên server. Trong hai trường hợp sau, dùng Push thường để tạo lại.
- Bấm vào: app tính trước (preview) dựa trên lần fetch cuối:
  - **Không commit nào bị mất** (server không có gì mà local không có): báo "Force Push Not Needed — Use Push instead". Không push.
  - **Có commit bị mất**: hộp cảnh báo:

```
Replace origin/main with your local "main"?

2 commit(s) on origin/main will be removed from it:
  • 1a2b3c4d Fix login
  • 5e6f7a8b Update README

Anyone who already pulled them will have to repair their copy.
If someone pushed after your last fetch, the push is refused and nothing is overwritten.
                                              [Cancel] [Force Push]
```

- Tối đa 10 commit, còn lại ghi "… and N more".
- Đang chạy: dòng tiến trình + **Cancel**, giống Push.
- Xong: ghi "Force pushed" cạnh các nút; header về ↑0 ↓0.
- Bị từ chối vì server đã đổi: thông báo "The server branch has changed since your last fetch (someone else pushed). Nothing was overwritten. Fetch, look at the new commits, then decide again."

## Lệnh git

| Việc | Lệnh |
|---|---|
| Upstream của branch hiện tại | `git for-each-ref --format=%(upstream:remotename)%1f%(upstream:remoteref)%1f%(upstream)%1f%(upstream:short) refs/heads/<branch>` |
| Server ở đâu lúc fetch cuối | `git rev-parse --verify --quiet refs/remotes/origin/main^{commit}` |
| Đếm commit | `git rev-list --count <id>..HEAD --` / `HEAD..<id>` |
| Commit sẽ mất | `git log -z --max-count=10 --format=%H%x1f%s HEAD..<id> --` |
| Force push | `git push --progress --force-with-lease=refs/heads/main:<id> -- origin HEAD:refs/heads/main` |

### Ghi chú
- **`%(upstream:remotename)` / `%(upstream:remoteref)`**: tên remote (`origin`) và tên branch **trên server** (`refs/heads/main`), đọc thẳng từ config `branch.<name>.remote` / `.merge`. Branch local `feature` có thể theo dõi `origin/khac`; app push đúng vào `refs/heads/khac`.
- **Refspec `HEAD:refs/heads/main` không có `+`**: dấu `+` nghĩa là "ép buộc" và sẽ vượt qua lease. Ở đây chỉ lease cho phép ghi đè.
- **`--`** trước `origin`: tên remote không bao giờ bị hiểu là option.
- `expected` từ giao diện phải là **id đầy đủ** (40 hoặc 64 ký tự hex). Một tên ref hay chuỗi `--force` không lọt vào được.
- Lỗi `stale info` được nhận ra trong `operations.rs` (chỗ đã đổi lỗi "fetch first" thành `PushRejected` ở bước 11).
- Quyết định cũ "Push never forces" vẫn đúng với nút **Push**. Force push là thao tác riêng, luôn có xác nhận (CLAUDE.md: thao tác phá hủy phải có hộp thoại).

## Luồng dữ liệu

```
▾ → pushMenu → Force Push… → forcePush()
  └─ forcePushPreview → invoke("force_push_preview") → push::force_push_preview
       upstream_of_current (for-each-ref) → tracking_commit (rev-parse) → rev-list / log
     ← ForcePushPreview { branch, upstream, ahead, lostCount, lost[], expected }
  └─ lostCount = 0 → message "Use Push instead"
  └─ ask(...) → Force Push
       └─ forcePushRepo(opId, repoId, expected) → invoke("force_push_repo") → push::force_push
            git push --force-with-lease=<ref>:<expected> -- <remote> HEAD:<ref>   (git-progress + Cancel)
       ← OK → "Force pushed" → emit("finished") → RepoView tải lại status
       ← stale info → PushRejected → RepoView hiện lỗi
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/push.rs` | `force_push_preview`, `force_push`, `Upstream`, `upstream_of_current`, `tracking_commit` + 5 test |
| `src-tauri/src/git/operations.rs` | `stale info` → `PushRejected` với câu dễ hiểu |
| `src-tauri/src/models.rs` | `ForcePushPreview`, `LostCommit` |
| `src-tauri/src/commands/push.rs`, `lib.rs` | command `force_push_preview`, `force_push_repo` |
| `src/types/sync.ts`, `src/api/sync.ts` | kiểu + `forcePushPreview`, `forcePushRepo` |
| `src/components/SyncButtons.vue` | nút ▾, menu, xác nhận, chạy có tiến trình |

## Khái niệm mới

### Rust
- **Closure dùng biến bên ngoài và trả `Result`** (`let count = |range: &str| -> Result<u32, AppError> { … run_git(repo, …)? … }`): một "hàm nhỏ" trong hàm, đọc được `repo` mà không cần truyền vào. Có thể dùng `?` bên trong vì closure tự khai báo kiểu trả về. Giống lambda của Java, nhưng lambda Java không ném được checked exception một cách gọn như vậy.
- **`matches!(expected.len(), 40 | 64)`**: kiểm tra một giá trị có khớp một trong các mẫu không, thay cho `len == 40 || len == 64`.
- **Struct riêng tư trong module** (`struct Upstream`, không `pub`): chỉ dùng nội bộ trong `push.rs`, gom 5 giá trị đi cùng nhau thay vì trả tuple 5 phần tử.

### Vue / TS
- **Mục menu native bị mờ** (`enabled: false`) và ghi lý do ngay trong chữ, giống nút bị mờ có `title` ở các bước trước.
- **Ghép nhiều dòng bằng mảng + `join("\n")`**: dễ đọc hơn một chuỗi dài với nhiều `\n`, và dễ thêm dòng có điều kiện (`… and N more`).

## Cách thử

App dev (bạn đang chạy) tự build lại phần Rust. Bấm ⌘R trước khi thử.

Chuẩn bị một repo có remote mà bạn được phép ghi đè, ví dụ một repo thử trên GitHub, hoặc server bare local: `git init --bare /tmp/srv.git`, rồi clone nó hai lần: A và B.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Branch chưa push (không upstream) → ▾ | "Force Push… (This branch is not on the server yet: use Push)", mờ |
| 2 | Branch đã push, không đổi gì → ▾ → Force Push… | Báo "nothing to push" |
| 3 | Commit mới (chưa push) → Force Push… | "Force Push Not Needed — Use Push instead" |
| 4 | Ở B commit + push. Ở A: Fetch, commit khác (tách nhánh) → Force Push… | Cảnh báo liệt kê commit của B sẽ mất |
| 5 | Cancel | Không có gì thay đổi |
| 6 | Làm lại, chọn Force Push | "Force pushed"; header ↑0 ↓0; History: commit của B không còn trên `origin/main` |
| 7 | Ở B commit + push lại. Ở A **không Fetch**, commit khác → Force Push… → Force Push | Bị từ chối: "The server branch has changed since your last fetch…", commit của B vẫn còn |
| 8 | Sau #7: Fetch → Force Push… | Lần này cảnh báo có thêm commit mới của B |
