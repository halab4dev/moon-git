# Bước 17 — Filter lịch sử theo commit id / message

## Trạng thái
- `cargo test`: 111/111 pass. Thêm 7 test cho `git/search.rs`:
  - khớp message / tiền tố hash;
  - tách subject từ message;
  - tìm trên toàn lịch sử, kể cả thân message, theo thứ tự mới → cũ;
  - chế độ ẩn nối commit khớp với tổ tiên khớp gần nhất;
  - viết lại cha của merge mà không bị trùng;
  - phân trang + theo ref đã chọn;
  - repo rỗng.
- `cargo clippy`: sạch. `pnpm test`: 15/15. `pnpm build`: OK.
- Một lần quét toàn bộ repo 30.000 commit kèm message: khoảng 0,5 giây phía git.
- Giao diện: đã xác nhận (2026-10-22). Đã commit.

## Mục tiêu
- Thanh công cụ History có thêm **ô lọc** "Filter: message or commit id", nút chọn chế độ **Highlight | Hide others**, số kết quả và nút **↑ / ↓**.
- **Khớp** khi:
  - commit id **bắt đầu bằng** chữ bạn gõ (chỉ khi chữ đó toàn ký tự hex, ví dụ `a1b2c3`), **hoặc**
  - **message đầy đủ** (tiêu đề + thân) **chứa** chữ đó.
  - Không phân biệt hoa thường.
- **Highlight** (mặc định): graph và danh sách giữ nguyên; commit không khớp bị **làm mờ**. Số kết quả tính trên **toàn bộ lịch sử**, kể cả phần chưa tải.
  - **Enter / ↓**: tới kết quả tiếp theo phía dưới commit đang chọn.
  - **Shift+Enter / ↑**: tới kết quả phía trên.
  - Hết thì quay vòng. Kết quả nằm ở trang chưa tải thì app tự tải tới đó rồi cuộn tới.
- **Hide others**: chỉ hiện commit khớp; graph vẫn **nối liền** (xem phần viết lại cha bên dưới).
- Gõ xong 0,3 giây mới tìm, để không phải quét lịch sử ở mỗi phím. **Esc** xóa ô lọc. Chế độ được nhớ.

## Vì sao lọc ở Rust, không lọc trên phần đã tải
Danh sách chỉ tải 500 commit mỗi lần (bước 16a). Nếu chỉ lọc phần đã tải, "3 kết quả" có thể là sai: có khi còn 200 kết quả ở các trang chưa tải. Vì vậy Rust quét **mọi commit** của các ref đang chọn một lần, kèm message đầy đủ:

```
git log -z --format=%H%x1f%P%x1f%aN%x1f%aE%x1f%at%x1f%D%x1f%B  (cùng các option như log.rs)
```

- `%B` là message gốc (tiêu đề + thân), đặt ở trường cuối, giống `%s` trước đây. Tiêu đề được tách lại bằng `subject_of()`: đoạn đầu tiên, các dòng nối bằng dấu cách, giống `%s` của git.
- Hai command:
  - `find_commits(refs, text)` → **id** của mọi commit khớp (cho Highlight).
  - `get_log(..., filter)` → khi có `filter`, trả về **trang** commit khớp với cha đã viết lại (cho Hide).
- Phần dựng lệnh `git log` (option cố định + ref đã chọn + `--end-of-options`) được tách khỏi `read_log` thành `run_log` để hai nơi dùng chung.
- Không dùng `git log --grep`, vì `--grep` không gộp được điều kiện "**hoặc** id bắt đầu bằng …".

### Chi phí
Mỗi lần tìm là một lần quét toàn bộ (~0,5 s cho 30.000 commit). Ở chế độ Hide, **mỗi trang** cũng quét lại từ đầu. Nếu thấy chậm trên repo thật của bạn, có thể thêm bộ nhớ đệm phía Rust sau.

## Viết lại cha (chế độ Hide)
Ẩn commit không khớp thì cha thật của commit khớp thường không có trong danh sách. Nếu để nguyên, graph chỉ còn các đường "đi xuống mãi" không nối vào đâu. Giống `git log --graph` khi lọc, mỗi commit khớp được nối với **các tổ tiên khớp gần nhất**:

```
lịch sử thật          sau khi lọc "match"
match four  ●          ● match four
three       ●          |
match two   ●          ● match two          (cha của four = two)
one         ●          (two không có cha khớp nào → không có đường xuống)
```

Thuật toán (`rewrite_parents`), không dùng đệ quy:
- Danh sách là mới → cũ và con luôn đứng trước cha (`--date-order`). Vì vậy đi **ngược từ cuối lên** thì cha luôn được xử lý trước con.
- Với mỗi commit `i` tính `reach[i]`: các commit khớp gặp **đầu tiên** khi đi xuống từ `i`.
  - `i` khớp → `reach[i] = [i]`; cha mới của `i` = hợp của `reach[cha]` với mọi cha thật.
  - `i` không khớp → `reach[i]` = hợp của `reach[cha]`.
- Hợp được giữ đúng thứ tự cha (cha thứ nhất trước) và bỏ trùng. Ví dụ một merge mà cả hai nhánh đều dẫn về cùng một commit khớp thì chỉ còn một cha.

Không dùng đệ quy vì lịch sử 50.000 commit thẳng hàng sẽ làm tràn stack (đệ quy sâu 50.000 tầng). Vòng lặp ngược thì không có giới hạn đó.

## Frontend
- **`HistoryFilter.vue`** (mới): ô lọc, hai nút chế độ, số kết quả, ↑/↓. Phát `jump(+1 | -1)`.
- **`HistoryView.vue`**:
  - `filterText` (đang gõ) → sau 0,3 s → `appliedFilter` (đang dùng).
  - `logFilter` = `appliedFilter` chỉ khi ở chế độ Hide → truyền vào `getLog`; đổi thì tải lại.
  - `search()` (Highlight) gọi `findCommits` → `matchIds` (thứ tự lịch sử) → `matchSet` cho CommitList.
  - `jump()`: tìm kết quả gần nhất phía dưới/trên commit đang chọn. Kết quả chưa tải được coi là "ở dưới cùng", và app tải từng trang tới khi thấy nó.
  - `loadMore()` giờ **trả về lần tải đang chạy** nếu đã có (biến `pending`). Nếu không, vòng lặp chờ trong `jump()` sẽ quay vô hạn đúng lúc danh sách đang tự tải trang tiếp theo.
- **`CommitList.vue`**:
  - class `dim` (mờ 35%) cho commit không có trong `matches`.
  - `watch(selected)` cuộn commit được chọn vào tầm nhìn. Nhảy từ xa thì đặt nó ở giữa màn hình; bước một dòng (↑/↓) thì chỉ cuộn vừa đủ.

## Luồng dữ liệu

```
gõ "login" → (300 ms) → appliedFilter = "login"
Highlight:
  search() → findCommits(repoId, refs, "login") → invoke("find_commits")
     └─ Rust search::find_matches → walk (git log … %B) → matches_filter → [id, id, …]
  matchSet → CommitList: hàng không khớp có class "dim"
  Enter → jump(+1) → (loadMore… nếu cần) → selected = id → CommitList cuộn tới
Hide:
  logFilter = "login" → reload() → getLog(…, filter="login") → invoke("get_log")
     └─ Rust search::read_filtered_log → walk → matched → rewrite_parents → trang 500
  → LaneLayout vẽ graph với cha đã viết lại
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src-tauri/src/git/search.rs` (mới) | `matches_filter`, `find_matches`, `read_filtered_log`, `rewrite_parents`, `subject_of` + 7 test |
| `src-tauri/src/git/log.rs` | tách `run_log` dùng chung |
| `src-tauri/src/commands/log.rs` | `get_log` nhận `filter`; command mới `find_commits` |
| `src/api/log.ts`, `src/types/log.ts` | `findCommits`, tham số `filter`, kiểu `FilterMode` |
| `src/components/HistoryFilter.vue` (mới) | ô lọc + chế độ + ↑/↓ |
| `src/views/HistoryView.vue` | trạng thái filter, chờ gõ xong, tìm, nhảy; `pending` cho các lần tải |
| `src/components/CommitList.vue` | làm mờ, cuộn theo commit được chọn |

## Khái niệm mới

### Rust
- **`std::mem::take(&mut x)`**: lấy giá trị ra và để lại giá trị rỗng (`String::new()`, `Vec::new()`). Dùng khi muốn **chuyển** dữ liệu ra khỏi một struct mà không copy, ví dụ chuyển message ra khỏi `commit.subject` trước khi gán subject mới.
- **Struct update `CommitSummary { parents: …, ..commit }`**: tạo struct mới, đổi một field, các field còn lại **chuyển** từ `commit` sang (không copy).
- **`HashMap<&str, usize>`**: khóa là tham chiếu mượn vào các `String` có sẵn, không copy id. Rust đảm bảo map không sống lâu hơn danh sách mà nó mượn.
- **Duyệt ngược** `for i in (0..n).rev()`.
- **`.collect::<Vec<_>>()`**: "turbofish" chỉ rõ kiểu cần thu về; `_` để Rust tự suy ra kiểu phần tử.

### Vue / TS
- **Chờ gõ xong (debounce)** bằng `setTimeout` / `clearTimeout`: mỗi phím hủy hẹn giờ cũ và đặt hẹn giờ mới, nên chỉ lần gõ cuối cùng mới chạy. Hủy hẹn giờ khi unmount.
- **`computed` làm "công tắc"**: `logFilter` chỉ có giá trị ở chế độ Hide. `watch(logFilter)` tải lại danh sách đúng khi cần: gõ trong chế độ Highlight không làm tải lại.
- **Dùng chung một Promise đang chạy**: `pending` giữ lần tải hiện tại; ai gọi `loadMore()` lúc đó sẽ chờ cùng lần tải ấy thay vì bị bỏ qua.
- **`?? Infinity`**: commit chưa tải không có vị trí → coi như ở vị trí vô cùng (dưới cùng). Nhờ vậy một phép so sánh xử lý được cả hai trường hợp.
- **Phím ghép** `@keydown.enter.exact` (Enter không kèm phím khác) vs `@keydown.shift.enter`.

## Cách thử

App dev đã được khởi động lại. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `moon-git` → History → gõ `step` | Sau một chút: "N matches"; commit không có chữ "step" bị mờ; graph giữ nguyên |
| 2 | Enter vài lần, rồi Shift+Enter | Chọn nhảy xuống / lên giữa các kết quả, có quay vòng |
| 3 | Copy 7 ký tự đầu của một commit id (cột Commit), dán vào ô lọc | Đúng 1 match, Enter nhảy tới đúng commit đó |
| 4 | Bấm **Hide others** | Chỉ còn các commit khớp; footer "N matching commit(s) loaded"; graph nối liền |
| 5 | Repo `branchy` → Hide others → gõ `f` | f1, f2 và các merge có chữ "f" ("Merge branch 'feature'"…); đường nối giữa các commit còn lại không bị đứt |
| 6 | Repo `big` (30.000 commit) → Highlight → gõ `commit 100` | Khoảng 0,5–1 giây ra "111 matches"; Enter → kết quả đầu tiên là "commit 10099", nằm ở khoảng dòng 19.900, nên app tự tải khoảng 40 trang rồi cuộn tới |
| 7 | Repo `big` → gõ `commit 1` → Hide others → kéo xuống cuối | Chỉ các commit có "commit 1…"; tải dần theo trang |
| 8 | Esc trong ô lọc | Xóa filter, mọi commit hiện bình thường |
| 9 | Đóng/mở lại app | Chế độ Highlight/Hide được nhớ; ô lọc trống |
