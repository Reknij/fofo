<script setup lang="ts">
import { NSpace, NCard, NButton, useMessage, useLoadingBar } from "naive-ui";
import { getGroup } from "~/api/group";
import type { SubPath } from "~/components/FofoBreadcrumb/model";
import { isLogined, useCurrentUser } from "~/states/auth";
import { UserType } from "~/models/user";

const loadingBar = useLoadingBar();
loadingBar.start();
const router = useRouter();
const config = useRuntimeConfig();
const message = useMessage();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const current = useCurrentUser();

const { data: group } = await getGroup(id);
const subpaths: SubPath[] = [
  {
    label: "Groups",
    href: "/groups",
  },
  {
    label: group.value?.title ?? "Unknown",
  },
];

async function goEdit() {
  if (isLogined()) await router.push(`/publish/group?edit_id=${id}`);
  else message.warning("Please login to continue!");
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

onMounted(() => loadingBar.finish());
</script>

<template>
  <n-space vertical>
    <FofoBreadcrumb :subpath="subpaths"></FofoBreadcrumb>
    <GroupInfo v-if="group" :group="group"></GroupInfo>
    <span v-else>Group info required.</span>
    <n-card size="small" v-if="current?.user_type === UserType.Administrator">
      <n-space align="center">
        <n-button round @click="goEdit">Edit group</n-button>
      </n-space>
    </n-card>
  </n-space>
</template>
