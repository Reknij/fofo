<script setup lang="ts">
import { setPostStatus } from "~/api/post";
import { ContentType } from "~/models/util";
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
import type { Category } from "~/models/category";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import type { DropdownItem } from '#ui/types'
import PostAndCommentInfo from "./PostAndCommentInfoModal.vue";

const router = useRouter();
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
const toast = useToast();
const modal = useModal();

const actionOptions: DropdownItem[][] = [
  [{
    label: "Info",
    click() {
      modal.open(PostAndCommentInfo, {
        info: props.post
      })
    }
  }],
];
if (canManage.value) {
  actionOptions.push([{
    label: "Edit",
    click() {
      editPost(props.post);
    }
  }])
}
if (canManage) {
  actionOptions.push([{
    label: "Reactive",
    click() {
      setStatus(props.post.id, PostStatus.Active)
    },
  },
  {
    label: "Archive",
    click() {
      setStatus(props.post.id, PostStatus.Archived)
    },
  },
  {
    label: "Ban",
    click() {
      setStatus(props.post.id, PostStatus.Banned)
    },
  }
  ])
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
    toast.add({
      color: 'yellow',
      description: `Unable to edit for ${serverInfo.value.editable_seconds / 60
        } minute(s) after posting`
    })
  else {
    toast.add({
      color: 'red',
      description: "Cannot get server information."
    })
  }
}

async function setStatus(id: number, status: PostStatus) {
  const { error } = await setPostStatus(id, {
    status,
  });
  if (!error.value) {
    let statusText = Object.values(PostStatus)[status];
    toast.add({
      description: `Post id '${id}': ${statusText}`
    })
    if (reload) await reload();
    else {
      toast.add({
        color: 'red',
        description: "Can't reload the page."
      })
    }
  } else {
    const err = getApiDetailError(error.value);
    toast.add({
      color: 'red',
      description: `(${err?.code}) ${err?.msg}`
    })
  }
}

async function goUser(id: number) {
  await router.push(`/user/${id}`);
}
</script>

<template>
  <div>
    <UCard :ui="{ body: { padding: '' } }">
      <template #header>
        <div class="space-y-0">
          <div v-if="created_by" class="flex justify-between items-center">
            <FofoUserAvatar :user="created_by" :tag="tag" />
            <UDropdown :items="actionOptions">
              <UButton icon="i-heroicons-ellipsis-vertical" variant="ghost" />
            </UDropdown>
          </div>
          <span v-else>Not found user.</span>
          <div class="text-2xl">{{ post.title }}</div>
          <span class="mx-1 code text-xs" v-for="tag in post.tags">#{{ tag }}</span>

          <div class="flex flex-wrap gap-x-1">
            <div class="flex items-center gap-x-2 flex-wrap">
              <div class="flex items-center gap-x-1">
                <UIcon name="i-heroicons-clock" />
                {{ timeAgo(post.created_at) }}
              </div>
              <div class="flex items-center gap-x-1">
                <UIcon name="i-heroicons-pencil-square" />
                {{ timeAgo(post.last_edit_at) }} -
                <span v-if="last_edit_by" class="userIdentity clickable" @click="goUser(last_edit_by!.id)">{{
                  last_edit_by.alias }}</span>
              </div>
            </div>
            <div class="flex items-center gap-x-1">
              <LikeStatusComponent :info="post" :status="likeStatus" :flag="LikeStatusFlag.TargetPost"
                @statusChanged="(v: LikeStatus | null) => emit('statusChanged', v)" />
              <UIcon name="i-heroicons-eye" />
              <span class="text-sm font-medium">{{ post.views }}</span>
            </div>
          </div>
        </div>
      </template>
      <MarkdownViewer class="post-info" v-if="post.content && post.content_type == ContentType.Markdown"
        :content="post.content" />
    </UCard>
  </div>
</template>

<style scoped>
:deep(.markdown-body) {
  @apply rounded-b-lg;
}

.userIdentity {
  text-decoration: none;
}

.userIdentity:hover {
  text-decoration: underline;
  text-decoration-color: black;
}
</style>
