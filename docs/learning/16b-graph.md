# Bước 16b — Graph lịch sử: thuật toán xếp làn

## Trạng thái
- `pnpm test` (Vitest): 11/11 pass. Gồm: lịch sử thẳng, branch + merge, màu theo branch, branch chưa merge nằm cạnh nhau, dùng lại cột trống, merge vào làn đã có, octopus merge (3 cha), cha chưa được tải, thêm theo từng trang cho kết quả giống hệt tính một lần, và một lịch sử ngẫu nhiên 400 commit. **Mọi** test đều kiểm tra "đường không bị đứt" giữa các dòng.
- `cargo test`: 98/98 (không đổi). `pnpm build`: OK.
- Hiệu năng: 50.000 commit tính một lần mất khoảng 60–100 ms; mỗi trang 500 commit tính thêm mất dưới 25 ms.
- Đã so với `git log --graph` trên một repo có 3 branch, 2 merge, 1 branch chưa merge (xem cuối file): cùng cấu trúc.
- Giao diện: đã xác nhận (2026-10-22). Đã commit.

## Thư viện mới: Vitest (devDependency)
- **Là gì:** test runner cho JavaScript/TypeScript, tương tự JUnit cho Java.
- **Vì sao chọn:** do chính nhóm Vite làm; đọc luôn `vite.config.ts` và chạy được TypeScript mà **không cần cấu hình gì**. Jest phổ biến hơn, nhưng muốn chạy TS và ESM phải thêm `ts-jest` hoặc babel và tự cấu hình.
- **Không đi vào app:** chỉ dùng khi chạy test, không có trong bản build.
- **Chạy:** `pnpm test`, tức `vitest run`: tìm mọi file `*.test.ts`, chạy một lần rồi thoát.

So với JUnit:

| JUnit 5 | Vitest |
|---|---|
| `class LaneLayoutTest` | `describe("layoutGraph", () => { … })` |
| `@Test void keepsStraightHistory()` | `it("keeps a straight history in one lane", () => { … })` |
| `assertEquals(1, layout.maxLanes)` | `expect(layout.maxLanes).toBe(1)` |
| `assertEquals(list1, list2)` (so nội dung) | `expect(a).toEqual(b)` (so sâu từng field) |

## Thuật toán xếp làn

**Đầu vào:** danh sách commit mới nhất ở trên. Commit con luôn đứng trước commit cha, nhờ `--date-order` ở bước 16a. Mỗi commit chỉ cần `id` và `parents`.

**Ý tưởng:** đi từ trên xuống, giữ một mảng **làn đang mở** (`lanes`). Mỗi phần tử là một cột: hoặc `null` (cột trống), hoặc `{ waitsFor: <id>, color }`, nghĩa là "đường ở cột này đang đi xuống để gặp commit `<id>`".

Với mỗi commit:
1. **Tìm cột đang chờ nó.** Có thể có nhiều cột cùng chờ, khi nhiều branch cùng tách ra từ commit này. Tất cả các đường đó đổ vào chấm của commit; chấm nằm ở cột **trái nhất**. Không cột nào chờ → commit là **đầu một branch**, lấy cột trống đầu tiên (hoặc thêm cột mới bên phải) và một màu mới.
2. **Nửa trên của dòng:** các cột đang chờ commit này vẽ đường **vào chấm** (`top`). Các cột khác vẽ đường **thẳng xuyên qua** (`full`).
3. **Nửa dưới:** cột của commit tiếp tục đi xuống, chờ **cha thứ nhất**, giữ nguyên màu (`bottom`).
4. **Cha thứ hai trở đi** (merge commit): nếu đã có cột đang chờ cha đó thì nối đường chéo vào cột ấy; nếu chưa có thì mở cột mới với màu mới.
5. Bỏ các cột trống ở mép phải.

**Các làn không bao giờ dịch sang ngang.** Đường đi qua một dòng luôn ở nguyên cột của nó; cột được giải phóng sẽ được branch kế tiếp dùng lại. Nhờ vậy chỉ cần vẽ đường chéo ở chỗ tách và chỗ nhập.

Ví dụ: một merge `m` gộp `a` (main) và `b` (side), cả hai tách ra từ `base`:

```
dòng  commit  lanes trước        vẽ                              lanes sau
0     m       []                 chấm cột 0; ra 0→0, ra 0→1      [chờ a, chờ b]
1     a       [chờ a, chờ b]     vào 0→0; cột 1 xuyên qua; ra 0→0 [chờ base, chờ b]
2     b       [chờ base, chờ b]  cột 0 xuyên qua; vào 1→1; ra 1→1 [chờ base, chờ base]
3     base    [chờ base, chờ base] vào 0→0, vào 1→0 (nhập)       []

m  ●
   |\
a  ● |
   | ●  b
   |/
base ●
```

### Kết quả cho mỗi dòng (`GraphRow`)
- `lane`: cột của chấm; `color`: chỉ số màu.
- `lines`: các mảnh đường trong dòng, mỗi mảnh `{ from, to, kind, color }`, với `kind` là `top` (mép trên → chấm), `bottom` (chấm → mép dưới) hoặc `full` (xuyên từ mép trên xuống mép dưới).

Component vẽ chỉ việc biến mỗi mảnh thành một `<path>` SVG, không cần biết gì về git.

### Tính dần theo trang
History tải 500 commit mỗi lần (bước 16a). Nếu mỗi lần lại tính lại cả 30.000 commit thì càng cuộn càng giật. Vì vậy thuật toán là class **`LaneLayout`** giữ trạng thái `lanes` giữa các lần gọi: `add(trangMới)` tính tiếp từ chỗ dừng. Test "gives the same result when commits are added page by page" chia 400 commit thành các trang 37 commit và kiểm tra kết quả **giống hệt** tính một lần.

### Kiểm tra "đường không bị đứt"
Helper `expectConnected()` trong file test kiểm tra rằng tập các cột có đường ra ở mép dưới dòng *i* **đúng bằng** tập các cột có đường vào ở mép trên dòng *i+1*, và dòng đầu tiên không có đường nào "từ trên trời rơi xuống". Mọi kịch bản đều chạy kiểm tra này, kể cả lịch sử ngẫu nhiên 400 commit. Lỗi nối đường ở bất kỳ chỗ nào đều bị bắt.

## Vẽ (`GraphCell.vue`)
- Mỗi dòng là một `<svg>` cao 24px; mỗi làn rộng 14px; tâm của cột *n* ở `x = n × 14 + 7`.
- Mảnh thẳng dùng `M x1 y1 L x2 y2`. Mảnh đổi cột dùng **đường cong Bézier bậc 3** `M x1 y1 C x1 yGiữa, x2 yGiữa, x2 y2`, cho hình chữ S mềm thay vì gấp khúc.
- Chấm tròn bán kính 4px; **merge commit** có chấm rỗng.
- 8 màu, lặp vòng theo chỉ số màu.
- Cột Graph rộng theo dòng rộng nhất, tối đa 20 làn (280px). Repo có hàng chục branch song song sẽ bị cắt bớt thay vì đẩy cột Message ra khỏi màn hình.
- Danh sách vẫn ảo hóa như bước 16a: chỉ khoảng 45 SVG tồn tại cùng lúc.

## Luồng dữ liệu

```
HistoryView reload()/loadMore()
  └─ getLog(...) → page.commits
  └─ layout.add(page.commits)          ← LaneLayout (TS thuần, không reactive)
  └─ graphRows = layout.rows.slice()   ← shallowRef: mảng mới để Vue nhận ra
CommitList: visible rows i ∈ [first, last)
  └─ <GraphCell :row="graph[i]" :merge="commit.parents.length > 1" :width>
       └─ <path> cho mỗi mảnh + <circle> cho chấm
```

## File đã tạo / sửa

| File | Việc |
|---|---|
| `src/graph/laneLayout.ts` (mới) | `LaneLayout` (tính dần), `layoutGraph` (tính một lần), các kiểu `GraphRow` / `GraphLine` |
| `src/graph/laneLayout.test.ts` (mới) | 11 test Vitest |
| `src/components/GraphCell.vue` (mới) | vẽ một dòng graph bằng SVG |
| `src/components/CommitList.vue` | thêm cột Graph (độ rộng theo `--graph-width`) |
| `src/views/HistoryView.vue` | giữ `LaneLayout`, tính tiếp theo từng trang |
| `package.json` | `vitest` (devDependency), script `"test": "vitest run"` |

## Khái niệm mới (TypeScript / Vue)
- **`class` trong TypeScript**: gần giống Java: field `private`, `readonly` (như `final`), method. Khác ở chỗ phải viết `this.` trước mỗi field.
- **Non-null assertion `x!`**: `lanes[lane]!.color` nghĩa là "tôi chắc chắn giá trị này không null". TypeScript tin lời, không kiểm tra lúc chạy. Chỉ dùng khi logic ngay phía trên đảm bảo điều đó.
- **Destructuring mảng có phần còn lại** `const [firstParent, ...otherParents] = commit.parents`: phần tử đầu và mảng các phần tử còn lại.
- **Biến CSS gán từ template** `:style="{ '--graph-width': '140px' }"`, rồi CSS dùng `var(--graph-width)` trong `grid-template-columns`. Cách đưa một số tính bằng JS vào CSS mà không phải viết style cho từng dòng.
- **Đối tượng không reactive bên cạnh state reactive**: `LaneLayout` là `let` thường (Vue không theo dõi); sau mỗi lần tính, copy kết quả vào `shallowRef` để giao diện cập nhật.

## So sánh với `git log --graph`
Repo thử: `base`; branch `feature` (f1, f2); `hotfix` (h1) merge vào main; `feature` merge vào main; branch `topic` (t1) tách từ `m2`, chưa merge.

```
git log --graph                MoonGit (M = merge, * = commit, | = làn đi qua)
*   Merge branch 'feature'     M         out 0->1
|\
| | * t1                       | | *     (đầu branch, cột 2)
| |/
|/|
* | m2                         * |       in 2->0   (t1 nối vào m2)
| * f2                         | *
* |   Merge branch 'hotfix'    M |       out 0->2  (dùng lại cột 2)
|\ \
| | * f1                       | * |
| * | h1                       | | *
|/ /
* / m1                         * |       in 2->0
|/
* base                         *         in 1->0
```

Cùng cấu trúc. MoonGit giữ các làn đứng yên nên ít đường chéo hơn.

## Cách thử

App dev đã được khởi động lại. Nếu giao diện trông lạ, bấm ⌘R.

| # | Làm | Mong đợi |
|---|---|---|
| 1 | Repo `moon-git` → tab **History** | Cột Graph bên trái: một đường thẳng, mỗi commit một chấm |
| 2 | Add Repository thư mục `/private/tmp/claude-501/-Users-halab-Projects-MoonGit-moon-git/b616aaa4-6e6a-4d5b-9fde-8a50c2649836/scratchpad/branchy` (repo thử có nhiều branch, tạo sẵn), mở History | Graph giống bảng so sánh ở trên: merge có chấm rỗng, `topic` một làn riêng chưa nối lại; nhãn `main`, `feature`, `hotfix`, `topic` |
| 3 | So với `git log --graph --oneline --branches` của repo đó trong terminal | Cùng cách nối |
| 4 | Repo `big` (30.000 commit) → kéo thanh cuộn tới cuối | Vẫn mượt; đường liền mạch qua chỗ nối giữa hai trang |
| 5 | Một repo thật nhiều branch của bạn | Đường không đứt, màu ổn định theo branch |
| 6 | Chạy `pnpm test` trong terminal | 11 passed |
