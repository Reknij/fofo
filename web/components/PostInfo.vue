<script setup lang="ts">
import { setPostStatus } from "~/api/post";
import { ContentType } from "~/models/util";
import {
  NCard,
  NText,
  NSpace,
  NH2,
  NIcon,
  useMessage,
  useDialog,
} from "naive-ui";
import {
  getApiDetailError,
  getServerInfoOnce,
  hasManagePermission,
  isEditable,
  timeAgo,
} from "~/helper";
import type { SafeUserInfo, UserTag } from "~/models/user";
import { type PostInfo, PostStatus } from "~/models/post";
import { useCurrentUser } from "~/states/auth";
import { EyeOutlined, ClockCircleOutlined, EditOutlined } from "@vicons/antd";
import "bytemd/dist/index.css";
import type { Category } from "~/models/category";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import type { MenuMixedOption } from "naive-ui/es/menu/src/interface";

const router = useRouter();
const message = useMessage();
const dialog = useDialog();
const props = defineProps<{
  post: PostInfo;
  category: Category;
  created_by: SafeUserInfo;
  last_edit_by?: SafeUserInfo | null;
  likeStatus: LikeStatus | null;
  tag?: UserTag;
}>();
const currentUser = useCurrentUser();
const canManage = ref(hasManagePermission(currentUser.value, props.category));
const actionOptions: MenuMixedOption[] = [
  {
    label: "Info",
    key: "info",
  },
  {
    label: "Edit",
    key: "edit",
    show: currentUser.value?.id === props.post.created_by_id || canManage.value,
  },
  {
    label: "Reactive",
    key: "active",
    show: canManage.value,
  },
  {
    label: "Archive",
    key: "archive",
    show: canManage.value,
  },
  {
    label: "Ban",
    key: "ban",
    show: canManage.value,
  },
];
async function moreSelect(key: string) {
  switch (key) {
    case "info":
      const createdBy = `${props.created_by.alias}@${props.created_by.username}`;
      const lastEditBy = props.last_edit_by
        ? `${props.last_edit_by?.alias}@${props.last_edit_by?.username}`
        : createdBy;
      dialog.info({
        title: "Info",
        content: () =>
          h(NSpace, { vertical: true, justify: "center" }, [
            h("span", `Post title: ${props.post.title}`),
            h("span", `Post likes: ${props.post.likes}`),
            h("span", `Post dislikes: ${props.post.dislikes}`),
            h(
              "span",
              `Total comment to post: ${props.post.total_comment_post}`
            ),
            h("span", `Total comment of post: ${props.post.total_comment}`),
            h(
              "span",
              `Created at: ${new Date(
                props.post.created_at * 1000
              ).toDateString()}`
            ),
            h("span", `Created by: ${createdBy}`),
            h(
              "span",
              `Last edit at: ${new Date(
                props.post.last_edit_at * 1000
              ).toDateString()}`
            ),
            h("span", `Last edit by: ${lastEditBy}`),
            h(
              "span",
              `Post content type: ${
                Object.values(ContentType)[props.post.content_type]
              }`
            ),
            h(
              "span",
              `Post status: ${Object.values(PostStatus)[props.post.status]}`
            ),
            h("span", `Top index: ${props.post.top_index ?? 0}`),
          ]),
      });
      break;
    case "edit":
      await editPost(props.post);
      break;
    case "active":
      await setStatus(props.post.id, PostStatus.Active);
      break;
    case "archive":
      await setStatus(props.post.id, PostStatus.Archived);
      break;
    case "ban":
      await setStatus(props.post.id, PostStatus.Banned);
      break;
    default:
      message.error("Unknown action.");
      break;
  }
}

const reload = inject<() => Promise<void>>("reload");
const emit = defineEmits<{
  (e: "statusChanged", status: LikeStatus | null): void;
}>();

async function editPost(post: PostInfo) {
  const hasPermission = hasManagePermission(currentUser.value, props.category);
  const canEdit = hasPermission ? true : await isEditable(post.created_at);
  const serverInfo = await getServerInfoOnce();
  if (canEdit) await router.push(`/publish/post?edit_id=${post.id}`);
  else if (serverInfo.value)
    message.warning(
      `Unable to edit for ${
        serverInfo.value.editable_seconds / 60
      } minute(s) after posting`
    );
  else message.error("Cannot get server information.");
}

async function setStatus(id: number, status: PostStatus) {
  const { error } = await setPostStatus(id, {
    status,
  });
  if (!error.value) {
    let statusText = Object.values(PostStatus)[status];
    message.success(`Post id '${id}': ${statusText}`);
    if (reload) await reload();
    else message.warning("Can't reload page.");
  } else {
    const err = getApiDetailError(error.value);
    message.error(`(${err?.code}) ${err?.msg}`);
  }
}

function getHeaderType(post: PostInfo) {
  switch (post.status) {
    default:
    case PostStatus.Active: {
      return "default";
    }
    case PostStatus.Archived: {
      return "warning";
    }
    case PostStatus.Banned: {
      return "error";
    }
  }
}

async function goUser(id: number) {
  await router.push(`/user/${id}`);
}
</script>

<template>
  <div>
    <n-space vertical>
      <n-card size="small">
        <n-space vertical size="small">
          <FofoUserAvatar v-if="created_by" :user="created_by" :tag="tag">
            <MoreButton
              :options="actionOptions"
              @select="moreSelect"
            ></MoreButton>
          </FofoUserAvatar>
          <span v-else>Not found user.</span>

          <n-h2 prefix="bar" :type="getHeaderType(post)" style="margin: 0">{{
            post.title
          }}</n-h2>

          <n-space align="center" v-if="post.tags.length > 0">
            <n-text v-for="tag in post.tags" type="success">
              #{{ tag }}
            </n-text>
          </n-space>
          <n-space align="center">
            <n-space :size="4" align="center">
              <n-icon :size="20" :component="ClockCircleOutlined"></n-icon>
              {{ timeAgo(post.created_at) }}
            </n-space>
            <n-space :size="4" align="center">
              <n-icon :size="20" :component="EditOutlined"></n-icon>
              {{ timeAgo(post.last_edit_at) }} -
              <span
                v-if="last_edit_by"
                class="userIdentity clickable"
                @click="goUser(last_edit_by!.id)"
                >{{ last_edit_by.alias }}</span
              >
            </n-space>
          </n-space>
          <n-space align="center">
            <LikeStatusComponent
              :info="post"
              :status="likeStatus"
              :flag="LikeStatusFlag.TargetPost"
              @statusChanged="(v: LikeStatus | null) => emit('statusChanged', v)"
            />
            <n-space :size="4" align="center">
              <n-icon :size="20" :component="EyeOutlined"></n-icon>
              {{ post.views }}
            </n-space>
          </n-space>
        </n-space>
      </n-card>
      <n-card size="small" v-if="post.content">
        <MarkdownViewer
          v-if="post.content_type == ContentType.Markdown"
          :content="post.content"
        />
      </n-card>
    </n-space>
  </div>
</template>

<style>
.userIdentity {
  text-decoration: none;
}

.userIdentity:hover {
  text-decoration: underline;
  text-decoration-color: black;
}
</style>
