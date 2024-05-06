<script setup lang="ts">
const props = defineProps<{
    modelValue: string
}>()
defineEmits<{
    (e: 'update:modelValue', v: string): void,
}>()

const GOOGLE_SEARCH_QUERY = "https://www.google.com/search?q=";
const SEARCH_ENGINE = GOOGLE_SEARCH_QUERY;
function search(v: string) {
    window.open(`${SEARCH_ENGINE}site:${window.location.host}/post ${v}`, '_blank')
}
</script>

<template>
    <UInput class="w-full" :ui="{ rounded: 'rounded-2xl', icon: { trailing: { pointer: '' } } }"
        :model-value="modelValue" @update:model-value="v => $emit('update:modelValue', v)" size="sm"
        placeholder="Search.." @keyup.enter="search(modelValue)">
        <template #trailing>
            <UButton as="span" icon="i-heroicons-magnifying-glass-20-solid" @click="search(modelValue)" color="primary"
                variant="link" :padded="false" />
        </template>
    </UInput>
</template>