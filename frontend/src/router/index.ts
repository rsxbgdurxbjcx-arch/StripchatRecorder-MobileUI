/**
 * 路由配置 / Router Configuration
 */

import { createRouter, createWebHistory } from "vue-router";
import { call, isLoggedIn, checkAuth } from "@/lib/api";
import type { Settings } from "@/stores/settings";

const router = createRouter({
        history: createWebHistory(),
        routes: [
                { path: "/login", component: () => import("../views/LoginView.vue") },
                { path: "/setup", component: () => import("../views/SetupView.vue") },
                { path: "/", component: () => import("../views/HomeView.vue") },
                { path: "/recordings", component: () => import("../views/RecordingsView.vue") },
                { path: "/postprocess", component: () => import("../views/PostprocessView.vue") },
                { path: "/settings", component: () => import("../views/SettingsView.vue") },
                { path: "/finder", component: () => import("../views/FinderView.vue") },
                { path: "/relay", component: () => import("../views/RelayView.vue") },
        ],
});

let setupChecked = false;
let authVerified = false;

router.beforeEach(async (to) => {
        if (to.path === "/login") {
                if (authVerified && isLoggedIn()) {
                        return "/";
                }
                return true;
        }

        if (to.path === "/setup") {
                return true;
        }

        if (!setupChecked) {
                try {
                        const settings = await call<Settings>("get_settings");
                        setupChecked = true;
                        if (!settings.setup_done) {
                                return "/setup";
                        }
                } catch {
                        setupChecked = true;
                }
        }

        if (!authVerified) {
                if (!isLoggedIn()) {
                        return "/login";
                }
                const valid = await checkAuth();
                authVerified = true;
                if (!valid) {
                        return "/login";
                }
        }

        return true;
});

export default router;
