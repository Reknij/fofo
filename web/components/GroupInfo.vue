<script setup lang="ts">
import { ContentType } from "~/models/util";
import { type Group } from "~/models/group";

const props = defineProps<{
  group: Group;
}>();
</script>

<template>
  <div class="space-y-1.5">
    <UCard>
      <template #header>
        <div class="flex items-center gap-1.5">
          <UIcon name="i-heroicons-user-group" />
          <span class="text-2xl">{{ group.title }}</span>
        </div>
      </template>
      <MarkdownViewer v-if="group.description_content_type == ContentType.Markdown" :content="group.description" />
    </UCard>
  </div>
</template>
