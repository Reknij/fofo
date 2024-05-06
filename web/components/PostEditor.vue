<script setup lang="ts">
import { getApiDetailError } from "~/helper";
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
import { array, number, object, string } from "yup";

const props = defineProps<{
  category: Category;
  edit?: PostInfo;
}>();
const router = useRouter();
const current = useCurrentUser();
const toast = useToast();
const state = reactive({
  title: props.edit?.title ?? "",
  content: props.edit?.content ?? "",
  content_type: props.edit?.content_type ?? ContentType.Markdown,
  tags: props.edit?.tags ?? [],
  top_index: props.edit?.top_index ?? 0,
})
const schema = object({
  title: string().required("Required"),
  content: string().nullable(),
  tags: array(),
  top_index: number().nullable(),
})
const canSetTop = computed(
  () =>
    props.category.moderator_ids.includes(current.value?.id ?? 0) ||
    current.value?.user_type === UserType.Administrator
);
async function postNow() {
  if (props.edit) {
    let { data: post, error } = await updatePost(props.edit.id, {
      target: {
        ...state,
      },
    });
    if (post.value) {
      toast.add({
        description: "Success edit the post!"
      })
      await router.push(`/post/${post.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Edit failed. ${err?.msg}`
      })
    }
  } else {
    let { data: post, error } = await createPost({
      target: {
        category_id: props.category.id,
        ...state
      },
    });
    if (post.value) {
      toast.add({
        description: "Success create the post!"
      })
      current.value!.total_post += 1;
      await router.push(`/post/${post.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
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
    <UForm class="space-y-1.5" :schema="schema" :state="state">
      <UFormGroup label="Title" path="title">
        <UInput v-model="state.title" placeholder="Your post title" />
      </UFormGroup>
      <UFormGroup label="Content" path="content">
        <MarkdownEditor v-model:value="state.content" placeholder="Your post content." />
      </UFormGroup>
      <UFormGroup label="Top index" path="top_index" v-if="canSetTop">
        <UInput type="number" v-model="state.top_index"></UInput>
      </UFormGroup>
      <UFormGroup>
        <UButton @click="postNow">
          {{ edit ? "Save edit" : "Post now" }}
        </UButton>
      </UFormGroup>
    </UForm>
  </UCard>
</template>
