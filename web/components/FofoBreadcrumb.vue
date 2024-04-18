<script setup lang="ts">
import type { BreadcrumbLink } from "#ui/types"

defineProps<{
    links: BreadcrumbLink[]
}>()
const homeLink = {
    icon: 'i-heroicons-home',
    label: '',
    to: '/'
}
</script>

<template>
    <div>
        <UCard>
            <UBreadcrumb :links="[homeLink, ...links]"></UBreadcrumb>
        </UCard>
    </div>
</template>