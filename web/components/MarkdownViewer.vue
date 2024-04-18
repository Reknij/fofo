<script setup lang="ts">
import 'bytemd/dist/index.css'
// @ts-ignore
import { Viewer } from "@bytemd/vue-next"; // dont remove it!
import { bytemdPlugins } from '~/helper';
import { NSpace, NButton, NDivider } from 'naive-ui';

const props = defineProps<{
    content?: string,
    max_row?: number,
}>();

function isClamped(element: HTMLElement | undefined) {
    if (element) return element.scrollHeight > element.clientHeight;
    else return false;
}


const row = props.max_row ?? 60;
const myEllipsis = ref<HTMLSpanElement>();
const showButton = ref(false);
const isExpand = ref(true);

onMounted(() => {
    if (myEllipsis) {
        showButton.value = isClamped(myEllipsis.value);
    }
})
function showMore() {
    const me = myEllipsis.value;
    if (me) {
        if (me.style.webkitLineClamp == row.toString()) {
            me.style.webkitLineClamp = ''
            isExpand.value = false;
        }
        else {
            me.style.webkitLineClamp = row.toString();
            isExpand.value = true;
        }
    }
}
</script>

<template>
    <n-space vertical :size="0" justify="center">
        <span ref="myEllipsis" class="myEllipsis" :style="{ WebkitLineClamp: row }">
            <Viewer :value="content" :plugins="bytemdPlugins"></Viewer>
        </span>
        <n-divider dashed style="margin: 0" v-if="showButton">
            <n-button text @click="showMore">{{ isExpand ? 'Click to expand' : 'Click to hide' }}</n-button>
        </n-divider>
    </n-space>
</template>

<style>
.myEllipsis {
    -webkit-box-orient: vertical;
    display: -webkit-box;
    text-overflow: ellipsis;
    overflow: hidden;
}

.markdown-body {
    margin-top: 0px;
    margin-bottom: 0px;
}

.markdown-body img {
    max-width: 100%;
    height: auto;
    display: block;
    object-fit: contain;
}
</style>