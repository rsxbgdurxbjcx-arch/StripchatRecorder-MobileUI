<!--
    登录页面 / Login View
-->

<script setup lang="ts">
        import { ref } from "vue";
        import { useRouter } from "vue-router";
        import { login } from "@/lib/api";
        import { useI18n } from "vue-i18n";
        import { Button } from "@/components/ui/button";
        import { Input } from "@/components/ui/input";
        import { Label } from "@/components/ui/label";
        import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

        const router = useRouter();
        const { t } = useI18n();

        const username = ref("");
        const password = ref("");
        const loading = ref(false);
        const error = ref("");

        async function handleLogin() {
                if (!username.value || !password.value) {
                        error.value = t("login.required");
                        return;
                }
                loading.value = true;
                error.value = "";
                try {
                        const ok = await login(username.value, password.value);
                        if (ok) {
                                router.replace("/");
                        } else {
                                error.value = t("login.failed");
                        }
                } catch (e) {
                        error.value = String(e);
                } finally {
                        loading.value = false;
                }
        }
</script>

<template>
        <div class="min-h-dvh flex items-center justify-center p-4 login-bg">
                <Card class="w-full max-w-sm rounded-2xl shadow-2xl">
                        <CardHeader class="items-center pt-8">
                                <div class="flex items-center gap-2.5 mb-1">
                                        <span class="brand-dot w-3 h-3 rounded-full" />
                                        <span class="text-base font-bold tracking-tight">StripchatRecorder</span>
                                </div>
                                <CardTitle class="text-center text-sm font-medium text-muted-foreground">{{ t("login.title") }}</CardTitle>
                        </CardHeader>
                        <CardContent class="pb-8">
                                <form class="space-y-4" @submit.prevent="handleLogin">
                                        <div class="space-y-2">
                                                <Label for="username">{{ t("login.username") }}</Label>
                                                <Input
                                                        id="username"
                                                        v-model="username"
                                                        type="text"
                                                        :placeholder="t('login.usernamePlaceholder')"
                                                        autocomplete="username"
                                                />
                                        </div>
                                        <div class="space-y-2">
                                                <Label for="password">{{ t("login.password") }}</Label>
                                                <Input
                                                        id="password"
                                                        v-model="password"
                                                        type="password"
                                                        :placeholder="t('login.passwordPlaceholder')"
                                                        autocomplete="current-password"
                                                />
                                        </div>
                                        <p v-if="error" class="text-sm text-destructive">
                                                {{ error }}
                                        </p>
                                        <Button
                                                type="submit"
                                                class="w-full"
                                                :disabled="loading"
                                        >
                                                {{ loading ? t("login.loading") : t("login.submit") }}
                                        </Button>
                                </form>
                        </CardContent>
                </Card>
        </div>
</template>
