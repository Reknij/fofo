<script setup lang="ts">
import { CategoryStatus } from '~/models/category';

defineProps<{
    value: CategoryStatus
}>()
defineEmits<{
    (e: 'update:value', v: CategoryStatus): void,
}>()
const options: any = [
    CategoryStatus.Active,
    CategoryStatus.Archived,
    CategoryStatus.Stopped,
];
const getLabel: any = (v: CategoryStatus) => {
    switch (v) {
        case CategoryStatus.Active:
            return "Active"
        case CategoryStatus.Archived:
            return "Archived"
        case CategoryStatus.Stopped:
            return "Stopped"
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