<script setup lang="ts">
import {
  getUserNotifications,
  setUserNotificationReaded,
} from "~/api/notification";
import { timeAgo } from "~/helper";
import {
  type GetUserNotificationsQuery,
  type UserNotification,
  UserNotificationType,
} from "~/models/notification";
import {
  getPostFromExtended,
  getUserFromExtended,
  getCategoryFromExtended,
  getCommentFromExtended,
  ContentType,
} from "~/models/util";
import "bytemd/dist/index.css";
import { UserTag } from "~/models/user";
import { PostStatus } from "~/models/post";
import type { CommentInfo } from "~/models/comment";

const config = useRuntimeConfig();
const router = useRouter();
const props = defineProps<{
  limit?: number;
  max?: number;
  hide_pagination?: boolean;
}>();
const query: GetUserNotificationsQuery = {
  index: 0,
  limit: props.limit ?? config.public.limitData.any,
  extended: true,
  only_unread: false,
};

let { data: notifications, refresh: refreshNotifications } =
  await getUserNotifications(query);
let { data: unreadNotifications, refresh: refreshUnreadNotifications } =
  await getUserNotifications({
    index: 0,
    limit: 0,
    extended: false,
    only_unread: true,
  });
let page = ref(1);

async function clickPost(e: Event, post_id: number) {
  e.preventDefault();
  await router.push(`/post/${post_id}`);
}

async function changePage(page_num: number) {
  query.index = page_num - 1;
  page.value = page_num;
  await refreshNotifications();
}

function getTag(userId: number, post_id: number) {
  let tags = UserTag.Null;
  if (notifications.value) {
    let post = getPostFromExtended(notifications.value!, post_id);
    let category = getCategoryFromExtended(
      notifications.value!,
      post?.category_id ?? 0
    );
    const m = category?.moderator_ids.find((m) => m == userId);
    const op = post?.created_by_id == userId;
    if (m) {
      tags = tags | UserTag.Moderator;
    }
    if (op) {
      tags = tags | UserTag.OP;
    }
  }

  return tags;
}

function checkContent(comment?: CommentInfo) {
  return comment != undefined && comment.content_type == ContentType.Markdown;
}

async function setReaded(readed: boolean, notification: UserNotification) {
  if (unreadNotifications.value) {
    if (readed) unreadNotifications.value.data.total -= 1;
    else unreadNotifications.value.data.total += 1;
  }
  notification.readed = readed;
  await setUserNotificationReaded({
    id: notification.id,
    readed: readed,
  });
}

async function setAllReaded(readed: boolean) {
  if (notifications.value) {
    for (let i = 0; i < notifications.value.data.items.length; i++) {
      const notification = notifications.value.data.items[i];
      notification.readed = readed;
    }
    if (unreadNotifications.value) {
      unreadNotifications.value.data.total = readed
        ? 0
        : notifications.value.data.total;
    }
  }
  await setUserNotificationReaded({
    id: 0,
    readed: readed,
  });
}

function getComment(ref_id?: number) {
  if (!ref_id) return undefined;
  if (notifications.value) {
    return getCommentFromExtended(notifications.value, ref_id);
  }
  return undefined;
}

function getPost(ref_id?: number) {
  if (!ref_id) return undefined;
  if (notifications.value) {
    return getPostFromExtended(notifications.value, ref_id);
  }
  return undefined;
}

function getUser(ref_id?: number) {
  if (!ref_id) return undefined;
  if (notifications.value) {
    return getUserFromExtended(notifications.value, ref_id);
  }
  return undefined;
}

function isCommentClass(un?: UserNotification) {
  switch (un?.n_type) {
    case UserNotificationType.Comment:
    case UserNotificationType.ReplyComment:
    case UserNotificationType.LikeComment:
    case UserNotificationType.DislikeComment:
      return true;
    default:
      return false;
  }
}

function getPostId(un?: UserNotification) {
  switch (un?.n_type) {
    case UserNotificationType.Comment:
    case UserNotificationType.ReplyComment:
    case UserNotificationType.LikeComment:
    case UserNotificationType.DislikeComment:
      return getComment(un.ref_id)?.post_id;

    case UserNotificationType.LikePost:
    case UserNotificationType.DislikePost:
      return getPost(un.ref_id)?.id;

    default:
      break;
  }
}

function getNotificationTitle(un?: UserNotification) {
  switch (un?.n_type) {
    case UserNotificationType.Comment:
      return `${getUser(un.created_by_id)?.alias} comment your post.`;
    case UserNotificationType.ReplyComment:
      return `${getUser(un.created_by_id)?.alias} reply your comment.`;
    case UserNotificationType.LikeComment:
      return `${getUser(un.created_by_id)?.alias} like your comment.`;
    case UserNotificationType.LikePost:
      return `${getUser(un.created_by_id)?.alias} like your post.`;
    case UserNotificationType.DislikeComment:
      return `${getUser(un.created_by_id)?.alias} dislike your comment.`;
    case UserNotificationType.DislikePost:
      return `${getUser(un.created_by_id)?.alias} dislike your post.`;
    default:
      return "Unknown title.";
  }
}
</script>

<template>
  <div class="space-y-2">
    <UAlert v-if="notifications?.data.total === 0" title="No notifications yet~" />
    <UPagination v-if="!hide_pagination && notifications?.data.total" :model-value="query.index + 1"
      @update:model-value="(v: number) => query.index = v + 1" :page-count="limit ?? config.public.limitData.any"
      :total="max ?? notifications.data.total" />

    <UBadge class="gap-x-2" variant="soft" v-if="unreadNotifications?.data.total">
      <span v-if="unreadNotifications?.data.total === 0">All readed</span>
      <span v-else>All unread</span>
      <UToggle :model-value="unreadNotifications?.data.total === 0"
        @update:model-value="(value: boolean) => setAllReaded(value)" />
    </UBadge>

    <UCard v-for="notification in notifications?.data.items">
      <div class="flex flex-col justify-center">
        <div class="flex flex-wrap items-center gap-x-2 gap-y-2">
          <span class="code">{{ timeAgo(notification.created_at) }}</span>
          <div class="flex items-center gap-x-2">
            <span v-if="notification.readed">Readed</span>
            <span v-else>Unread</span>
            <UToggle :model-value="notification.readed"
              @update:model-value="(value: boolean) => setReaded(value, notification)" />
          </div>
        </div>
        <span>{{ getNotificationTitle(notification) }}</span>
        <div class="flex gap-x-1 items-center" v-if="getPost(getPostId(notification))">
          <UIcon v-if="getPost(getPostId(notification))!.top_index" name="i-ph-push-pin" dynamic />
          <UIcon v-if="getPost(getPostId(notification))!.status === PostStatus.Banned" name="i-heroicons-lock-closed"
            class="text-red-500" />
          <UIcon v-else-if="getPost(getPostId(notification))!.status === PostStatus.Archived"
            name="i-heroicons-lock-closed" class="text-yellow-500" />
          <ULink class="line-clamp-1" :to="`/post/${getPost(getPostId(notification))!.id}`" active-class="text-primary"
            inactive-class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
            {{ getPost(getPostId(notification))!.title }}
          </ULink>
        </div>
        <div class="flex flex-col justify-center gap-y-1">
          <FofoUserAvatar :user="getUser(notification.created_by_id)" :tag="getTag(
            notification.created_by_id,
            getComment(notification.ref_id)?.post_id ?? 0
          )">
            <div class="flex flex-row items-center space-x-1" v-if="
              isCommentClass(notification) &&
              getComment(notification.ref_id)?.reply_comment_id
            ">
              <UIcon name="i-mdi-menu-right-outline" />
              <UBadge variant="subtle">You</UBadge>
            </div>
          </FofoUserAvatar>
          <MarkdownViewer v-if="checkContent(getComment(notification.ref_id))"
            :content="getComment(notification.ref_id)?.content" :max_row="1" />
        </div>
      </div>
    </UCard>
  </div>
</template>
