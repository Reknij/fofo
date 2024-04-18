<script setup lang="ts">
import { NSpace, NTag, NCard, NButton, useLoadingBar } from 'naive-ui';
import type { SubPath } from '~/components/FofoBreadcrumb/model';
import { useCurrentUser } from '~/states/auth';

const loadingBar = useLoadingBar()
loadingBar.start()
const user = useCurrentUser();
const router = useRouter();
const subpaths: SubPath[] = [
    {
        label: 'Notifications'
    }
]

async function goLogin() {
    await router.push('/login')
}

onMounted(()=>loadingBar.finish())
</script>

<template>
    <div>
        <n-space vertical v-if="user">
            <FofoBreadcrumb :subpath="subpaths"></FofoBreadcrumb>
            <n-tag :bordered="false">User notifications</n-tag>
            <n-card size="small">
                <UserNotifications />
            </n-card>
        </n-space>
        <n-space vertical v-else>
            <n-card size="small">
            <n-space vertical>
                Please login first.
                <n-button @click="goLogin">Login now</n-button>
            </n-space>
        </n-card>
        </n-space>
    </div>
</template>