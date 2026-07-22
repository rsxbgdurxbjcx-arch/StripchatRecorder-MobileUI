<!--
    应用根组件 / Application Root Component

    提供侧边栏导航和主内容区域的整体布局。
    负责：
    - 跟随系统主题自动切换深色/浅色模式
    - 监听 ffmpeg-missing 事件并显示警告
    - 监听 SSE 断开/重连事件，重连后自动刷新页面
    - 监听 startup-warnings 事件，处理不存在的主播和孤立的后处理记录

    Provides the overall layout with sidebar navigation and main content area.
    Responsible for:
    - Auto dark/light mode following system theme
    - Listening for ffmpeg-missing events and showing warnings
    - Listening for SSE disconnect/reconnect events, auto-reloading on reconnect
    - Listening for startup-warnings to handle non-existent streamers and orphaned post-processing records
-->
<script setup lang="ts">
	import { onMounted, onUnmounted, ref } from "vue";
	import { RouterView, useRouter, useRoute } from "vue-router";
	import NotifyLayer from "./components/NotifyLayer.vue";
	import { Button } from "@/components/ui/button";
	import { call, on, onSseReconnect, onSseDisconnect } from "@/lib/api";
	import { useNotify } from "@/composables/useNotify";
	import { toast as sonnerToast } from "vue-sonner";
	import { useStreamersStore } from "@/stores/streamers";
	import { useI18n } from "vue-i18n";
	import { Users, Film, Workflow, Radio, Search, Settings } from "@lucide/vue";
	import { useScrollbar } from "@/composables/useScrollbar";
	import { loadLocaleFromServer } from "@/i18n";
	import { useModuleLocaleStore } from "@/stores/moduleLocale";
	import { useLocalesStore } from "@/stores/locales";
	import { usePostprocessStore } from "@/stores/postprocess";

	const router = useRouter();
	const route = useRoute();
	const { toast, confirm } = useNotify();
	const streamersStore = useStreamersStore();
	const { t, locale } = useI18n();
	const moduleLocaleStore = useModuleLocaleStore();
	const localesStore = useLocalesStore();
	const postprocessStore = usePostprocessStore();

	/** 无感刷新计数器：递增会重挂载主布局当前视图 / Silent refresh counter: incrementing remounts the current main-layout view */
	const refreshKey = ref(0);

	const mainScrollEl = ref<HTMLElement | null>(null);
	useScrollbar(mainScrollEl);

	/** 侧边栏导航项配置 / Sidebar navigation items configuration */
	const navItems = [
		{ to: "/", labelKey: "nav.streamers", icon: Users },
		{ to: "/recordings", labelKey: "nav.recordings", icon: Film },
		{ to: "/postprocess", labelKey: "nav.postprocess", icon: Workflow },
		{ to: "/relay", labelKey: "nav.relay", icon: Radio },
		{ to: "/finder", labelKey: "nav.finder", icon: Search },
		{ to: "/settings", labelKey: "nav.settings", icon: Settings },
	];

	/**
	 * 根据参数切换文档根元素的 dark 类，实现深色/浅色主题切换。
	 * Toggle the dark class on the document root element for dark/light theme switching.
	 *
	 * @param dark - 是否应用深色主题 / Whether to apply dark theme
	 */
	function applyTheme(dark: boolean) {
		document.documentElement.classList.toggle("dark", dark);
	}

	// 监听系统主题变化 / Listen for system theme changes
	const mq = window.matchMedia("(prefers-color-scheme: dark)");
	function onThemeChange(e: MediaQueryListEvent) {
		applyTheme(e.matches);
	}

	/**
	 * 将应用外壳高度实时同步为 visualViewport 高度（--app-height）。
	 * 安卓 Chrome/Firefox 的动态工具栏、键盘弹起都会改变可视视口；
	 * dvh 在部分 Firefox 版本更新不及时会导致底部菜单"松动"，
	 * visualViewport 事件在两个浏览器都能精确跟踪。
	 *
	 * Sync the app shell height to the visualViewport height in real time (--app-height).
	 * Dynamic toolbars and the virtual keyboard on Android Chrome/Firefox change the
	 * visual viewport; dvh updates lag on some Firefox versions, leaving the bottom
	 * nav "loose". visualViewport events track precisely on both browsers.
	 */
	function syncAppHeight() {
		const h = window.visualViewport ? window.visualViewport.height : window.innerHeight;
		document.documentElement.style.setProperty("--app-height", `${h}px`);
	}

	// 事件取消订阅函数 / Event unsubscribe functions
	let unlistenFfmpeg: (() => void) | null = null;
	let unlistenReconnect: (() => void) | null = null;
	let unlistenDisconnect: (() => void) | null = null;
	let unlistenWarnings: (() => void) | null = null;
	let unlistenLocaleWarnings: (() => void) | null = null;

	/**
	 * 处理启动时的警告事件：
	 * 1. 不存在的主播账号 -> 提示用户并自动删除
	 * 2. 孤立的后处理记录（对应文件已删除）-> 提示用户并清理
	 *
	 * Handle startup warning events:
	 * 1. Non-existent streamer accounts -> prompt user and auto-delete
	 * 2. Orphaned post-processing records (files deleted) -> prompt user and clean up
	 */
	async function handleStartupWarnings(payload: unknown) {
		const w = payload as {
			missing_streamers: string[];
			missing_pp_results: string[];
		};

		if (w.missing_streamers.length > 0) {
			const ok = await confirm({
				title: t("notify.missingStreamers.title"),
				message: t("notify.missingStreamers.message", { list: w.missing_streamers.join("\n") }),
				confirmText: t("notify.missingStreamers.confirm"),
				cancelText: t("notify.missingStreamers.ignore"),
				danger: true,
			});
			if (ok) {
				for (const username of w.missing_streamers) {
					await streamersStore.removeStreamer(username).catch(() => {});
				}
				toast(t("notify.missingStreamers.done", { count: w.missing_streamers.length }), "success");
			}
		}

		if (w.missing_pp_results.length > 0) {
			const ok = await confirm({
				title: t("notify.missingPpResults.title"),
				message: t("notify.missingPpResults.message", { list: w.missing_pp_results.map((p) => p.split(/[\\/]/).pop()).join("\n") }),
				confirmText: t("notify.missingPpResults.confirm"),
				cancelText: t("notify.missingPpResults.ignore"),
			});
			if (ok) {
				await call("remove_missing_pp_results", {
					paths: w.missing_pp_results,
				}).catch(() => {});
				toast(t("notify.missingPpResults.done", { count: w.missing_pp_results.length }), "success");
			}
		}
	}

	onMounted(async () => {
		// 初始化主题并监听系统主题变化 / Initialize theme and listen for system theme changes
		applyTheme(mq.matches);
		mq.addEventListener("change", onThemeChange);

		// 同步应用外壳高度到可视视口，并跟踪其变化（动态工具栏/键盘）
		// Sync app shell height to the visual viewport and track changes
		// (dynamic toolbar / virtual keyboard)
		syncAppHeight();
		window.addEventListener("resize", syncAppHeight);
		window.visualViewport?.addEventListener("resize", syncAppHeight);

		// 从后端同步语言设置，先加载消息再切换 locale
		// Sync language from backend, load messages before switching locale
		try {
			const settings = await call<{ language?: string }>("get_settings");
			if (settings?.language) {
				// 先加载该语言的消息，再切换 locale，保证首屏就用正确语言渲染
				// Load messages for the language first, then switch locale,
				// so the first render already uses the correct language
				const { modules: moduleLocales } = await loadLocaleFromServer(settings.language);
				locale.value = settings.language;
				moduleLocaleStore.setLocales(settings.language, moduleLocales);
			} else {
				// 无自定义语言，仍加载默认 locale 的服务器覆盖（模块翻译等）
				// No custom language, still load server overrides for the default locale
				const { modules: moduleLocales } = await loadLocaleFromServer(locale.value);
				moduleLocaleStore.setLocales(locale.value, moduleLocales);
			}
		} catch {
			// 后端未就绪时加载当前 locale 的消息作为 fallback
			// Backend not ready: load current locale messages as fallback
			const { modules: moduleLocales } = await loadLocaleFromServer(locale.value);
			moduleLocaleStore.setLocales(locale.value, moduleLocales);
		}

		// 监听 ffmpeg 缺失警告 / Listen for ffmpeg missing warning
		unlistenFfmpeg = await on("ffmpeg-missing", (payload) => {
			const p = payload as { message: string };
			toast(p.message, "warning");
		});

		// SSE 重连后无感刷新：递增 refreshKey 重挂载当前视图（各视图 onMounted 重新拉取数据），
		// 并主动刷新全局 store，替代旧的倒计时整页 reload。
		// Silent refresh after SSE reconnect: bump refreshKey to remount the current view
		// (each view re-fetches its data in onMounted) and proactively refresh global stores,
		// replacing the old countdown + full page reload.
		unlistenReconnect = onSseReconnect(() => {
			refreshKey.value++;
			void streamersStore.fetchStreamers();
			void postprocessStore.fetchPipeline();
			void localesStore.refresh();
			sonnerToast.info(t("notify.reconnectedSilent"), { duration: 2000 });
		});

		// 监听 SSE 断开连接 / Listen for SSE disconnect
		unlistenDisconnect = onSseDisconnect(() => {
			toast(t("notify.disconnected"), "warning");
		});

		// 监听启动警告 / Listen for startup warnings
		unlistenWarnings = await on("startup-warnings", handleStartupWarnings);

		// 监听自定义语言文件校验警告 / Listen for custom locale file validation warnings
		unlistenLocaleWarnings = await on(
			"locale-warnings",
			(payload) => {
				const items = payload as Array<{ path: string; reason: string }>;
				for (const item of items) {
					const file = item.path.replace(/\\/g, "/").split("/").pop() ?? item.path;
					toast(`${t("settings.localeFileInvalid", { file })}: ${item.reason}`, "warning");
				}
			},
		);

		// 初始加载可用语言列表 / Initial load of available locales
		await localesStore.refresh();

		// locale-files-changed 事件已在 localesStore 内部监听，无需在此重复注册
		// locale-files-changed is already listened inside localesStore; no need to register here
	});

	onUnmounted(() => {
		// 清理所有事件监听器 / Clean up all event listeners
		mq.removeEventListener("change", onThemeChange);
		window.removeEventListener("resize", syncAppHeight);
		window.visualViewport?.removeEventListener("resize", syncAppHeight);
		unlistenFfmpeg?.();
		unlistenReconnect?.();
		unlistenDisconnect?.();
		unlistenWarnings?.();
		unlistenLocaleWarnings?.();
	});
</script>

<template>
	<Transition name="layout" mode="out-in">
		<div v-if="route.path === '/setup' || route.path === '/login'" key="setup" class="contents">
			<RouterView v-slot="{ Component }">
				<Transition name="page" mode="out-in">
					<component :is="Component" :key="route.path" />
				</Transition>
			</RouterView>
			<NotifyLayer />
		</div>

		<!-- 外壳 fixed 钉在视口顶部，高度实时取 visualViewport（--app-height，回退 100dvh）：
		     Chrome/Firefox 安卓版动态工具栏伸缩、键盘弹起时外壳高度始终等于可视高度，底部菜单永不松动移位 -->
		<!-- Shell fixed to viewport top, height synced to visualViewport in real time
		     (--app-height, fallback 100dvh): on Chrome/Firefox Android the shell always
		     matches the visible height, so the bottom nav never comes loose -->
		<div v-else key="main" class="fixed inset-x-0 top-0 flex flex-col overflow-hidden md:flex-row app-shell">
			<aside class="hidden md:flex w-52 shrink-0 bg-sidebar/85 backdrop-blur-xl border-r border-sidebar-border flex-col p-3 gap-1">
				<div class="flex items-center gap-2.5 px-2 pt-4 pb-4 mb-2 border-b border-sidebar-border">
					<span class="brand-dot w-2.5 h-2.5 rounded-full shrink-0" />
					<div class="flex flex-col leading-none">
						<span class="text-sm font-bold tracking-tight text-sidebar-foreground">StripchatRecorder</span>
						<span class="text-[10px] text-sidebar-foreground/45 mt-1 tracking-widest uppercase">Mobile UI</span>
					</div>
				</div>
				<nav class="flex flex-col gap-1">
					<Button v-for="item in navItems" :key="item.to" variant="ghost"
						class="w-full justify-start text-sm font-normal rounded-lg transition-all duration-150 hover:translate-x-0.5"
						:class="route.path === item.to ? 'bg-primary/12 text-primary font-semibold shadow-[inset_0_1px_0_oklch(1_0_0/6%)]' : 'text-sidebar-foreground/65 hover:text-sidebar-foreground hover:bg-sidebar-accent/60'"
						@click="router.push(item.to)">
						<component :is="item.icon" class="size-4 mr-2" />
						{{ t(item.labelKey) }}
						<span v-if="route.path === item.to" class="ml-auto size-1.5 rounded-full bg-primary shadow-[0_0_6px_oklch(0.62_0.19_16/80%)]" />
					</Button>
				</nav>
			</aside>
			<main class="flex-1 overflow-hidden">
				<div ref="mainScrollEl" class="h-full overflow-y-scroll p-3 md:p-6 scrollbar-overlay pb-3 md:pb-6">
					<RouterView v-slot="{ Component }">
						<Transition name="page" mode="out-in">
							<component :is="Component" :key="route.path + ':' + refreshKey" />
						</Transition>
					</RouterView>
				</div>
			</main>

			<nav class="md:hidden shrink-0 bg-sidebar/85 backdrop-blur-xl border-t border-sidebar-border flex justify-around items-stretch h-[calc(4rem+env(safe-area-inset-bottom))] pb-[env(safe-area-inset-bottom)]">
				<button v-for="item in navItems" :key="item.to"
					class="relative flex flex-col items-center justify-center gap-0.5 py-1.5 px-1 flex-1 transition-colors"
					:class="route.path === item.to ? 'text-primary font-semibold' : 'text-sidebar-foreground/55'"
					@click="router.push(item.to)">
					<span class="absolute top-0 h-0.5 w-8 rounded-full bg-primary transition-opacity duration-200"
						:class="route.path === item.to ? 'opacity-100' : 'opacity-0'" />
					<component :is="item.icon" class="size-5" />
					<span class="text-[10px]">{{ t(item.labelKey) }}</span>
				</button>
			</nav>

			<NotifyLayer />
		</div>
	</Transition>
</template>
