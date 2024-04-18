<script setup lang="ts">
import { getCategory } from "~/api/category";
import { UserType } from "~/models/user";
import { NCard, useMessage, useLoadingBar } from "naive-ui";
import { getComment } from "~/api/comment";
import { getPost } from "~/api/post";
import type { CommentInfo } from "~/models/comment";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();
const loadingBar = useLoadingBar();
loadingBar.start();

const message = useMessage();
const user = useCurrentUser();
const query = router.currentRoute.value.query as any as {
  post_id: string;
  reply_comment_id?: string;
  edit_id?: string;
};
const { data: post } = await getPost(Number.parseInt(query.post_id));
const { data: category } = await getCategory(post.value?.category_id ?? 0);
const replyComment = ref();

const authorized = ref(query.edit_id == undefined);
const editMode = computed(() => query.edit_id !== undefined);
const comment = ref<CommentInfo>();

if (query.edit_id) {
  const { data } = await getComment(Number.parseInt(query.edit_id));
  if (data.value && category.value) {
    if (
      data.value.created_by_id != user.value?.id &&
      !category.value.moderator_ids.includes(user.value?.id ?? 0) &&
      user.value?.user_type !== UserType.Administrator
    ) {
      message.error("You are not the original poster.");
      authorized.value = false;
    } else {
      comment.value = data.value;
      const { data: p } = await getPost(comment.value.post_id);
      post.value = p.value;
      authorized.value = true;
    }
  }
}
if (query.reply_comment_id) {
  const { data: comment } = await getComment(
    Number.parseInt(query.reply_comment_id)
  );
  replyComment.value = comment.value;
}

onMounted(() => loadingBar.finish());
</script>

<template>
  <ClientOnly>
    <n-card v-if="!post" size="small">Post not found.</n-card>
    <n-card v-else-if="!category" size="small">Category not found.</n-card>
    <n-card v-else-if="query.reply_comment_id != undefined && !replyComment"
      >Reply comment not found</n-card
    >
    <n-card v-else-if="!user">Please login to continue!</n-card>
    <n-card v-else-if="editMode && !comment">Not found the comment!</n-card>
    <CommentEditor
      v-else
      :post="post"
      :category="category"
      :comment="replyComment"
      :edit="comment"
    ></CommentEditor>
  </ClientOnly>
</template>
