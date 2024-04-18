<script setup lang="ts">
import { getApiDetailError } from "~/helper";
import {
  NSpace,
  NInputNumber,
  NButton,
  NCard,
  NPageHeader,
  useMessage,
} from "naive-ui";
import type { CommentInfo } from "~/models/comment";
import { ContentType } from "~/models/util";
import { createComment, updateComment } from "~/api/comment";
import type { PostInfo } from "~/models/post";
import { useCurrentUser } from "~/states/auth";
import { UserType } from "~/models/user";
import type { Category } from "~/models/category";
import MarkdownEditor from "./MarkdownEditor.vue";

const props = defineProps<{
  post: PostInfo;
  category: Category;
  comment?: CommentInfo;
  edit?: CommentInfo;
}>();

const router = useRouter();
const message = useMessage();
const content = ref(props.edit?.content ?? "");
const topIndex = ref(props.edit?.top_index ?? 0);
const config = useRuntimeConfig();
const current = useCurrentUser();
const canSetTop = computed(
  () =>
    props.post.created_by_id === current.value?.id ||
    props.category.moderator_ids.includes(current.value?.id ?? 0) ||
    current.value?.user_type === UserType.Administrator
);

async function commentNow() {
  let comment: CommentInfo | null;
  let error;
  if (props.edit) {
    let { data: commentResp, error: errorResp } = await updateComment(
      props.edit.id,
      {
        target: {
          content: content.value,
          content_type: ContentType.Markdown,
          top_index: topIndex.value,
        },
      }
    );
    comment = commentResp.value;
    error = errorResp.value;
    if (comment) {
    message.success("Success update the comment!");
    await router.push(
      `/post/${props.post.id}`
    );
  } else {
    const err = getApiDetailError(error);
    message.error(`(${err?.code}) Create failed. ${err?.msg}`);
  }
  } else {
    let { data: commentResp, error: errorResp } = await createComment({
      target: {
        content: content.value,
        content_type: ContentType.Markdown,
        post_id: props.post.id,
        reply_comment_id: props.comment?.id ?? 0,
        top_index: topIndex.value,
      },
    });
    comment = commentResp.value;
    error = errorResp.value;
    if (comment) {
      message.success("Success create the comment!");
      current.value!.total_comment += 1;
      await router.push(
        `/post/${props.post.id}?comment_page=${Math.ceil(
          props.post.total_comment_post / config.public.limitData.comments
        )}`
      );
    } else {
      const err = getApiDetailError(error);
      message.error(`(${err?.code}) Create failed. ${err?.msg}`);
    }
  }
}

async function handleBack() {
  router.replace(`/post/${props.post.id}`);
}
</script>

<template>
  <n-card size="small">
    <n-page-header :title="post.title" @back="handleBack">
      <n-space vertical>
        <MarkdownEditor
          v-model:value="content"
          placeholder="Your comment content."
        />
        <n-space align="center" v-if="canSetTop">
          Top Index
          <n-input-number v-model:value="topIndex"></n-input-number>
        </n-space>
        <n-button @click="commentNow">
          {{ edit ? "Save edit" : "Comment now" }}
        </n-button>
      </n-space>
    </n-page-header>
  </n-card>
</template>

<style>
.bytemd {
  height: 400px;
}
</style>
