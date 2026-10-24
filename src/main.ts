import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import DiffWindow from "./views/DiffWindow.vue";
import ConflictWindow from "./views/ConflictWindow.vue";
import "./styles.css";

// Every window loads this same page; "?view=..." picks its screen
// (see windows.ts). No view = the main window.
const view = new URLSearchParams(window.location.search).get("view");
const root = view === "diff" ? DiffWindow : view === "conflict" ? ConflictWindow : App;

createApp(root).use(createPinia()).mount("#app");
