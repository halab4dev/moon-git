# Bước 1 — Khung Tauri v2 + Vue 3 + TypeScript

## Trạng thái
- Frontend: `pnpm build` (type-check `vue-tsc` + Vite build) chạy thành công.
- Rust: `cargo check` và `pnpm tauri dev` chạy thành công (Rust 1.99.1), không có warning, cửa sổ app mở ra. Lần build đầu mất khoảng 4 phút rưỡi (342 crate).
- Lưu ý: nếu `xcode-select -p` vẫn in ra `/Applications/Xcode.app/...`, cần chạy `sudo xcode-select --switch /Library/Developer/CommandLineTools` một lần (xem [bước 0](00-moi-truong.md)).

## Đã làm gì
1. Sinh template chính thức bằng `create-tauri-app` (Tauri 2, template `vue-ts`, pnpm) trong thư mục tạm, rồi chép vào dự án.
2. Đổi tên app thành **MoonGit**, cửa sổ 1200×800 (nhỏ nhất 800×500) trong `tauri.conf.json`.
3. **Bỏ plugin `opener`** của template (dùng để mở link/file bằng app ngoài) vì chưa cần — tránh thêm thứ không dùng.
4. Tách code theo kiến trúc CLAUDE.md: `commands/`, `models.rs`, `src/api/`, `src/types/`, `src/components/`, `src/views/`.
5. Layout 2 cột: sidebar 260px | nội dung chính, màu tự đổi theo dark mode.
6. `git init -b main` (chưa commit).

Để dành cho bước sau: **Pinia** (bước 3, khi có state dùng chung), **`error.rs`** (bước 2, cùng lớp chạy git).

## Cấu trúc thư mục

```
moon-git/
├── package.json          # dependency frontend + script
├── pnpm-lock.yaml        # khóa phiên bản chính xác
├── index.html            # trang duy nhất, Vue gắn vào <div id="app">
├── vite.config.ts        # dev server cổng 1420 cố định (Tauri trỏ vào đó)
├── tsconfig.json         # TypeScript, "strict": true
├── docs/learning/        # các file giải thích này
├── src/                  # ── FRONTEND (chạy trong webview) ──
│   ├── main.ts           # khởi động: tạo app Vue, nạp CSS
│   ├── App.vue           # layout gốc: grid 2 cột
│   ├── styles.css        # biến màu + dark mode
│   ├── components/AppSidebar.vue   # sidebar (placeholder)
│   ├── views/WelcomeView.vue       # màn hình chính + demo gọi Rust
│   ├── api/system.ts     # hàm TS có kiểu bọc invoke()
│   └── types/system.ts   # interface khớp struct Rust
└── src-tauri/            # ── BACKEND RUST ──
    ├── Cargo.toml        # như pom.xml: tên crate, dependency (tauri, serde)
    ├── build.rs          # chạy lúc build, Tauri sinh code từ config
    ├── tauri.conf.json   # tên app, cửa sổ, lệnh build frontend, icon
    ├── capabilities/default.json  # quyền: frontend được gọi API Tauri nào
    ├── icons/
    └── src/
        ├── main.rs       # main() chỉ gọi moongit_lib::run()
        ├── lib.rs        # dựng app Tauri + đăng ký command
        ├── models.rs     # struct trả về frontend
        └── commands/
            ├── mod.rs    # khai báo module con
            └── system.rs # command `greet`
```

`main.rs` gọi sang `lib.rs` vì Tauri muốn code nằm trong thư viện để dùng lại cho bản mobile. Với mình: `lib.rs` là nơi khởi động thật.

`capabilities/default.json` hiện chỉ có `core:default`. Tauri v2 chặn mặc định — plugin nào (dialog, fs…) muốn frontend gọi được phải khai quyền ở đây. Command tự viết như `greet` thì không cần khai.

## Luồng một lời gọi: Vue → Rust → Vue

```
WelcomeView.vue          api/system.ts            [Tauri IPC]        commands/system.rs
onGreet()  ──────────▶  greet(name)  ──────▶  invoke("greet",   ──▶  fn greet(name: String)
                                               {name: "Ha"})          -> Greeting
reply.value = ... ◀──── Promise<Greeting> ◀──  JSON ◀───────────────  serde serialize
```

1. Gõ tên, bấm nút → `@submit.prevent="onGreet"` gọi `onGreet()` (`.prevent` chặn form reload trang).
2. `greet(name.value)` trong `api/system.ts` gọi `invoke("greet", { name })`. Tauri đóng gói JSON, gửi qua cầu nối webview sang tiến trình Rust.
3. `generate_handler![...]` trong `lib.rs` đã sinh sẵn lúc compile bảng tra "tên command → hàm Rust". Tauri chuyển `{"name": "Ha"}` thành `name: String` và gọi hàm.
4. Hàm trả `Greeting`; `serde` chuyển thành `{"message": "...", "platform": "macos"}`.
5. Bên TS, Promise resolve với object kiểu `Greeting`; Vue tự cập nhật giao diện.

**Lưu ý:** key trong `{ name }` phải trùng tên tham số Rust (Tauri tự đổi snake_case ↔ camelCase cho tham số). Sai tên thì lỗi lúc chạy, không phải lúc compile — vì vậy gom mọi `invoke` vào `src/api/` để chỉ sai ở một chỗ.

Tương tự, `interface Greeting` trong `src/types/system.ts` phải được sửa tay cho khớp `models.rs` mỗi khi đổi struct.

## Code chính

`src-tauri/src/models.rs`
```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Greeting {
    pub message: String,
    pub platform: String,
}
```

`src-tauri/src/commands/system.rs`
```rust
#[tauri::command]
pub fn greet(name: String) -> Greeting {
    let trimmed = name.trim();
    let who = if trimmed.is_empty() { "stranger" } else { trimmed };

    Greeting {
        message: format!("Hello, {who}! This message comes from Rust."),
        platform: std::env::consts::OS.to_string(),
    }
}
```

`src/api/system.ts`
```ts
export function greet(name: string): Promise<Greeting> {
  return invoke<Greeting>("greet", { name });
}
```

## Khái niệm Rust lần đầu gặp

- **`struct` + `pub`**: như class Java chỉ có field, không method. Mặc định private trong module; `pub` ≈ `public`.
- **`#[derive(Serialize)]`**: compiler tự sinh code chuyển struct → JSON. Giống Jackson nhưng sinh lúc compile, không reflection lúc chạy. `#[serde(rename_all = "camelCase")]` ≈ `@JsonNaming`. `Debug` cho phép in struct ra để debug (≈ `toString()` tự sinh).
- **`#[tauri::command]`**: attribute, trông như annotation Java; sinh code để hàm gọi được từ JS.
- **`mod` / `use crate::...`**: `mod commands;` khai báo module `commands` (thư mục `commands/`, nội dung chính ở `mod.rs`). `use` ≈ `import`; `crate::` là gốc project.
- **`String` vs `&str` (ownership):**
  - `String`: biến **sở hữu** chuỗi. `&str`: **mượn** tạm chuỗi của chỗ khác.
  - Java: biến String luôn là tham chiếu, GC tự dọn. Rust không có GC: mỗi giá trị có đúng một chủ sở hữu, giải phóng khi chủ sở hữu ra khỏi scope.
  - Template gốc dùng `name: &str`; đã đổi sang `String` cho dễ hiểu, tránh phải nghĩ về lifetime.
  - `name.trim()` trả `&str` mượn từ `name` — dùng được khi `name` còn sống.
- **Không cần `return`**: biểu thức cuối (không có `;`) là giá trị trả về. `if … { a } else { b }` cũng là biểu thức, dùng như `? :` của Java.
- **`format!("Hello, {who}!")`**: dấu `!` nghĩa là **macro** (sinh code lúc compile). `format!` ≈ `String.format`, có thể nhét tên biến thẳng vào `{}`.

## Khái niệm Vue lần đầu gặp

- **`<script setup lang="ts">`**: biến/hàm khai báo ở đây dùng thẳng trong `<template>`.
- **`ref("")`**: giá trị reactive. Trong script đọc/ghi qua `.value`; trong template dùng tên trực tiếp. Đổi `.value` → Vue tự vẽ lại phần liên quan. Giống observable/listener nhưng không phải tự đăng ký.
- **`v-model="name"`**: ràng buộc hai chiều ô input ↔ `name`.
- **`v-if`**: chỉ render khi điều kiện đúng.
- **`@submit.prevent`**: `@` = lắng nghe sự kiện (≈ `addEventListener`), `.prevent` = gọi `preventDefault()`.
- **Component**: một file `.vue` = template + script + style. `<style scoped>` chỉ áp dụng cho component đó. Dùng bằng cách import rồi viết `<AppSidebar />`.

## Cách chạy và kiểm tra

```bash
cd ~/Projects/MoonGit/moon-git
pnpm install        # nếu chưa cài dependency
pnpm tauri dev
```
Lần đầu mất vài phút (cargo tải và compile vài trăm crate); các lần sau nhanh hơn nhiều.

Kiểm tra:
- Cửa sổ tiêu đề **MoonGit**, sidebar trái: "Repositories / No repositories yet."
- Gõ tên, bấm **Greet from Rust** (hoặc Enter) → hiện "Hello, … This message comes from Rust. (platform: macos)".
- Đổi Dark/Light của macOS → màu giao diện đổi theo.
- Sửa chữ trong `WelcomeView.vue` → cửa sổ cập nhật ngay (hot reload). Sửa file `.rs` → Tauri tự compile lại và khởi động lại app.

Lệnh hữu ích khác:
- `pnpm build` — chỉ type-check + build frontend.
- `cd src-tauri && cargo check` — chỉ kiểm tra Rust compile được (nhanh hơn build).
