<script setup lang="ts">
import 'bytemd/dist/index.css'
import '~/styles/bytemd.css'
import '~/github-markdown-style.css'

// @ts-ignore
import { Viewer } from "@bytemd/vue-next"; // dont remove it!
import { bytemdPlugins } from '~/helper';

const props = defineProps<{
    content?: string,
}>();
const container = ref<HTMLDivElement>()
const isClamped = computed(() => {
    if (container.value) return container.value.scrollHeight > container.value.clientHeight;
    else return false;
})

const showMore = ref(false);
</script>

<template>
    <div class="flex flex-col">
        <div ref="container" :class="showMore ? 'showContent' : `hideContent`">
            <Viewer :value="content" :plugins="bytemdPlugins"></Viewer>
        </div>
        <UDivider v-if="isClamped">
            <UButton variant="link" @click="showMore = !showMore">{{ showMore ? 'Hide' : 'Expand' }}</UButton>
        </UDivider>
    </div>
</template>

<style>
/* @import url("https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.5.1/github-markdown.min.css"); */

.showContent {
    @apply max-w-full;
}

.hideContent {
    @apply line-clamp-[15] xl:line-clamp-[30] max-w-full;
}
</style>