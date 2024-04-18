<script setup lang="ts">
import { UserType } from '~/models/user';

defineProps<{
    value: UserType
}>()
defineEmits<{
    (e: 'update:value', v: UserType): void,
}>()
const options: any = [
    UserType.Guest,
    UserType.General,
    UserType.Administrator,
];
const getLabel: any = (v: UserType) => {
    switch (v) {
        case UserType.Guest:
            return "Guest"
        case UserType.General:
            return "General"
        case UserType.Administrator:
            return "Administrator"
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