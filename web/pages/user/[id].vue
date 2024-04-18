<script setup lang="ts">
import { getUser, setUserStatus } from "~/api/user";
import { PostAlgorithmOrder } from "~/models/post";
import { UserType, UserStatus } from "~/models/user";
import { timeAgo } from "~/helper";
import { useCurrentUser } from "~/states/auth";
import { getPostsNoContent } from "~/api/post";

const route = useRoute();
const toast = useToast();
const id = Number.parseInt(route.params.id as string);
const currentUser = useCurrentUser();
const { data: user } = await getUser(id);

const links = [
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
    toast.add({
      description: `Set ${user.value?.alias}@${user.value?.username} status to ${UserStatus[status]}`
    })
  } else {
    toast.add({
      color: 'red',
      description: `User(ID ${id}) not found!`
    })
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
</script>

<template>
  <div class="space-y-2" v-if="user">
    <FofoBreadcrumb :links="links"></FofoBreadcrumb>
    <UCard>
      <FofoUserAvatar :user="user"></FofoUserAvatar>
      <span class="code">{{ user.signature }}</span>
      <div class="flex items-center gap-x-1">
        <span style="font-weight: bold">Joined time:</span>
        <span class="code">{{ timeAgo(user.created_at) }}</span>
      </div>
      <div class="flex items-center gap-x-1">
        <span style="font-weight: bold">Total post:</span>
        <span class="code">{{ user.total_post }}</span>
      </div>
      <div class="flex items-center gap-x-1">
        <span style="font-weight: bold">Total comment:</span>
        <span class="code">{{ user.total_comment }}</span>
      </div>
      <div class="flex items-center gap-x-1">
        <span style="font-weight: bold">Status:</span>
        <span v-if="user.status === UserStatus.Active" style="color: green;">Active</span>
        <span v-else-if="user.status === UserStatus.Banned" style="color: red;">Banned</span>
        <span v-else-if="user.status === UserStatus.Observer" style="color: yellow;">Only view</span>
        <span v-else-if="user.status === UserStatus.OnlyComment" style="color: orange;">Only comment</span>
        <span v-else style="color: red;">Unknown</span>
      </div>
    </UCard>

    <PostList hide_user hide_info :sort="PostAlgorithmOrder.Newest" time="lifetime" config.public.default.distinct
      query_pagination :created_by_id="user.id"></PostList>
  </div>
</template>
