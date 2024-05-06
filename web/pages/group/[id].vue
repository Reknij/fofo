<script setup lang="ts">
import { getGroup } from "~/api/group";
import { isLogined, useCurrentUser } from "~/states/auth";
import { UserType } from "~/models/user";

const router = useRouter();
const config = useRuntimeConfig();
const toast = useToast();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const current = useCurrentUser();

const { data: group } = await getGroup(id);
const links = [
  {
    label: "Groups",
    to: "/groups",
  },
  {
    label: group.value?.title ?? "Unknown",
  },
];

async function goEdit() {
  if (isLogined()) await router.push(`/publish/group?edit_id=${id}`);
  else toast.add({
    description: "Please login to continue!",
  })
}

useHead({
  title: `${group.value?.title}`,
  meta: [
    {
      name: "robots",
      content: "noindex",
    },
  ],
});
</script>

<template>
  <div class="space-y-1.5">
    <FofoBreadcrumb :links="links"></FofoBreadcrumb>
    <GroupInfo v-if="group" :group="group"></GroupInfo>
    <span v-else>Group info required.</span>
    <UButton v-if="current?.user_type === UserType.Administrator" round @click="goEdit">Edit group</UButton>
  </div>
</template>
