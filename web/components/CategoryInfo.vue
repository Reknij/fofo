<script setup lang="ts">
import { NSpace, NCard, NAlert } from "naive-ui";
import { ContentType } from "~/models/util";
import { type Category, CategoryStatus } from "~/models/category";

const props = defineProps<{
  category: Category;
}>();
</script>

<template>
  <n-space vertical>
    <n-alert
      title="Note"
      type="warning"
      v-if="category.status === CategoryStatus.Archived"
    >
      Category is archived. Meaning you can't create, update or comment post on it.
    </n-alert>
    <n-alert
      title="Note"
      type="error"
      v-else-if="category.status === CategoryStatus.Stopped"
    >
      Category is stopped. Meaning you can't create, update or comment on or even view posts on it.
    </n-alert>
    <n-card size="small">
      <h2 style="margin: 0">
        {{ category?.title }}
      </h2>
    </n-card>
    <n-card size="small" v-if="category?.description">
      <MarkdownViewer
        v-if="category.description_content_type == ContentType.Markdown"
        :content="category.description"
      />
    </n-card>
  </n-space>
</template>
