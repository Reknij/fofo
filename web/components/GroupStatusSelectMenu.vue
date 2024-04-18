<script setup lang="ts">
import { GroupStatus } from '~/models/group';

defineProps<{
    value: GroupStatus
}>()
defineEmits<{
    (e: 'update:value', v: GroupStatus): void,
}>()
const options: any = [
    GroupStatus.Active,
    GroupStatus.Observer,
    GroupStatus.OnlyComment,
];
const getLabel: any = (v: GroupStatus) => {
    switch (v) {
        case GroupStatus.Active:
            return "Active"
        case GroupStatus.Observer:
            return "Observer"
        case GroupStatus.OnlyComment:
            return "Only Comment"
        default:
            break;
    }
}
</script>

<template>
    <USelectMenu :model-value="value" @update:model-value="(v: any) => $emit('update:value', v)" :options="options">
        <template #label>
            {{ getLabel(value) }}
        </template>
        <template #option="{ option }">
            {{ getLabel(option) }}
        </template>
    </USelectMenu>
</template>