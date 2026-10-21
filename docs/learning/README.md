# Nhật ký học — MoonGit

Mỗi bước làm xong có một file giải thích riêng, đọc theo thứ tự. Tiến độ tổng: [../PROGRESS.md](../PROGRESS.md).

| Bước | File | Nội dung |
|---|---|---|
| 0 | [00-moi-truong.md](00-moi-truong.md) | Chuẩn bị môi trường (Xcode CLT, Rust, Node, git) và lộ trình Giai đoạn 1 |
| 1 | [01-khung-du-an.md](01-khung-du-an.md) | Khung Tauri v2 + Vue 3 + TS, layout sidebar \| nội dung, command `greet` |
| 2 | [02-lop-chay-git.md](02-lop-chay-git.md) | `run_git`, `AppError`, command `check_repo`, unit test với repo tạm |

## Mục lục khái niệm (lần đầu xuất hiện ở bước nào)

**Rust:** `struct`/`pub` (1) · `#[derive]`, attribute (1) · `mod`/`use` (1) · `String` vs `&str`, ownership (1) · biểu thức trả về không cần `return` (1) · macro `format!` (1) · `Result` thay exception (2) · toán tử `?`, `map_err` (2) · `enum` + `match`, match guard (2) · `Option` thay null (2) · `impl`, hàm `new` (2) · `Clone`/`Copy`/`PartialEq` (2) · `&Path` (2) · unit test `#[cfg(test)]`, `unwrap` (2) · dev-dependency (2)

**Vue / TS:** `<script setup>` (1) · `ref` (1) · `v-model`, `v-if`, `@submit.prevent` (1) · component, `<style scoped>` (1) · `ref<T>` có kiểu, union type (2) · `try/catch/finally` với `await` (2) · `:attr` / `v-bind` (2)
