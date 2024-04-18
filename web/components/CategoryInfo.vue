<script setup lang="ts">
import { ContentType } from "~/models/util";
import { type Category, CategoryStatus } from "~/models/category";

const props = defineProps<{
  category: Category;
}>();
</script>

<template>
  <div class="space-y-2">
    <UAlert title="Archived" color="yellow"
      description="Category is archived. Meaning you can't create, update or comment post on it."
      v-if="category.status === CategoryStatus.Archived" />
    <UAlert title="Stopped" color="red"
      description="Category is stopped. Meaning you can't create, update or comment on or even view posts on it."
      v-else-if="category.status === CategoryStatus.Stopped" />
    <UCard :ui="{ body: { padding: '!p-0' } }">
      <template #header>
        <div class="flex items-center gap-x-2">
          <UIcon name="i-heroicons-square-3-stack-3d" />
          <span class="text-2xl">{{ category.title }}</span>
        </div>
      </template>
      <MarkdownViewer v-if="category.description_content_type == ContentType.Markdown"
        :content="category.description" />
    </UCard>
  </div>
</template>
