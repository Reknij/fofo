<script setup lang="ts">
import { getUser, setUserStatus } from "~/api/user";
import {
  NSpace,
  NCard,
  NSelect,
  NTag,
  useLoadingBar,
  useMessage,
} from "naive-ui";
import { PostAlgorithmOrder } from "~/models/post";
import { UserType, UserStatus } from "~/models/user";
import { timeAgo } from "~/helper";
import { useCurrentUser } from "~/states/auth";
import type { SubPath } from "~/components/FofoBreadcrumb/model";
import { getPostsNoContent } from "~/api/post";

const route = useRoute();
const loadingBar = useLoadingBar();
loadingBar.start();
const message = useMessage();
const id = Number.parseInt(route.params.id as string);
const currentUser = useCurrentUser();
const { data: user } = await getUser(id);

const subPaths: SubPath[] = [
  {
    label: "User",
  },
  {
    label: user.value?.id.toString() ?? "Unknown",
  },
];

const statusOptions = [
  {
    label: "Active",
    value: UserStatus.Active,
  },
  {
    label: "Banned",
    value: UserStatus.Banned,
  },
  {
    label: "Only observe",
    value: UserStatus.Observer,
  },
  {
    label: "Only comment",
    value: UserStatus.OnlyComment,
  },
];

async function userStatusSelectHandle(status: UserStatus) {
  if (user.value) {
    await setUserStatus(id, {
      status: status,
    });
    user.value.status = status;
    message.success(
      `Set ${user.value?.alias}@${user.value?.username} status to ${UserStatus[status]}`
    );
  } else {
    message.error(`User(ID ${id}) not found!`);
  }
}

useHead({
  title: `${user.value?.alias}`,
  meta: [
    {
      name: "robots",
      content: "noindex",
    },
  ],
});

onMounted(() => loadingBar.finish());
</script>

<template>
  <n-space vertical v-if="user">
    <FofoBreadcrumb :subpath="subPaths"></FofoBreadcrumb>
    <n-tag :bordered="false">User info</n-tag>
    <n-card size="small">
      <n-space vertical>
        <FofoUserAvatar :user="user"></FofoUserAvatar>
        <span>{{ user.signature }}</span>
        <n-space align="center">
          <span style="font-weight: bold">Joined time:</span>
          {{ timeAgo(user.created_at) }}
        </n-space>
        <n-space align="center">
          <span style="font-weight: bold">Created posts:</span>
          {{ user.total_post }}
        </n-space>
        <n-space align="center">
          <span style="font-weight: bold">Created comments:</span>
          {{ user.total_comment }}
        </n-space>
        <n-space align="center">
          <span style="font-weight: bold">Status:</span>
          <span v-if="user.status === UserStatus.Active" style="color: green;">Active</span>
          <span v-else-if="user.status === UserStatus.Banned" style="color: red;">Banned</span>
          <span v-else-if="user.status === UserStatus.Observer" style="color: yellow;">Only view</span>
          <span v-else-if="user.status === UserStatus.OnlyComment" style="color: orange;">Only comment</span>
          <span v-else style="color: red;">Unknown</span>
        </n-space>
      </n-space>
    </n-card>

    <n-space vertical v-if="currentUser?.user_type === UserType.Administrator">
      <n-tag :bordered="false">Actions</n-tag>
      <n-card size="small">
        <n-space align="center">
          <n-select
            :consistent-menu-width="false"
            :options="statusOptions"
            :value="user.status"
            @update:value="userStatusSelectHandle"
          ></n-select>
        </n-space>
      </n-card>
    </n-space>

    <n-tag :bordered="false">Created posts</n-tag>
    <n-card size="small">
      <PostList
        hide_user
        hide_info
        :sort="PostAlgorithmOrder.Newest"
        time="lifetime"
        config.public.default.distinct
        query_pagination
        :created_by_id="user.id"
      ></PostList>
    </n-card>
  </n-space>
</template>
