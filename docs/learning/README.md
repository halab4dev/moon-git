# Nhật ký học — MoonGit

Mỗi bước làm xong có một file giải thích riêng, đọc theo thứ tự. Tiến độ tổng: [../PROGRESS.md](../PROGRESS.md).

| Bước | File | Nội dung |
|---|---|---|
| 0 | [00-moi-truong.md](00-moi-truong.md) | Chuẩn bị môi trường (Xcode CLT, Rust, Node, git) và lộ trình Giai đoạn 1 |
| 1 | [01-khung-du-an.md](01-khung-du-an.md) | Khung Tauri v2 + Vue 3 + TS, layout sidebar \| nội dung, command `greet` |
| 2 | [02-lop-chay-git.md](02-lop-chay-git.md) | `run_git`, `AppError`, command `check_repo`, unit test với repo tạm |
| 3 | [03-danh-sach-repo.md](03-danh-sach-repo.md) | `RepoStore` lưu `repos.json`, plugin dialog, Pinia store, sidebar danh sách repo |

## Mục lục khái niệm (lần đầu xuất hiện ở bước nào)

**Rust:** `struct`/`pub` (1) · `#[derive]`, attribute (1) · `mod`/`use` (1) · `String` vs `&str`, ownership (1) · biểu thức trả về không cần `return` (1) · macro `format!` (1) · `Result` thay exception (2) · toán tử `?`, `map_err` (2) · `enum` + `match`, match guard (2) · `Option` thay null (2) · `impl`, hàm `new` (2) · `Clone`/`Copy`/`PartialEq` (2) · `&Path` (2) · unit test `#[cfg(test)]`, `unwrap` (2) · dev-dependency (2) · `State` + `app.manage` (3) · lifetime `'_` (3) · `Mutex`, `MutexGuard` (3) · closure, iterator (3) · `if let` (3) · `Clone` (3) · `Deserialize` (3) · `setup` (3)

**Vue / TS:** `<script setup>` (1) · `ref` (1) · `v-model`, `v-if`, `@submit.prevent` (1) · component, `<style scoped>` (1) · `ref<T>` có kiểu, union type (2) · `try/catch/finally` với `await` (2) · `:attr` / `v-bind` (2) · Pinia store (3) · `computed`, `??` (3) · props `defineProps` (3) · `onMounted` (3) · `v-for` + `:key` (3) · `:class`, `@click.stop` (3)
