import { createRouter, createWebHashHistory } from "vue-router";

const BrowserList = () => import("../pages/BrowserList.vue");
const Proxies = () => import("../pages/Proxies.vue");
const Engines = () => import("../pages/Engines.vue");
const Settings = () => import("../pages/Settings.vue");

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/browsers" },
    { path: "/browsers", component: BrowserList },
    { path: "/proxies", component: Proxies },
    { path: "/engines", component: Engines },
    { path: "/settings", component: Settings },
  ],
});

export default router;
