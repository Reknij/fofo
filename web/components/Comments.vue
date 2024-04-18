<script setup lang="ts">
import { getComment, getComments, setCommentStatus } from "~/api/comment";
import {
  getApiDetailError,
  hasManagePermission,
  isEditable,
  timeAgo,
} from "~/helper";
import {
  GetCommentsSort,
  type CommentInfo,
  CommentStatus,
} from "~/models/comment";
import { type PostInfo, PostStatus } from "~/models/post";
import { type SafeUserInfo, UserTag } from "~/models/user";
import {
  ContentType,
  getCategoryFromExtended,
  getCommentLikeStatusFromExtended,
  getUserFromExtended,
} from "~/models/util";
import type { Category } from "~/models/category";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import { useCurrentUser } from "~/states/auth";
import { useServerInfo } from "~/states/server";
import type { DropdownItem } from '#ui/types'
import PostAndCommentInfoModal from "./PostAndCommentInfoModal.vue";

const toast = useToast();
const modal = useModal();
const config = useRuntimeConfig();
const router = useRouter();
const props = defineProps<{
  post: PostInfo;
  category: Category;
  limit?: number;
  parent_comment?: CommentInfo;
  parent_comment_user?: SafeUserInfo;
  query_pagination?: boolean;
}>();

const currentUser = useCurrentUser();
const canManage = ref(hasManagePermission(currentUser.value, props.category));
const routeQuery = router.currentRoute.value.query as any as {
  comment_page?: string;
};
let routePage = Number.parseInt(routeQuery.comment_page ?? "");
if (isNaN(routePage)) routePage = 0;
const page = ref(routePage > 0 ? routePage : 1);
if (!props.query_pagination) {
  page.value = 1;
}

const query = {
  post_id: props.post.id,
  parent_id: props.parent_comment?.id ?? 0,
  index: props.parent_comment ? 0 : page.value - 1,
  limit: props.limit ?? config.public.limitData.comments,
  sort: GetCommentsSort.Id,
  desc: false,
  extended: true,
};

function updateQuery(refreshNow = true) {
  query.post_id = props.post.id;
  query.parent_id = props.parent_comment?.id ?? 0;
  query.index = props.parent_comment ? 0 : page.value - 1;
  if (refreshNow) refresh();
}

const { data: comments, error, refresh } = await getComments(query);
watch(() => props, () => updateQuery());

function getActionOptions(comment: CommentInfo) {
  const actionOptions: DropdownItem[][] = [
    [{
      label: "Info",
      click() {
        modal.open(PostAndCommentInfoModal, {
          info: comment,
        })
      }
    }, {
      label: "Reply",
      click() {
        goComment(comment)
      }
    }],
  ];
  if (canManage.value) {
    actionOptions.push([{
      label: "Edit",
      click() {
        editComment(comment);
      }
    }])
  }
  if (canManage) {
    actionOptions.push([{
      label: "Reactive",
      click() {
        setStatus(comment, CommentStatus.Active)
      },
    },
    {
      label: "Ban",
      click() {
        setStatus(comment, CommentStatus.Banned)
      },
    }
    ])
  }
  return actionOptions;
}

async function editComment(comment: CommentInfo) {
  const canEdit = await isEditable(comment.created_at);
  const serverInfo = useServerInfo();
  if (canEdit || canManage.value)
    await router.push(
      `/publish/comment?post_id=${comment.post_id}&edit_id=${comment.id}`
    );
  else if (serverInfo.value)
    toast.add({
      color: 'yellow',
      description: `Unable to edit for ${serverInfo.value.editable_seconds / 60
        } minute(s) after posting`,
    })
  else toast.add({
    color: 'red',
    description: "Cannot get server information."
  })
}

async function setStatus(comment: CommentInfo, status: CommentStatus) {
  const { error } = await setCommentStatus(comment.id, {
    status,
  });
  if (!error.value) {
    let statusText = Object.values(CommentStatus)[status];
    if (comment.status === CommentStatus.Banned) {
      // previous is banned and it don't have content.
      let { data: newComment } = await getComment(comment.id);
      if (newComment.value) comment.content = newComment.value.content;
      else toast.add({
        color: 'red',
        description: `Can't get target comment, id '${comment.id}'`
      })
    }

    comment.status = status;
    toast.add({
      description: `Comment id '${comment.id}': ${statusText}`
    })
  } else {
    let err = getApiDetailError(error.value);
    toast.add({
      color: 'red',
      description: `(${err?.code}) ${err?.msg}`
    })
  }
}

const getTotalComment = () =>
  props.parent_comment?.total_comment ?? props.post.total_comment_post;

async function changePage(page_num: number) {
  query.index = page_num - 1;
  page.value = page_num;
  if (props.query_pagination) {
    router.push({
      query: {
        comment_page: page_num,
      },
    });
  }
  await refresh();
}

function getTag(userId: number) {
  let tags = UserTag.Null;
  const m = props.category.moderator_ids.find((m) => m == userId);
  const op = props.post.created_by_id == userId;
  if (m) {
    tags = tags | UserTag.Moderator;
  }
  if (op) {
    tags = tags | UserTag.OP;
  }

  return tags;
}

async function statusChanged(comment: CommentInfo, status: LikeStatus | null) {
  if (comments.value) {
    if (!comments.value.comments_like_status) {
      comments.value.comments_like_status = {};
    }
    const previous = comments.value.comments_like_status[comment.id];
    if (previous) {
      previous.is_like ? (comment.likes -= 1) : (comment.dislikes -= 1);
    }
    if (status) {
      status.is_like ? (comment.likes += 1) : (comment.dislikes += 1);
    }
    comments.value.comments_like_status[comment.id] = status;
  }
}

async function goComment(comment?: CommentInfo) {
  let go = `/publish/comment?post_id=${props.post.id}`;
  if (comment) {
    go = go + `&reply_comment_id=${comment.id}`;
  }
  await router.push(go);
}
</script>

<template>
  <div class="space-y-2 m-2" v-if="comments">
    <UPagination :model-value="query.index + 1" @update:model-value="changePage" :page-count="query.limit"
      :total="getTotalComment()" />

    <div v-for="(comment, i) in comments.data.items" class="flex flex-col justify-center gap-y-1">
      <div class="flex flex-row items-center justify-between">
        <div>
          <UIcon v-if="comment.top_index! > 0" name="i-ic-outline-push-pin" />
          <FofoUserAvatar :user="getUserFromExtended(comments, comment.created_by_id)!"
            :tag="getTag(comment.created_by_id)">
            <div class="flex flex-row flex-wrap items-center gap-x-1 gap-y-1" v-if="comment.reply_comment_id > 0">
              <UIcon name="i-mdi-menu-right-outline" />
              <UBadge variant="subtle" v-if="comment.reply_user_id === currentUser?.id">You</UBadge>
              <FofoUserAvatar v-else :disable-avatar="true"
                :user="getUserFromExtended(comments, comment.reply_user_id)!" :tag="getTag(comment.reply_user_id)">
              </FofoUserAvatar>
              <span>{{ timeAgo(comment.created_at) }} </span>
            </div>
          </FofoUserAvatar>
        </div>
        <UDropdown :items="getActionOptions(comment)">
          <UButton icon="i-heroicons-ellipsis-vertical" variant="ghost" />
        </UDropdown>
      </div>
      <MarkdownViewer class="-m-2" v-if="
        comment.status === CommentStatus.Active &&
        comment.content_type == ContentType.Markdown
      " :content="comment.content" :max_row="2" />
      <UAlert v-else color="red" variant="soft" title="Comment is banned!" />
      <div class="flex items-center space-x-2" v-if="comment.status === CommentStatus.Active">
        <LikeStatusComponent :info="comment" :flag="LikeStatusFlag.TargetComment" :status="getCommentLikeStatusFromExtended(comments, comment.id)
          " @statusChanged="(s: LikeStatus | null) => statusChanged(comment, s)" />
        <UButton v-if="post.status === PostStatus.Active" color="primary" variant="link" :padded="false"
          @click="(comment as any).showComments = !(comment as any).showComments">
          {{ (comment as any).showComments ? "Hide comments" : "View comments" }}</UButton>
      </div>
      <div class=" bg-gray-100 dark:bg-slate-800 shadow-inner rounded-2xl my-1 p-2"
        v-if="(comment as any).showComments">
        <Comments v-if="comment.parent_id == 0 && comment.total_comment > 0" :post="post" :parent_comment="comment"
          :category="category" :limit="Math.round((limit ?? config.public.limitData.comments) / 2)"
          :parent_comment_user="getUserFromExtended(comments, comment.created_by_id)
            "></Comments>
      </div>
      <UDivider v-if="i < comments.data.items.length - 1" />
    </div>
  </div>
</template>
