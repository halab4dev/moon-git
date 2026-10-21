# Bước 16a — Lịch sử commit (chưa có graph)

Bắt đầu Giai đoạn 2. Bước 12–15 (credential, account, keyring, user.name/email, SSH key) **tạm hoãn** theo yêu cầu của bạn.

Bước 16 trong CLAUDE.md khá lớn, nên được chia làm ba:
- **16a (bước này):** đọc lịch sử bằng `git log`, tab **History**, danh sách ảo hóa, tải dần theo trang, nhãn branch/tag.
- **16b:** thuật toán xếp làn (lane layout) bằng TypeScript, là module riêng có test (sẽ cần thêm thư viện test, giải thích khi tới nơi), và vẽ cột graph.
- **16c:** chọn branch / remote / tag nào được hiển thị.

## Trạng thái
- `cargo test`: 98/98 pass (thêm 8 test: parse bản ghi + nhãn ref, HEAD detached + tên branch có `/`, bản ghi sai định dạng, repo rỗng, phân trang, branch phụ + tag + merge + thứ tự date-order, remote branch + HEAD detached, kích thước trang sai). `cargo clippy`: sạch. `pnpm build`: OK.
- Hiệu năng `git log` cho một trang 500 commit, đo trên repo 30.000 commit: khoảng 0,26 giây khi không có commit-graph, 0,015 giây khi có (chi tiết bên dưới).
- Giao diện: đã xác nhận (2026-10-22). Đã commit.

## Mục tiêu
- Màn hình repo có hai tab: **Changes** (như trước, kèm số file thay đổi) và **History**. Tab được nhớ lại.
- History liệt kê commit của mọi branch local, remote branch, tag và HEAD, mới nhất ở trên. Các cột: **Message** (có nhãn branch/tag phía trước), **Author**, **Date**, **Commit** (8 ký tự đầu).
- Nhãn có màu theo loại:
  - branch local: xanh lá; **đậm** nếu là branch đang checkout;
  - remote branch: xanh dương;
  - tag: vàng;
  - HEAD detached: đỏ.
- Mượt với repo lớn:
  - DOM chỉ chứa các dòng đang nhìn thấy;
  - tải 500 commit mỗi trang, tự tải trang tiếp khi cuộn gần hết.
- ↑/↓ để di chuyển chọn commit. Bấm vào commit để xem chi tiết là bước 18.

## Lệnh git

```
git log -z --format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%D%x1f%s
        --decorate=full --date-order --no-show-signature --no-color
        --max-count=<limit+1> --skip=<skip>
        --branches --remotes --tags [HEAD] --
```

| Phần | Ý nghĩa |
|---|---|
| `-z` | mỗi commit kết thúc bằng NUL |
| `%x1f` | ký tự 0x1F ngăn cách các trường, gần như không thể có trong tên hay message |
| `%H %P` | id commit, các id cha (cách nhau dấu cách; merge có ≥ 2) |
| `%aN %aE` | tên / email tác giả, **theo `.mailmap`** (chữ hoa N/E) |
| `%at` | thời điểm tác giả, giây Unix; frontend đổi sang giờ địa phương |
| `%D` + `--decorate=full` | ref trỏ vào commit, tên đầy đủ: `HEAD -> refs/heads/main, refs/remotes/origin/main, tag: refs/tags/v1` |
| `%s` | tiêu đề message, **đặt cuối cùng**: nếu message có ký tự lạ thì nó cũng chỉ nằm trong trường cuối (`splitn(7, …)`), không làm lệch các trường khác |
| `--date-order` | commit con luôn đứng **trước** commit cha (graph ở 16b cần điều này để vẽ đường từ trên xuống) |
| `--no-show-signature --no-color` | vô hiệu cấu hình cá nhân có thể chèn thêm chữ vào output |
| `--max-count=limit+1` | lấy dư 1 để biết còn trang sau hay không (`has_more`) |
| `--branches --remotes --tags` | không dùng `--all`, vì `--all` kéo cả commit của **stash** vào |
| `HEAD` | chỉ thêm khi repo đã có commit (repo rỗng thì `HEAD` chưa trỏ vào đâu, git báo lỗi) |
| `--` | hết phần revision; không gì phía sau bị hiểu là tên file |

Nhãn được lọc: `refs/remotes/origin/HEAD` chỉ cho biết branch mặc định của server nên bị bỏ (SmartGit cũng ẩn nó), `refs/stash` và notes cũng bị bỏ.

### Hiệu năng và commit-graph
`--date-order` bắt git đọc **toàn bộ** lịch sử để sắp xếp, kể cả khi chỉ lấy 500 commit. Đo trên repo 30.000 commit:

| | Trang đầu | Trang ở commit thứ 29.000 |
|---|---|---|
| Không có commit-graph | ~0,26 giây | ~0,26 giây |
| Có commit-graph | ~0,015 giây | |

**Commit-graph** (`.git/objects/info/commit-graph`) là file chỉ mục git tự tạo khi chạy `git gc` (mặc định bật). Repo dùng lâu thường đã có. Nếu repo khổng lồ mà vẫn chậm, chạy `git commit-graph write --reachable` một lần.

### Phân trang bằng `--skip`
Mỗi trang là một lệnh `git log` mới với `--skip=<số đã tải>`. Đơn giản, nhưng nếu có commit mới chen vào giữa hai lần tải thì danh sách có thể lệch một chút. Vì vậy mỗi lần status được làm mới (focus cửa sổ, Refresh, sau commit/pull…), History **tải lại từ đầu**. Số commit tải lại bằng số đang có (tối đa 2000), để vị trí cuộn không bị nhảy.

## Danh sách ảo hóa (virtualized list): cách hoạt động
Render 30.000 dòng HTML sẽ rất chậm, nhất là trên WebKitGTK (Ubuntu). Ý tưởng là chỉ tạo các dòng **đang nhìn thấy**:

```
viewport (cao 600px, overflow: auto)  ← thanh cuộn thật
└─ spacer (cao = số commit × 24px)    ← chỉ để thanh cuộn có đúng kích thước
   ├─ dòng 412 (position: absolute; top: 412 × 24px)
   ├─ dòng 413
   └─ … khoảng 25 dòng nhìn thấy + 10 dòng dư mỗi phía
```

- Mọi dòng cao **cố định 24px**, nên "dòng nào đang hiện" chỉ là một phép chia: `first = floor(scrollTop / 24) - 10`, `last = ceil((scrollTop + chiều cao) / 24) + 10`.
- Mỗi lần cuộn chỉ cập nhật `scrollTop`; `computed` tính lại `visible = commits.slice(first, last)` và Vue chỉ vẽ lại vài chục dòng.
- `ResizeObserver` theo dõi chiều cao viewport (khi kéo giãn cửa sổ).
- Khi `last` còn cách cuối danh sách dưới 100 dòng thì emit `loadMore`, và HistoryView tải trang tiếp.

Bước này **tự viết**, không thêm thư viện: khoảng 30 dòng code. Thư viện như `vue-virtual-scroller` giải quyết thêm các trường hợp dòng cao khác nhau, mà ở đây không cần.

## Luồng dữ liệu

```
RepoView: tab History  →  <HistoryView v-if> mount
HistoryView reload() → getLog(repoId, 0, 500) → invoke("get_log")
  └─ Rust commands/log.rs → run_blocking → git::log::read_log
       └─ git log -z … → parse_log → parse_record → parse_decorations / classify
  └─ LogPage { commits, hasMore } → commits (shallowRef)
CommitList: cuộn → scrollTop → first/last → visible (≈ 45 dòng trong DOM)
  └─ gần cuối → emit("loadMore") → HistoryView loadMore() → getLog(repoId, đã có, 500) → nối vào
Status làm mới (statusVersion++) → HistoryView watch(version) → reload()
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/log.rs` (mới) | `read_log`, `parse_log`, `parse_decorations`, `classify` + 8 test |
| `src-tauri/src/commands/log.rs` (mới) | command `get_log(repo_id, skip, limit)` |
| `src-tauri/src/models.rs` | `CommitSummary`, `CommitRef`, `RefKind`, `LogPage` |
| `src/types/log.ts`, `src/api/log.ts` (mới) | kiểu + wrapper |
| `src/components/CommitList.vue` (mới) | danh sách ảo hóa, nhãn ref, ↑/↓ |
| `src/views/HistoryView.vue` (mới) | tải trang, tải lại, chống trả lời cũ |
| `src/views/RepoView.vue` | tab Changes / History; Stage/Unstage chỉ hiện ở Changes; Fetch/Pull/Push và Refresh dùng chung |

## Khái niệm mới

### Rust
- **Slice pattern** `let [id, parents, …, subject] = fields[..] else { … }`: tách mảng thành các biến **và** kiểm tra đúng 7 phần tử trong một câu. Sai số lượng thì vào nhánh `else`. Java không có cú pháp tương đương; phải tự kiểm tra `length` rồi `fields[0]`, `fields[1]`…
- **`const` mảng tuple** `PREFIXES: [(&str, RefKind); 3]` + vòng `for`: thay chuỗi `if … else if …` bằng bảng dữ liệu. Thêm loại ref mới chỉ cần thêm một dòng.
- **`refs.extend(option)`**: `Option` cũng là một iterator có 0 hoặc 1 phần tử. `extend(Some(x))` thêm x, còn `extend(None)` không làm gì.
- **`truncate(n)`**: cắt `Vec` còn n phần tử, như `list.subList(0, n).clear()` ngược lại.

### Vue / TS
- **`shallowRef`**: chỉ theo dõi việc **gán lại** biến, không theo dõi từng field bên trong. Với hàng chục nghìn commit, `ref` thường sẽ bọc proxy cho mọi object, vừa tốn bộ nhớ vừa chậm. Đổi lại phải luôn **thay mảng mới** (`commits.value = commits.value.concat(...)`), không được `push`.
- **`v-show` vs `v-if`**: tab Changes dùng `v-show` (chỉ ẩn bằng CSS, giữ nguyên lựa chọn file và diff); tab History dùng `v-if` (tạo khi mở, hủy khi rời), nên mỗi lần mở tab là dữ liệu mới.
- **`ResizeObserver`**: API của trình duyệt, báo khi kích thước một phần tử thay đổi. Nhớ `disconnect()` khi unmount.
- **`Intl.DateTimeFormat`**: định dạng ngày giờ theo ngôn ngữ của máy, giống `DateTimeFormatter.ofLocalizedDateTime` trong Java.
- **Biến `generation`** chống trả lời cũ: mỗi lần tải lại thì tăng lên; câu trả lời mang số cũ bị bỏ qua. Cùng ý tưởng `lastRequest` ở DiffPanel (bước 8).

## Cách thử

App dev đang chạy bản mới. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Mở repo `moon-git`, bấm tab **History** | Commit mới nhất ở trên, nhãn `main` đậm (xanh lá) và `origin/main` nếu đã push |
| 2 | Bấm tab **Changes** rồi quay lại | Changes vẫn giữ file đang chọn; History tải lại |
| 3 | Đóng và mở lại app | Mở đúng tab lần trước |
| 4 | Commit một thay đổi ở tab Changes, rồi sang History | Commit mới ở đầu danh sách |
| 5 | Add Repository thư mục `/private/tmp/claude-501/-Users-halab-Projects-MoonGit-moon-git/b616aaa4-6e6a-4d5b-9fde-8a50c2649836/scratchpad/big` (30.000 commit, đã tạo sẵn để thử), mở History, kéo thanh cuộn xuống cuối | Cuộn mượt; dòng cuối có "… commit(s) loaded, more below · loading…" và số tăng dần theo từng 500; tới cuối thì không còn "more below" |
| 6 | Bấm chọn một commit, dùng ↑/↓ | Dòng chọn đổi, danh sách tự cuộn theo |
| 7 | Repo nháp `~/tmp/try-commit` với `git checkout --detach` | Nhãn `HEAD` màu đỏ trên commit hiện tại |
