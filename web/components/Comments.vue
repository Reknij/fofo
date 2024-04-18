<script setup lang="ts">
import {
  NCard,
  NList,
  NListItem,
  NSpace,
  NButton,
  NPagination,
  NText,
  NIcon,
  useMessage,
  useDialog,
} from "naive-ui";
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
import { MoreOutlined, RightOutlined, PushpinOutlined } from "@vicons/antd";
import type { Category } from "~/models/category";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import { useCurrentUser } from "~/states/auth";
import type { MenuMixedOption } from "naive-ui/es/menu/src/interface";
import { useServerInfo } from "~/states/server";

const message = useMessage();
const dialog = useDialog();
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
  const actionOptions: MenuMixedOption[] = [
    {
      label: "Info",
      key: "info",
    },
    {
      label: "Edit",
      key: "edit",
      show: currentUser.value?.id === comment.created_by_id || canManage.value,
    },
    {
      label: "Reactive",
      key: "active",
      show: canManage.value,
    },
    {
      label: "Ban",
      key: "ban",
      show: canManage.value,
    },
  ];
  return actionOptions;
}

async function actionSelect(key: string, comment: CommentInfo) {
  switch (key) {
    case "info":
      const commentCreatedBy = comments.value
        ? getUserFromExtended(comments.value, comment.created_by_id)
        : undefined;
      const commentLastEditdBy = comments.value
        ? getUserFromExtended(comments.value, comment.last_edit_by_id)
        : undefined;
      const createdBy = `${commentCreatedBy?.alias}@${commentCreatedBy?.username}`;
      const lastEditBy =
        comment.last_edit_by_id > 0
          ? `${commentLastEditdBy?.alias}@${commentLastEditdBy?.username}`
          : createdBy;
      dialog.info({
        title: "Info",
        content: () =>
          h(NSpace, { vertical: true, justify: "center" }, [
            h("span", `Post title: ${props.post.title}`),
            h("span", `Post id: ${props.post.id}`),
            h("span", `Comment id: ${comment.id}`),
            h("span", `Reply user id: ${comment.reply_user_id}`),
            h("span", `Reply comment id: ${comment.reply_comment_id}`),
            h(
              "span",
              `Created at: ${new Date(
                comment.created_at * 1000
              ).toDateString()}`
            ),
            h("span", `Created by: ${createdBy}`),
            h(
              "span",
              `Last edit at: ${new Date(
                comment.last_edit_at * 1000
              ).toDateString()}`
            ),
            h("span", `Last edit by: ${lastEditBy}`),
            h(
              "span",
              `Comment content type: ${
                Object.values(ContentType)[comment.content_type]
              }`
            ),
            h(
              "span",
              `Comment status: ${Object.values(CommentStatus)[comment.status]}`
            ),
            h(
              "span",
              `Top Index: ${comment.top_index ?? 0}`
            ),
          ]),
      });
      break;
    case "edit":
      await editComment(comment);
      break;
    case "active":
      await setStatus(comment, CommentStatus.Active);
      break;
    case "ban":
      await setStatus(comment, CommentStatus.Banned);
      break;
    default:
      message.error("Unknown action.");
      break;
  }
}

async function editComment(comment: CommentInfo) {
  const canEdit = await isEditable(comment.created_at);
  const serverInfo = useServerInfo();
  if (canEdit || canManage.value)
    await router.push(
      `/publish/comment?post_id=${comment.post_id}&edit_id=${comment.id}`
    );
  else if (serverInfo.value)
    message.warning(
      `Unable to edit for ${
        serverInfo.value.editable_seconds / 60
      } minute(s) after posting`
    );
  else message.error("Cannot get server information.");
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
      else message.error(`Can't get target comment, id '${comment.id}'`);
    }

    comment.status = status;
    message.success(`Comment id '${comment.id}': ${statusText}`);
  } else {
    let err = getApiDetailError(error.value);
    message.error(`(${err?.code}) ${err?.msg}`);
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
  <div>
    <n-card
      size="small"
      v-if="comments"
      :embedded="props.parent_comment != undefined"
    >
      <n-space vertical :size="0">
        <n-space align="center">
          <n-pagination
            simple
            v-if="getTotalComment()"
            v-model:page="page"
            :page-size="limit ?? config.public.limitData.comments"
            :item-count="getTotalComment()"
            :on-update:page="changePage"
          />
          <n-button
            v-if="post.status === PostStatus.Active && !parent_comment"
            @click="goComment()"
            >Reply post</n-button
          >
        </n-space>

        <n-list v-if="comments">
          <template v-for="comment in comments.data.items">
            <n-list-item>
              <n-space vertical :size="0">
                <n-space align="center" :size="2">
                  <n-icon size="24" v-if="comment.top_index! > 0">
                    <PushpinOutlined />
                  </n-icon>
                  <FofoUserAvatar
                    :user="getUserFromExtended(comments, comment.created_by_id)!"
                    :tag="getTag(comment.created_by_id)"
                  >
                    <n-space
                      align="center"
                      v-if="comment.reply_comment_id > 0"
                      :size="2"
                      :wrap="false"
                    >
                      <n-icon :component="RightOutlined"></n-icon>
                      <FofoUserAvatar
                        :disable-avatar="true"
                        :user="getUserFromExtended(comments, comment.reply_user_id)!"
                        :tag="getTag(comment.reply_user_id)"
                      ></FofoUserAvatar>
                    </n-space>
                    <n-text code>{{ timeAgo(comment.created_at) }} </n-text>
                    <MoreButton
                      :options="getActionOptions(comment)"
                      @select="(key: string) => actionSelect(key, comment)"
                    >
                      <n-icon
                        class="n-button n-button--default-type n-button--medium-type"
                        :component="MoreOutlined"
                        :size="20"
                      />
                    </MoreButton>
                  </FofoUserAvatar>
                </n-space>
                <MarkdownViewer
                  v-if="
                    comment.status === CommentStatus.Active &&
                    comment.content_type == ContentType.Markdown
                  "
                  :content="comment.content"
                  :max_row="2"
                />
                <n-text v-else code type="warning" style="margin: 8px 0 8px 0">
                  Comment is banned!
                </n-text>
                <n-space
                  align="center"
                  v-if="comment.status === CommentStatus.Active"
                >
                  <LikeStatusComponent
                    :info="comment"
                    :flag="LikeStatusFlag.TargetComment"
                    :status="
                      getCommentLikeStatusFromExtended(comments, comment.id)
                    "
                    @statusChanged="(s: LikeStatus | null) => statusChanged(comment, s)"
                  />
                  <n-button
                    v-if="post.status === PostStatus.Active"
                    text
                    @click="goComment(comment)"
                    >Reply comment</n-button
                  >
                </n-space>
                <Comments
                  v-if="comment.parent_id == 0 && comment.total_comment > 0"
                  :post="post"
                  :parent_comment="comment"
                  :category="category"
                  :limit="Math.round((limit ?? config.public.limitData.comments) / 2)"
                  :parent_comment_user="
                    getUserFromExtended(comments, comment.created_by_id)
                  "
                ></Comments>
              </n-space>
            </n-list-item>
          </template>
        </n-list>
      </n-space>
    </n-card>
  </div>
</template>
