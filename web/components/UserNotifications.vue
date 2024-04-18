<script setup lang="ts">
import {
  NList,
  NSpace,
  NListItem,
  NGrid,
  NGi,
  NEllipsis,
  NIcon,
  NPagination,
  NButton,
  NText,
  NTag,
  NSwitch,
} from "naive-ui";
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
import type { CommentInfo } from "~/models/comment";
import { RightOutlined } from "@vicons/antd";

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
  <n-space vertical>
    <n-space>
      <n-pagination
        simple
        v-if="!hide_pagination && notifications"
        v-model:page="page"
        :page-size="limit ?? config.public.limitData.any"
        :item-count="max ?? notifications.data.total"
        :on-update:page="changePage"
      />
      <n-switch
        :value="unreadNotifications?.data.total === 0"
        @update:value="(value: boolean) => setAllReaded(value)"
        v-if="unreadNotifications?.data.total"
      >
        <template #checked> All readed </template>
        <template #unchecked> All unread </template>
      </n-switch>
    </n-space>
    <n-list v-if="notifications">
      <n-list-item v-if="notifications.data.items.length == 0">
        <n-space vertical>
          <span>No notifications yet~</span>
        </n-space>
      </n-list-item>
      <n-list-item v-for="notification in notifications.data.items">
        <n-space vertical>
          <n-space>
            <n-text code>{{ timeAgo(notification.created_at) }}</n-text>
            <n-switch
              :value="notification.readed"
              @update:value="(value: boolean) => setReaded(value, notification)"
            >
              <template #checked> Readed </template>
              <template #unchecked> Unread </template>
            </n-switch>
          </n-space>
          <span>{{ getNotificationTitle(notification) }}</span>
          <n-space align="center" :wrap="false">
            <slot name="prefix"></slot>
            <n-grid :cols="1">
              <n-gi>
                <n-button
                  tag="a"
                  :href="`/post/${getPostId(notification)}`"
                  text
                  style="font-size: large; width: 100%"
                  @click="(e: Event) => clickPost(e, getPostId(notification) ?? 0)"
                >
                  <n-ellipsis style="padding: 5px 0px; text-align: center">
                    {{ getPost(getPostId(notification))?.title }}
                  </n-ellipsis>
                </n-button>
              </n-gi>
            </n-grid>
          </n-space>
          <n-space vertical :size="0">
            <FofoUserAvatar
              :user="getUser(notification.created_by_id)"
              :tag="
                getTag(
                  notification.created_by_id,
                  getComment(notification.ref_id)?.post_id ?? 0
                )
              "
              style="margin-bottom: -10px"
            >
              <n-space
                align="center"
                v-if="
                  isCommentClass(notification) &&
                  getComment(notification.ref_id)?.reply_comment_id
                "
              >
                <n-icon>
                  <RightOutlined></RightOutlined>
                </n-icon>
                <n-tag :bordered="false" type="info">You</n-tag>
              </n-space>
            </FofoUserAvatar>
            <MarkdownViewer
              v-if="checkContent(getComment(notification.ref_id))"
              :content="getComment(notification.ref_id)?.content"
              :max_row="1"
            />
          </n-space>
        </n-space>
      </n-list-item>
    </n-list>
  </n-space>
</template>
