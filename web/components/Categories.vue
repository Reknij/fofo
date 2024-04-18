<script setup lang="ts">
import {
  NList,
  NSpace,
  NListItem,
  NGrid,
  NGi,
  NEllipsis,
  NTag,
  NPagination,
  NButton,
  NIcon,
} from "naive-ui";
import { getCategories } from "~/api/category";
import {
  GetCategoriesSort,
  type GetCategoriesQuery,
  type Category,
} from "~/models/category";

const config = useRuntimeConfig();
const router = useRouter();
const props = defineProps<{
  hide_pagination?: boolean;
  query_pagination?: boolean;
  max?: number;
  limit?: number;
}>();

const query_pagination = props.query_pagination ?? false;
await updateValuesFromUrl();
await updateRouteFromValues();
const query: GetCategoriesQuery = {
  index: 0,
  limit: props.limit ?? config.public.limitData.any,
  desc: true,
  sort: GetCategoriesSort.TotalPost,
  extended: true,
};

updateQueryFromValues();
let { data: categories, refresh } = await getCategories(query);
let page = ref(1);
const maxPage = Math.round((categories.value?.data.total ?? 0) / query.limit);

function getHrefWithPage(page: number) {
  const p = new URL(router.currentRoute.value.fullPath, "https://example.com");
  p.searchParams.set("page", page.toString());
  return `${p.pathname}${p.search}`;
}
const previousHref = ref(getHrefWithPage(page.value > 1 ? page.value - 1 : 1));
const nextHref = ref(getHrefWithPage(page.value + 1));

async function clickCategory(e: Event, category: Category) {
  e.preventDefault();
  await router.push(`/category/${category.id}`);
}

async function changePage(page_num: number) {
  query.index = page_num - 1;
  page.value = page_num;
  updateQueryFromValues();
  await updateRouteFromValues();
  await refresh();
}

function updateQueryFromValues() {
  if (query_pagination) {
    query.index = page.value - 1;
  }
}

async function updateValuesFromUrl() {
  if (query_pagination) {
    const routeQuery = router.currentRoute.value.query as any as {
      page?: string;
    };
    let routePage = Number.parseInt(routeQuery.page ?? "");

    page.value = routePage > 0 ? routePage : 1;
  }
}

async function updateRouteFromValues() {
  if (query_pagination) {
    await router.replace({
      query: {
        page: page.value > 1 ? page.value : undefined,
      } as any,
    });
    previousHref.value = getHrefWithPage(page.value > 1 ? page.value - 1 : 1);
    nextHref.value = getHrefWithPage(page.value + 1);
  }
}
</script>

<template>
  <n-space vertical>
    <n-pagination
      simple
      v-if="!hide_pagination && categories?.data.total"
      v-model:page="page"
      :page-size="limit ?? config.public.limitData.any"
      :item-count="max ?? categories.data.total"
      :on-update:page="changePage"
    >
      <template #prev>
        <n-button
          :disabled="page <= 1"
          tag="a"
          :href="previousHref"
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
          :href="nextHref"
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
    <n-list>
      <n-list-item v-if="categories" v-for="category in categories.data.items">
        <n-space vertical>
          <n-space align="center" :wrap="false">
            <slot name="prefix"></slot>
            <n-grid :cols="1">
              <n-gi>
                <n-ellipsis>
                  <n-button
                    tag="a"
                    :href="`/category/${category.id}`"
                    text
                    style="font-size: large; width: 100%"
                    @click="(e: Event) => clickCategory(e, category)"
                  >
                    <template #icon>
                      <slot name="prefix"></slot>
                    </template>
                    {{ category.title }}
                  </n-button>
                </n-ellipsis>
              </n-gi>
            </n-grid>
            <n-tag>{{ category.total_post }}</n-tag>
          </n-space>
        </n-space>
      </n-list-item>
      <n-list-item v-else>
        <n-space vertical>
          <span>No categories yet~</span>
        </n-space>
      </n-list-item>
    </n-list>
  </n-space>
</template>
