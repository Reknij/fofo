<script setup lang="ts">
import { getPostsNoContent } from "~/api/post";
import { timeAgo } from "~/helper";
import {
  PostAlgorithmOrder,
  type PostInfo,
  type GetPostsQuery,
  PostStatus,
} from "~/models/post";
import {
  getCategoryFromExtended,
  getPostLikeStatusFromExtended,
  getUserFromExtended,
} from "~/models/util";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import { useCurrentUser } from "~/states/auth";

const config = useRuntimeConfig();
let props = defineProps<{
  disable_query?: boolean;
  distinct?: boolean;
  created_by_id?: number;
  category_id?: number;
  sort: PostAlgorithmOrder;
  hide_category?: boolean;
  hide_user?: boolean;
  limit?: number;
  time?: string;
  time_num?: number;
  sort_select?: boolean;
  top_order_enable?: boolean;
}>();
const sortOptions: any[] = [
  PostAlgorithmOrder.Hot,
  PostAlgorithmOrder.Views,
  PostAlgorithmOrder.Likes,
  PostAlgorithmOrder.Newest,
];
const getSortLabel = (sort: PostAlgorithmOrder) => {
  switch (sort) {
    case PostAlgorithmOrder.Hot:
      return "Hot"
    case PostAlgorithmOrder.Likes:
      return "Likes"
    case PostAlgorithmOrder.Newest:
      return "Newest"
    case PostAlgorithmOrder.Views:
      return "Views"
    default:
      break;
  }
}

const currentUser = useCurrentUser();
const router = useRouter();
const query: GetPostsQuery = reactive({
  distinct: props.distinct,
  created_by_id: props.created_by_id,
  category_id: props.category_id,
  index: 0,
  sort: props.sort,
  time: props.time,
  time_num: props.time_num,
  extended: true,
  limit: props.limit ?? config.public.limitData.any,
  top_order_enable: props.top_order_enable ?? false,
});

function changeRouteQuery() {
  const q = {
    page: query.index + 1,
    sort: query.sort.toString(),
    distinct: query.distinct,
  }
  router.replace({
    query: q as any
  })
}

function getHrefWithPage(page: number) {
  const p = new URL(router.currentRoute.value.fullPath, "https://example.com");
  if (props.sort !== query.sort)
    p.searchParams.set("sort", query.sort.toString());
  if (query.distinct) p.searchParams.set("distinct", query.distinct.toString());
  else p.searchParams.delete('distinct')
  p.searchParams.set("page", page.toString());
  return `${p.pathname}${p.search}`;
}

if (!props.disable_query) {
  watch(query, changeRouteQuery);
  await updateQueryFromUrl()
}

let { data: posts, refresh: refreshPosts } = await getPostsNoContent(query);
async function updateQueryFromUrl() {
  const routeQuery = router.currentRoute.value.query as any as {
    page?: string;
    sort?: string;
    distinct?: string;
  };
  let routePage = Number.parseInt(routeQuery.page ?? "");
  let routeSort = Number.parseInt(routeQuery.sort ?? "");
  let routeDistinct = routeQuery.distinct?.toLowerCase() === "true";
  query.index = routePage > 0 ? routePage - 1 : 0;
  query.sort = !isNaN(routeSort) ? routeSort : props.sort;
  query.distinct = routeDistinct;
}

async function statusChanged(post: PostInfo, status: LikeStatus | null) {
  if (posts.value) {
    if (!posts.value.posts_like_status) {
      posts.value.posts_like_status = {};
    }
    const previous = posts.value.posts_like_status[post.id];
    if (previous) {
      previous.is_like ? (post.likes -= 1) : (post.dislikes -= 1);
    }
    if (status) {
      status.is_like ? (post.likes += 1) : (post.dislikes += 1);
    }
    posts.value.posts_like_status[post.id] = status;
  }
}

function getUser(id?: number) {
  return posts.value && id ? getUserFromExtended(posts.value, id) : undefined;
}

function getCategory(id: number) {
  return posts.value ? getCategoryFromExtended(posts.value, id) : undefined;
}
</script>

<template>
  <div class="space-y-2">
    <UPagination v-if="posts" :model-value="query.index + 1" @update:model-value="(v: number) => query.index = v - 1"
      :page-count="query.limit" :total="posts?.data.total ?? 0">
      <template #prev="{ onClick }">
        <UButton :to="disable_query? undefined: getHrefWithPage(query.index)" icon="i-heroicons-arrow-small-left-20-solid" class="rounded-r-none"
          square color="white" @click="onClick" />
      </template>
      <template #next="{ onClick }">
        <UButton :to="disable_query? undefined: getHrefWithPage(query.index + 2)" icon="i-heroicons-arrow-small-right-20-solid"
          class="rounded-l-none" square color="white" @click="onClick" />
      </template>
    </UPagination>
    <div class="flex gap-x-1 items-center">
      <USelectMenu v-model="query.sort" @update:model-value="query.index = 0" :options="sortOptions">
        <template #label>
          <span class="truncate">{{ getSortLabel(query.sort) }}</span>
        </template>
        <template #option="{ option: sort }">
          <span class="truncate">{{ getSortLabel(sort) }}</span>
        </template>
      </USelectMenu>
      <UBadge class="gap-x-2" variant="soft">
        Distinct
        <UToggle @update:model-value="query.index = 0" v-model="query.distinct" />
      </UBadge>
    </div>
    <UCard v-if="posts" v-for="post in posts.data.items">
      <div class="flex flex-col justify-between gap-x-2 gap-y-1">
        <div class="flex items-center justify-between gap-x-2 gap-y-1 flex-wrap flex-grow">
          <div class="flex flex-col gap-x-2">
            <div class="flex gap-x-1 items-center ">
              <UIcon v-if="post.top_index" name="i-ph-push-pin" dynamic />
              <UIcon v-if="post.status === PostStatus.Active" name="i-heroicons-chevron-double-right" />
              <UIcon v-else-if="post.status === PostStatus.Banned" name="i-heroicons-lock-closed"
                class="text-red-500" />
              <UIcon v-else-if="post.status === PostStatus.Archived" name="i-heroicons-lock-closed"
                class="text-yellow-500" />
              <ULink class="line-clamp-1" :to="`/post/${post.id}`" active-class="text-primary"
                inactive-class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
                {{ post.title }}
              </ULink>
            </div>
            <div class="flex items-center gap-x-1 flex-wrap">
              <ULink v-if="!hide_category" class="code code-button text-sm" :to="`/category/${post.category_id}`">{{
                getCategory(post.category_id)?.title ??
                'Unknown' }}
              </ULink>
              <ULink v-if="!hide_user" class="code code-button text-xs" :to="`/user/${post.created_by_id}`">{{
                getUser(post.created_by_id)?.alias ??
                'Unknown'
              }}</ULink>
              <span class="text-xs code">{{ timeAgo(post.created_at) }}</span>
              <LikeStatusComponent :info="post"
                :status="getPostLikeStatusFromExtended(posts!, post.id)"
                :flag="LikeStatusFlag.TargetPost" @statusChanged="(v: LikeStatus | null) => statusChanged(post, v)" />
              <UIcon name="i-heroicons-eye" />
              <span class="text-sm font-medium">{{ post.views }}</span>
            </div>
          </div>
          <UBadge v-if="post.total_comment > 0" color="gray">
            <div class="flex items-center justify-center gap-x-1">
              <UIcon name="i-heroicons-chat-bubble-left-ellipsis size-4" />
              {{ post.total_comment }}
            </div>
          </UBadge>
        </div>
        <div v-if="post.cover_url" class="flex items-center justify-center xl:justify-normal">
          <img class="rounded-lg max-h-[150px] xl:max-h-[200px] bg-cover shadow-md" :src="post.cover_url" />
        </div>
      </div>
    </UCard>
  </div>
</template>