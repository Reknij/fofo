<script setup lang="ts">
import { NSpace, NCard } from "naive-ui";
import { ContentType } from "~/models/util";
import type { Group } from "~/models/group";

const props = defineProps<{
  group: Group;
}>();
</script>

<template>
  <n-space vertical>
    <n-card size="small">
      <h2 style="margin: 0">
        {{ group?.title }}
      </h2>
    </n-card>
    <n-card size="small" v-if="group?.description">
      <MarkdownViewer
        v-if="group.description_content_type == ContentType.Markdown"
        :content="group.description"
      />
    </n-card>
  </n-space>
</template>
