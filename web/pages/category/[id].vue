<script setup lang="ts">
import { getCategory } from "~/api/category";
import { PostAlgorithmOrder } from "~/models/post";
import { UserType } from "~/models/user";
import { isLogined, useCurrentUser } from "~/states/auth";

const router = useRouter();
const config = useRuntimeConfig();
const toast = useToast();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const current = useCurrentUser();

const { data: category } = await getCategory(id);
const links = [
  {
    label: "Categories",
    to: "/categories",
  },
  {
    label: category.value?.title ?? "Unknown",
  },
];
async function createPost() {
  if (isLogined()) await router.push(`/publish/post?category_id=${id}`);
  else toast.add({
    color: 'yellow',
    description: "Please login to continue!"
  })
}
async function goEdit() {
  if (isLogined()) await router.push(`/publish/category?edit_id=${id}`);
  else toast.add({
    color: 'yellow',
    description: "Please login to continue!"
  })
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
</script>

<template>
  <div class="space-y-1.5">
    <FofoBreadcrumb :links="links"></FofoBreadcrumb>
    <CategoryInfo v-if="category" :category="category"></CategoryInfo>
    <span v-else>Category info required.</span>
    <div class="flex gap-1.5">
      <UButton variant="soft" round @click="createPost">Create post</UButton>
      <UButton variant="soft" v-if="current?.user_type === UserType.Administrator" round @click="goEdit">Edit category
      </UButton>
    </div>
    <UAlert title="This week's post." color="primary" variant="subtle" />
    <PostList hide_category :category_id="id" :sort="PostAlgorithmOrder.Newest"
      :distinct="config.public.default.distinct" :limit="20" time="week" :time_num="1" :top_order_enable="true"
      disable_query></PostList>
    <UAlert title="Hot post." color="primary" variant="subtle" />
    <PostList hide_category :category_id="id" :top_order_enable="false" :sort="PostAlgorithmOrder.Hot" :distinct="config.public.default.distinct"
      :limit="10"></PostList>
  </div>
</template>
