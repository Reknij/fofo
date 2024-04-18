<script setup lang="ts">
import { NDropdown, NIcon } from 'naive-ui';
import type { DropdownMixedOption, OnUpdateValue } from 'naive-ui/es/dropdown/src/interface';
import { MoreOutlined } from '@vicons/antd'

const props = defineProps<{
    options: DropdownMixedOption[],
}>();
const emit = defineEmits<{
    (e: 'select', v: any): void,
}>()

</script>

<template>
    <div>
        <n-dropdown trigger="click" :options="options" @select="key => $emit('select', key)">
        <n-icon class="n-button n-button--default-type n-button--medium-type" :component="MoreOutlined" :size="20" />
    </n-dropdown>
    </div>
</template>