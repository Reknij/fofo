<script setup lang="ts">
import { getApiDetailError } from "~/helper";
import type { CommentInfo } from "~/models/comment";
import { ContentType } from "~/models/util";
import { createComment, updateComment } from "~/api/comment";
import type { PostInfo } from "~/models/post";
import { useCurrentUser } from "~/states/auth";
import { UserType } from "~/models/user";
import type { Category } from "~/models/category";
import MarkdownEditor from "./MarkdownEditor.vue";
import { number, object, string } from "yup";

const props = defineProps<{
  post: PostInfo;
  category: Category;
  comment?: CommentInfo;
  edit?: CommentInfo;
}>();

const router = useRouter();
const toast = useToast();
const state = reactive({
  content: props.edit?.content ?? '',
  content_type: ContentType.Markdown,
  top_index: props.edit?.top_index ?? 0,
})

const schema = object({
  content: string().required("Required"),
  top_index: number().nullable(),
})
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
          ...state
        },
      }
    );
    comment = commentResp.value;
    error = errorResp.value;
    if (comment) {
      toast.add({
        description: "Success update the comment!"
      })
      await router.push(
        `/post/${props.post.id}`
      );
    } else {
      const err = getApiDetailError(error);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Create failed. ${err?.msg}`
      })
    }
  } else {
    let { data: commentResp, error: errorResp } = await createComment({
      target: {
        ...state,
        post_id: props.post.id,
        reply_comment_id: props.comment?.id ?? 0,
      },
    });
    comment = commentResp.value;
    error = errorResp.value;
    if (comment) {
      toast.add({
        description: "Success create the comment!"
      })
      current.value!.total_comment += 1;
      await router.push(
        `/post/${props.post.id}?comment_page=${props.post.total_comment_post ? Math.ceil(
          props.post.total_comment_post / config.public.limitData.comments
        ) : 1}`
      );
    } else {
      const err = getApiDetailError(error);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Create failed. ${err?.msg}`
      })
    }
  }
}
</script>

<template>
  <UCard>
    <UForm class="space-y-2" :schema="schema" :state="state">
      <UFormGroup label="Content" path="content">
        <MarkdownEditor v-model:value="state.content" placeholder="Your comment content." />
      </UFormGroup>
      <UFormGroup v-if="canSetTop" label="Top index" path="top_index">
        <UInput type="number" v-model="state.top_index" />
      </UFormGroup>
      <UFormGroup>
        <UButton @click="commentNow">
          {{ edit ? "Save edit" : "Comment now" }}
        </UButton>
      </UFormGroup>
    </UForm>
  </UCard>
</template>
