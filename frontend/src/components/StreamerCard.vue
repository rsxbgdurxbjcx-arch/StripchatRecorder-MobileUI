<!--
    主播卡片组件 / Streamer Card Component

    展示单个主播的缩略图、在线状态、观看人数和录制控制按钮。
    通过 useFastThumbnail 对多个 CDN 域名进行竞速，加快缩略图加载速度。
    状态徽章与观看人数以浮层形式叠加在缩略图上，卡片内容区仅保留两行，
    达到最紧凑状态，无冗余空白。

    Displays a single streamer's thumbnail, online status, viewer count, and recording controls.
    Uses useFastThumbnail to race multiple CDN domains for faster thumbnail loading.
    Status badges and viewer count are overlaid on the thumbnail; the content area keeps
    only two rows for maximum compactness with no wasted space.

    Props:
        streamer - 主播数据对象 / Streamer data object

    Emits:
        remove       - 用户点击移除按钮 / User clicks remove button
        start        - 用户点击开始录制 / User clicks start recording
        stop         - 用户点击停止录制 / User clicks stop recording
        toggle-auto  - 用户切换自动录制开关 / User toggles auto-record switch
-->
<script setup lang="ts">
	import type { StreamerEntry } from "../stores/streamers";
	import { Card, CardContent } from "@/components/ui/card";
	import { Badge } from "@/components/ui/badge";
	import { Button } from "@/components/ui/button";
	import { Switch } from "@/components/ui/switch";
	import { ref, watch, computed } from "vue";
	import { useFastThumbnail } from "@/composables/useFastThumbnail";
	import { copyToClipboard } from "@/lib/utils";
	import { X, Circle, Eye, Copy, Check } from "@lucide/vue";
	import { useI18n } from "vue-i18n";

	const props = defineProps<{ streamer: StreamerEntry }>();
	void props;
	const emit = defineEmits<{
		remove: [];
		start: [];
		stop: [];
		"toggle-auto": [enabled: boolean];
	}>();
	const { t } = useI18n();

	const autoRecord = ref(props.streamer.auto_record);
	watch(
		() => props.streamer.auto_record,
		(val) => { autoRecord.value = val; },
	);

	const thumbnailSrc = computed(() => props.streamer.thumbnail_url ?? null);
	const fastThumbnail = useFastThumbnail(thumbnailSrc);

	const copied = ref(false);

	// 转发流地址 / Stream relay URL
	const streamUrl = computed(
		() => `${window.location.origin}/stream/${props.streamer.username}`,
	);

	async function copyStreamUrl() {
		try {
			await copyToClipboard(streamUrl.value);
			copied.value = true;
			setTimeout(() => { copied.value = false; }, 2000);
		} catch {}
	}

	function onAutoChange(val: boolean) {
		autoRecord.value = val;
		emit("toggle-auto", val);
	}

	function statusClass(s: StreamerEntry): string {
		if (!s.is_online) return "bg-zinc-800/85 text-zinc-300 border-transparent";
		if (s.status === "公开秀") return "bg-green-900/85 text-green-300 border-transparent";
		return "bg-amber-900/85 text-amber-300 border-transparent";
	}
</script>

<template>
	<Card class="streamer-card overflow-hidden transition-all py-0 gap-0"
		:class="{ 'border-green-900/50': streamer.is_online && !streamer.is_recording, 'border-red-900/50': streamer.is_recording }">
		<div class="relative aspect-video bg-muted overflow-hidden">
			<img v-if="fastThumbnail" :src="fastThumbnail" loading="lazy" class="w-full h-full object-cover" />
			<div v-else class="w-full h-full flex items-center justify-center text-2xl font-bold text-muted-foreground/20">
				{{ streamer.username[0].toUpperCase() }}
			</div>
			<!-- 状态徽章浮层（左上）/ Status badge overlay (top-left) -->
			<div class="absolute top-1 left-1 flex items-center gap-1">
				<Badge :class="statusClass(streamer)" class="text-[9px] px-1 py-0 leading-tight backdrop-blur-[2px]">
					{{ streamer.is_online ? streamer.status : t("streamerCard.offline") }}
				</Badge>
				<Badge v-if="streamer.is_recording" variant="destructive" class="text-[9px] px-1 py-0 leading-tight">{{ t("streamerCard.recording") }}</Badge>
			</div>
			<Circle v-if="streamer.is_recording" class="absolute top-1 right-1 size-1.5 fill-red-500 text-red-500 animate-pulse" />
			<!-- 观看人数浮层（左下）/ Viewer count overlay (bottom-left) -->
			<span v-if="streamer.is_online && streamer.viewers > 0"
				class="absolute bottom-1 left-1 text-[9px] text-white bg-black/55 backdrop-blur-[2px] rounded px-1 py-px flex items-center gap-0.5">
				<Eye class="size-2" /> {{ streamer.viewers.toLocaleString() }}
			</span>
		</div>
		<CardContent class="p-1.5 pt-1 flex flex-col gap-1">
			<div class="flex items-center justify-between gap-1">
				<span class="font-semibold text-[11px] truncate leading-tight">{{ streamer.username }}</span>
				<Button variant="ghost" size="icon" class="h-4 w-4 shrink-0 text-muted-foreground hover:text-destructive"
					:title="t('streamerCard.removeTitle')" @click="emit('remove')">
					<X class="size-2.5" />
				</Button>
			</div>
			<div class="flex items-center gap-1">
				<Button v-if="!streamer.is_recording" size="sm" class="flex-1 h-6 text-[10px] px-1"
					:disabled="!streamer.is_recordable" :title="!streamer.is_recordable ? streamer.status : ''" @click="emit('start')">
					{{ t("streamerCard.startRecording") }}
				</Button>
				<Button v-else size="sm" variant="destructive" class="flex-1 h-6 text-[10px] px-1" @click="emit('stop')">
					{{ t("streamerCard.stopRecording") }}
				</Button>
				<Switch :id="`auto-${streamer.username}`" :model-value="autoRecord" @update:model-value="onAutoChange" class="scale-90" />
				<Button size="icon" variant="ghost" class="shrink-0 h-6 w-6 text-muted-foreground hover:text-primary"
					:title="`${t('streamerCard.copyStreamUrl')}\n${streamUrl}`" @click="copyStreamUrl">
					<Check v-if="copied" class="size-2.5 text-green-400" />
					<Copy v-else class="size-2.5" />
				</Button>
			</div>
		</CardContent>
	</Card>
</template>
