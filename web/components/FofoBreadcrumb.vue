<script setup lang="ts">
import { NCard, NBreadcrumb, NBreadcrumbItem, NIcon, NScrollbar } from 'naive-ui';
import type { SubPath } from './FofoBreadcrumb/model';
import { HomeOutlined } from '@vicons/antd';

const router = await useRouter();
const props = defineProps<{
    subpath: SubPath[],
}>();

async function go(e: Event, path?: string) {
    e.preventDefault();
    if (path) await router.push(path)
}
</script>

<template>
    <n-card size="small">
        <n-scrollbar x-scrollable>
            <n-breadcrumb>
                <n-breadcrumb-item href="/" @click="(e: Event) => go(e, '/')">
                    <n-icon :component="HomeOutlined"/>
                </n-breadcrumb-item>
                <n-breadcrumb-item v-for="p in subpath" @click="(e: Event) => go(e, p.href)" :href="p.href">{{ p.label }}</n-breadcrumb-item>
            </n-breadcrumb>
        </n-scrollbar>
    </n-card>
</template>

<style>
.n-breadcrumb ul {
    display: inline-flex;
}
</style>