<script setup lang="ts">
import {
  NList,
  NPagination,
  NListItem,
  NTag,
  NSpace,
  NButton,
  NEllipsis,
  NGrid,
  NSelect,
  NGi,
  type SelectOption,
  NSwitch,
  NText,
  NIcon,
  NImage,
} from "naive-ui";
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
import { EyeOutlined, LeftOutlined, PushpinOutlined } from "@vicons/antd";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";

const config = useRuntimeConfig();
let props = defineProps<{
  distinct?: boolean;
  query_pagination?: boolean;
  created_by_id?: number;
  category_id?: number;
  sort: PostAlgorithmOrder;
  hide_pagination?: boolean;
  hide_category?: boolean;
  hide_user?: boolean;
  hide_info?: boolean;
  simplification?: boolean;
  limit?: number;
  max?: number;
  time?: string;
  time_num?: number;
  sort_select?: boolean;
  top_order_enable?: boolean;
}>();
let query_pagination = props.simplification
  ? false
  : props.query_pagination === true;
const sortOptions: SelectOption[] = [
  {
    label: "Hot",
    value: PostAlgorithmOrder.Hot,
  },
  {
    label: "Views",
    value: PostAlgorithmOrder.Views,
  },
  {
    label: "Likes",
    value: PostAlgorithmOrder.Likes,
  },
  {
    label: "Newest",
    value: PostAlgorithmOrder.Newest,
  },
];

const router = useRouter();

const page = ref(1);
const sortValue = ref(props.sort);
const isDistinct = ref(props.distinct);

function getHrefWithPage(page: number) {
  const p = new URL(router.currentRoute.value.fullPath, "https://example.com");
  if (props.sort !== sortValue.value)
    p.searchParams.set("sort", sortValue.value.toString());
  if (props.distinct !== isDistinct.value)
    p.searchParams.set("distinct", isDistinct.value.toString());
  p.searchParams.set("page", page.toString());
  return `${p.pathname}${p.search}`;
}
const previousHref = ref(getHrefWithPage(page.value > 1 ? page.value - 1 : 1));
const nextHref = ref(getHrefWithPage(page.value + 1));

await updateValuesFromUrl();
await updateRouteFromValues();

const query: GetPostsQuery = {
  distinct: props.distinct,
  created_by_id: props.created_by_id,
  category_id: props.category_id,
  index: page.value - 1,
  sort: props.sort,
  time: props.time ?? "week",
  time_num: props.time_num ?? 1,
  extended: true,
  limit: props.limit ?? config.public.limitData.any,
  top_order_enable: props.top_order_enable ?? false,
};
updateQueryFromValues();

let { data: posts, refresh: refreshPosts } = await getPostsNoContent(query);
const postsCountTotalOrZero = posts.value?.data.total ?? 0;
const maxPage = Math.round(
  (props.max
    ? postsCountTotalOrZero > props.max
      ? props.max
      : postsCountTotalOrZero
    : postsCountTotalOrZero) / query.limit
);

const refresh = async () => {
  await refreshPosts();
};

async function clickPost(e: Event, post_id: number) {
  e.preventDefault();
  await router.push(`/post/${post_id}`);
}

async function changePage(page_num: number, refreshCountTogether = false) {
  page.value = page_num;
  updateQueryFromValues();
  await updateRouteFromValues();
  await refresh();
}

function updateQueryFromValues() {
  if (query_pagination) {
    query.sort = sortValue.value;
    query.distinct = isDistinct.value;
    query.index = page.value - 1;
  }
}

async function updateValuesFromUrl() {
  if (query_pagination) {
    const routeQuery = router.currentRoute.value.query as any as {
      page?: string;
      sort?: string;
      distinct?: string;
    };
    let routePage = Number.parseInt(routeQuery.page ?? "");
    let routeSort = Number.parseInt(routeQuery.sort ?? "");
    let routeDistinct = routeQuery.distinct?.toLowerCase() === "true";
    page.value = routePage > 0 ? routePage : 1;
    sortValue.value = !isNaN(routeSort) ? routeSort : props.sort;
    isDistinct.value = routeDistinct;
  }
}

async function updateRouteFromValues() {
  if (query_pagination) {
    await router.replace({
      query: {
        page: page.value > 1 ? page.value : undefined,
        sort: !isNaN(sortValue.value)
          ? props.sort != sortValue.value
            ? sortValue.value
            : undefined
          : undefined,
        distinct:
          props.distinct === isDistinct.value ? undefined : isDistinct.value,
      } as any,
    });
    previousHref.value = getHrefWithPage(page.value > 1 ? page.value - 1 : 1);
    nextHref.value = getHrefWithPage(page.value + 1);
  }
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

async function goCategory(e: Event, category_id: number) {
  e.stopPropagation();
  await router.push(`/category/${category_id}`);
}

async function goCategories(e: Event) {
  e.stopPropagation();
  await router.push(`/categories`);
}

async function goUser(e: Event, user_id: number) {
  e.preventDefault();
  await router.push(`/user/${user_id}`);
}

async function changeSort(optionValue: PostAlgorithmOrder) {
  sortValue.value = optionValue;
  await changePage(1, true);
}

async function switchDistinct(value: boolean) {
  isDistinct.value = value;
  await changePage(1, true);
}

function getPostStatusColor(post: PostInfo) {
  switch (post.status) {
    default:
    case PostStatus.Active:
      return "lime";
    case PostStatus.Archived:
      return "orange";
    case PostStatus.Banned:
      return "red";
  }
}
</script>

<template>
  <div>
    <n-space vertical>
      <n-space align="center">
        <n-pagination
          simple
          v-if="!hide_pagination && posts?.data.total"
          v-model:page="page"
          :page-size="limit ?? config.public.limitData.any"
          :item-count="
            max
              ? posts.data.total > max
                ? max
                : posts.data.total
              : posts.data.total
          "
          :on-update:page="changePage"
        >
          <template #prev>
            <n-button
              :disabled="page <= 1"
              tag="a"
              :href="page <= 1 ? undefined : previousHref"
              size="small"
              circle
              @click="(e: Event) => e.preventDefault()"
            >
              <template #icon>
                <n-icon>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    xmlns:xlink="http://www.w3.org/1999/xlink"
                    viewBox="0 0 24 24"
                  >
                    <path
                      d="M15.61 7.41L14.2 6l-6 6l6 6l1.41-1.41L11.03 12l4.58-4.59z"
                      fill="currentColor"
                    ></path>
                  </svg>
                </n-icon>
              </template>
            </n-button>
          </template>
          <template #next>
            <n-button
              :disabled="page >= maxPage"
              tag="a"
              :href="page >= maxPage ? undefined : nextHref"
              size="small"
              circle
              @click="(e: Event) => e.preventDefault()"
            >
              <template #icon>
                <n-icon>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    xmlns:xlink="http://www.w3.org/1999/xlink"
                    viewBox="0 0 24 24"
                  >
                    <path
                      d="M10.02 6L8.61 7.41L13.19 12l-4.58 4.59L10.02 18l6-6l-6-6z"
                      fill="currentColor"
                    ></path>
                  </svg>
                </n-icon>
              </template>
            </n-button>
          </template>
        </n-pagination>
        <n-select
          v-if="!simplification"
          style="width: 90px"
          :value="sortValue"
          @update:value="changeSort"
          :options="sortOptions"
        ></n-select>
        <n-switch
          v-if="!simplification"
          :value="isDistinct"
          @update:value="switchDistinct"
        >
          <template #checked> Distinct </template>
          <template #unchecked> Normal </template>
        </n-switch>
      </n-space>
      <n-list v-if="posts">
        <n-list-item
          class="clickable"
          v-if="posts.data.items.length == 0"
          @click="(e: Event) => goCategories(e)"
        >
          <n-space vertical>
            <span>No articles yet, go look the categories~</span>
          </n-space>
        </n-list-item>
        <n-list-item v-for="post in posts.data.items">
          <div id="postGridContainer">
            <div class="coverSide" v-if="post.cover_url">
              <n-image
                lzay
                class="postCover"
                :src="post.cover_url"
                object-fit="cover"
              ></n-image>
            </div>
            <div
              class="mainPostSide"
              :style="{
                gridColumn: post.cover_url
                  ? 'span 3 !important'
                  : 'span 4 !important',
              }"
            >
              <n-space vertical v-if="simplification === true">
                <n-space :wrap="false">
                  <n-grid :cols="1">
                    <n-gi>
                      <n-button
                        tag="a"
                        :href="`/post/${post.id}`"
                        text
                        style="font-size: large; width: 100%"
                        @click="(e: Event) => clickPost(e, post.id)"
                      >
                        <template #icon>
                          <slot name="prefix"></slot>
                        </template>
                        <n-ellipsis style="padding: 5px 0px">
                          {{ post.title }}
                        </n-ellipsis>
                      </n-button>
                    </n-gi>
                  </n-grid>
                </n-space>
              </n-space>
              <n-space vertical v-else :wrap-item="false">
                <n-space align="center" :wrap="false" :size="2">
                  <slot name="prefix"></slot>
                  <n-icon size="24" v-if="post.top_index! > 0">
                    <PushpinOutlined />
                  </n-icon>
                  <n-grid :cols="1">
                    <n-gi>
                      <n-button
                        tag="a"
                        :href="`/post/${post.id}`"
                        text
                        style="font-size: large; width: 100%"
                        @click="(e: Event) => clickPost(e, post.id)"
                      >
                        <template #icon>
                          <slot name="prefix"></slot>
                          <span
                            style="
                              border-radius: 3px;
                              background-color: #3de993;
                              width: 10px;
                              height: 10px;
                            "
                            :style="{
                              backgroundColor: getPostStatusColor(post),
                            }"
                          ></span>
                        </template>

                        <n-ellipsis
                          style="padding: 5px 0px; text-align: center"
                        >
                          {{ post.title }}
                        </n-ellipsis>
                      </n-button>
                    </n-gi>
                  </n-grid>
                  <n-tag
                    v-if="post.total_comment > 0"
                    round
                    :bordered="false"
                    >{{ post.total_comment }}</n-tag
                  >
                </n-space>
                <n-space align="center" :wrap-item="false" :size="5">
                  <n-tag
                    v-if="!hide_category"
                    size="small"
                    class="clickable categoryTag"
                    @click="(e: Event) => goCategory(e, post.category_id)"
                  >
                    {{ getCategory(post.category_id)?.title ?? "Unknown" }}
                  </n-tag>
                  <n-text
                    class="userIdentity clickable"
                    tag="a"
                    :href="`/user/${post.created_by_id}`"
                    @click="(e: Event) => goUser(e, post.created_by_id)"
                    :depth="3"
                    >{{ getUser(post.created_by_id)?.alias ?? "Unknown" }}
                  </n-text>
                  <n-text code>{{ timeAgo(post.last_edit_at) }}</n-text>
                  <n-space
                    v-if="post.last_comment_by_id != 0"
                    :wrap-item="false"
                    :size="5"
                    align="center"
                  >
                    <n-icon :component="LeftOutlined"></n-icon>
                    <n-text
                      class="userIdentity clickable"
                      :depth="3"
                      @click="(e: Event) => goUser(e, post.last_comment_by_id)"
                      >{{
                        getUser(post.last_comment_by_id)?.alias ?? "Unknown"
                      }}
                    </n-text>
                    <n-text code>{{ timeAgo(post.last_comment_at) }}</n-text>
                  </n-space>
                </n-space>
                <n-space align="center" class="postStatus" v-if="!hide_info">
                  <LikeStatusComponent
                    :info="post"
                    :flag="LikeStatusFlag.TargetPost"
                    :status="getPostLikeStatusFromExtended(posts, post.id)"
                    @statusChanged="(s: LikeStatus | null) => statusChanged(post, s)"
                  />
                  <n-space :size="4" align="center">
                    <n-icon :size="20" :component="EyeOutlined"></n-icon>
                    {{ post.views }}
                  </n-space>
                </n-space>
              </n-space>
            </div>
          </div>
        </n-list-item>
      </n-list>
    </n-space>
  </div>
</template>

<style>
#postGridContainer {
  display: grid;
  grid-template-columns: repeat(4, minmax(0px, 1fr));
  gap: 6px;
}

@media only screen and (max-width: 1023px) {
  .coverSide {
    grid-column: span 1 !important;
  }

  .mainPostSide {
    grid-column: span 3 !important;
  }

  .postStatus {
    display: none !important;
  }
}

@media only screen and (min-width: 1024px) {
  .coverSide {
    grid-column: span 1 !important;
  }

  .mainPostSide {
    grid-column: span 3 !important;
  }
}

.postCover > img {
  max-width: 100%;
  max-height: 200px;
}

.userIdentity {
  text-decoration: none;
}

.userIdentity:hover {
  text-decoration: underline;
  text-decoration-color: black;
}
</style>
