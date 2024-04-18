<script setup lang="ts">
import { NSpace, NCard, NButton, useMessage, useLoadingBar } from "naive-ui";
import { getCategory } from "~/api/category";
import type { SubPath } from "~/components/FofoBreadcrumb/model";
import { PostAlgorithmOrder } from "~/models/post";
import { UserType } from "~/models/user";
import { isLogined, useCurrentUser } from "~/states/auth";

const loadingBar = useLoadingBar();
loadingBar.start();
const router = useRouter();
const config = useRuntimeConfig();
const message = useMessage();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const current = useCurrentUser();

const { data: category } = await getCategory(id);
const subpaths: SubPath[] = [
  {
    label: "Categories",
    href: "/categories",
  },
  {
    label: category.value?.title ?? "Unknown",
  },
];
async function createPost() {
  if (isLogined()) await router.push(`/publish/post?category_id=${id}`);
  else message.warning("Please login to continue!");
}
async function goEdit() {
  if (isLogined()) await router.push(`/publish/category?edit_id=${id}`);
  else message.warning("Please login to continue!");
}

useHead({
  title: `${category.value?.title}`,
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
    <CategoryInfo v-if="category" :category="category"></CategoryInfo>
    <span v-else>Category info required.</span>
    <n-card size="small">
      <n-space align="center">
        <n-button round @click="createPost">Create post</n-button>
        <n-button v-if="current?.user_type === UserType.Administrator" round @click="goEdit">Edit category</n-button>
      </n-space>
    </n-card>
    <n-card size="small">
      <PostList
        hide_category
        :sort="PostAlgorithmOrder.Newest"
        :distinct="config.public.default.distinct"
        time="lifetime"
        query_pagination
        :top_order_enable="true"
        :category_id="id"
      ></PostList>
    </n-card>
  </n-space>
</template>
