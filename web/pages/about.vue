<script setup lang="ts">
import data from '/public/about.md?raw'

</script>

<template>
    <UCard>
        <MarkdownViewer v-if="data" :content="data" />
        <span v-else>Can't fetch the <span class="code">about.md</span>.</span>
    </UCard>
</template>