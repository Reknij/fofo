<script setup lang="ts">
import { getCategory } from "~/api/category";
import { UserType } from "~/models/user";
import { getComment } from "~/api/comment";
import { getPost } from "~/api/post";
import type { CommentInfo } from "~/models/comment";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();

const toast = useToast();
const user = useCurrentUser();
const query = router.currentRoute.value.query as any as {
  post_id: string;
  reply_comment_id?: string;
  edit_id?: string;
};
const { data: post } = await getPost(Number.parseInt(query.post_id), {});
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
      toast.add({
        color: 'yellow',
        description: "You are not the original poster and don't have permission to edit."
      })
      authorized.value = false;
    } else {
      comment.value = data.value;
      const { data: p } = await getPost(comment.value.post_id, {});
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
const links = [
  {
    label: 'Categories',
    to: '/categories',
  },
  {
    label: category.value?.title ?? "Unknown",
    to: category.value ? `/category/${category.value.id}` : '/categories'
  },
  {
    label: post.value ? post.value.id.toString() : 'Unknown',
    to: post.value ? `/post/${post.value.id}` : undefined,
  },
  {
    label: editMode.value ? 'Edit comment' : 'Create comment'
  }
]
</script>

<template>
  <ClientOnly>
    <div class="space-y-2">
      <FofoBreadcrumb :links="links" />
      <UAlert v-if="!post" title="Post not found." />
      <UAlert v-else-if="!category" title="Category not found." />
      <UAlert v-else-if="query.reply_comment_id != undefined && !replyComment" title="Reply comment not found." />
      <UAlert v-else-if="!user" title="Please login to continue!" />
      <UAlert v-else-if="editMode && !comment" title="Not found the comment!" />
      <CommentEditor v-else :post="post" :category="category" :comment="replyComment" :edit="comment"></CommentEditor>
    </div>
  </ClientOnly>
</template>
