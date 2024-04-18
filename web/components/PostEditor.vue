<script setup lang="ts">
import { getApiDetailError } from "~/helper";
import {
  NSpace,
  NInput,
  NButton,
  NCard,
  NDynamicTags,
  NInputNumber,
  NPageHeader,
  useMessage,
} from "naive-ui";
import type { PostToCreate, PostToUpdate } from "~/models/post";
import { ContentType } from "~/models/util";
import { createPost, updatePost } from "~/api/post";
import type { Category } from "~/models/category";
import "bytemd/dist/index.css";
import { bytemdPlugins } from "~/helper";
import type { PostInfo } from "~/models/post";
import { useCurrentUser } from "~/states/auth";
import { UserType } from "~/models/user";
import MarkdownEditor from "./MarkdownEditor.vue";

const props = defineProps<{
  category: Category;
  edit?: PostInfo;
}>();
const router = useRouter();
const current = useCurrentUser();
const message = useMessage();
const title = ref(props.edit?.title ?? "");
const tags = ref<string[]>(props.edit?.tags ?? []);
const content = ref(props.edit?.content ?? "");
const topIndex = ref(props.edit?.top_index ?? 0);
const canSetTop = computed(
  () =>
    props.category.moderator_ids.includes(current.value?.id ?? 0) ||
    current.value?.user_type === UserType.Administrator
);
async function postNow() {
  if (props.edit) {
    let query: PostToUpdate = {
      title: title.value,
      content: content.value,
      content_type: ContentType.Markdown,
      tags: tags.value,
      top_index: topIndex.value,
    };
    let { data: post, error } = await updatePost(props.edit.id, {
      target: query,
    });
    if (post.value) {
      message.success("Success edit the post!");
      await router.push(`/post/${post.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      message.error(`(${err?.code}) Edit failed. ${err?.msg}`);
    }
  } else {
    let query: PostToCreate = {
      title: title.value,
      content: content.value,
      content_type: ContentType.Markdown,
      category_id: props.category.id,
      tags: tags.value,
      top_index: topIndex.value,
    };
    let { data: post, error } = await createPost({
      target: query,
    });
    if (post.value) {
      message.success("Success create the post!");
      current.value!.total_post += 1;
      await router.push(`/post/${post.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      message.error(`(${err?.code}) Create failed. ${err?.msg}`);
    }
  }
}

async function handleBack() {
  router.replace(`/category/${props.category.id}`);
}
</script>

<template>
  <n-card size="small">
    <n-page-header :title="category.title" @back="handleBack">
      <n-space vertical>
        <n-input
          v-model:value="title"
          type="text"
          placeholder="Your post title"
          clearable
        />
        <n-dynamic-tags v-model:value="tags" :max="9" />
        <MarkdownEditor
          v-model:value="content"
          placeholder="Your post content."
        />
        <n-space align="center" v-if="canSetTop">
          Top Index
          <n-input-number v-model:value="topIndex"></n-input-number>
        </n-space>
        <n-button @click="postNow">
          {{ edit ? "Save edit" : "Post now" }}
        </n-button>
      </n-space>
    </n-page-header>
  </n-card>
</template>

<style>
.bytemd {
  height: 400px;
}

.bytemd-fullscreen.bytemd {
  z-index: 99999;
}
</style>
