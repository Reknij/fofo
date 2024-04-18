<script setup lang="ts">
import { ArrowLeftOutlined } from "@vicons/antd";
import {
  NSpace,
  NCard,
  NTag,
  NButton,
  NIcon,
  NAlert,
  useLoadingBar,
} from "naive-ui";
import { getCategory } from "~/api/category";
import { getLikeStatus } from "~/api/like";
import { getPost } from "~/api/post";
import { getUser } from "~/api/user";
import type { SubPath } from "~/components/FofoBreadcrumb/model";
import { getApiDetailError } from "~/helper";
import { DetailErrorCode } from "~/models/detailError";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import { PostStatus } from "~/models/post";
import { CategoryStatus } from "~/models/category";
import { UserTag } from "~/models/user";
import { useCurrentUser } from "~/states/auth";
import { usePostUserAndTag } from "~/states/post";

const router = useRouter();

const loadingBar = useLoadingBar();
loadingBar.start();

const showComment = ref(false);
const currentUser = useCurrentUser();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const {
  data: post,
  error: postError,
  refresh: refreshPost,
} = await getPost(id);
const r = await Promise.all([
  getCategory(post.value?.category_id ?? 0),
  getUser(post.value?.created_by_id ?? 0),
  getUser(post.value?.last_edit_by_id ?? 0),
  getLikeStatus({
    flag_ref_id: post.value?.id ?? 0,
    flag: LikeStatusFlag.TargetPost,
    created_by_id: currentUser.value?.id ?? 0,
  }),
]);

const { data: category, error: categoryError } = r[0];
const { data: createdBy } = r[1];
const { data: lastEditBy } = r[2];
const { data: likeStatus } = r[3];

const apiError = ref(getApiDetailError(postError.value ?? categoryError.value));
console.log(apiError.value);
provide("reload", async () => {
  await refreshPost();
  apiError.value = getApiDetailError(postError.value ?? categoryError.value);
});

const subPaths: SubPath[] = [
  {
    label: category.value?.title ?? "Unknown",
    href: `/category/${category.value?.id}`,
  },
  {
    label: post.value?.id.toString() ?? "Unknown",
  },
];

function getTag() {
  let tags = UserTag.OP;
  const m = category.value?.moderator_ids.find((m) => m == createdBy.value?.id);
  if (m) {
    tags = tags | UserTag.Moderator;
  }

  return tags;
}

function statusChanged(newStatus: LikeStatus | null) {
  const previous = likeStatus.value;
  if (post.value) {
    const p = post.value;
    if (previous) {
      previous.is_like ? (p.likes -= 1) : (p.dislikes -= 1);
    }
    if (newStatus) {
      newStatus.is_like ? (p.likes += 1) : (p.dislikes += 1);
    }
  }
  likeStatus.value = newStatus;
}

if (createdBy.value)
  usePostUserAndTag().value = {
    user: createdBy.value,
    tag: getTag(),
  };

onUnmounted(() => {
  usePostUserAndTag().value = null;
});

onMounted(() => loadingBar.finish());

async function handleBack() {
  router.back();
}

async function goComment(post_id: number) {
  let go = `/publish/comment?post_id=${post_id}`;
  await router.push(go);
}

useHead({
  title: post.value?.title,
  meta: [
    { name: "keywords", content: post.value?.tags.join(", ") },
    { name: "robots", content: "max-image-preview:large" },
    {
      name: "author",
      content: `${createdBy.value?.alias}@${createdBy.value?.username}`,
    },
  ],
});
let origin = useRequestURL().origin;
let created_at = new Date((post.value?.created_at ?? 0) * 1000);
let last_edit_at = new Date((post.value?.last_edit_at ?? 0) * 1000);
useJsonld({
  "@context": "https://schema.org",
  "@type": "NewsArticle",
  headline: post.value?.title,
  image: post.value?.cover_url ? [post.value.cover_url] : [],
  datePublished: created_at.toISOString(),
  dateModified: last_edit_at.toISOString(),
  author: [
    {
      "@type": "Person",
      name: createdBy.value?.alias,
      url: createdBy.value ? `${origin}/user/${createdBy.value.id}` : undefined,
    },
  ],
});
</script>

<template>
  <div>
    <n-space vertical>
      <n-space vertical v-if="post && createdBy && category">
        <FofoBreadcrumb :subpath="subPaths"></FofoBreadcrumb>
        <n-alert
          title="Note"
          type="warning"
          v-if="category.status === CategoryStatus.Archived"
        >
          Category is archived. Means you can't create, update or comment post
          on it.
        </n-alert>
        <n-alert
          title="Note"
          type="warning"
          v-if="post.status === PostStatus.Archived"
        >
          Post is archived. Means you can't update or comment on it.
        </n-alert>
        <n-alert
          title="Note"
          type="error"
          v-if="post.status === PostStatus.Banned"
        >
          Post is banned! Content invisible and you can't update or comment on
          it.
        </n-alert>
        <PostInfo
          :category="category"
          :post="post"
          :tag="getTag()"
          :created_by="createdBy"
          :last_edit_by="lastEditBy"
          :likeStatus="likeStatus"
          @statusChanged="statusChanged"
        >
        </PostInfo>
        <n-tag :bordered="false">Comments</n-tag>
        <LazyClientOnly>
          <n-card size="small">
            <n-space v-if="post.total_comment > 0" align="center">
              Post have {{ post.total_comment }} comments.
              <n-button text type="info" @click="showComment = true">
                Click me to view!
              </n-button>
            </n-space>
            <n-space v-else align="center">
              Don't have any comments.
              <n-button
                v-if="post.status === PostStatus.Active"
                text
                type="info"
                @click="goComment(post!.id)"
                >Click me to reply post!</n-button
              >
            </n-space>
            <Comments
              v-if="showComment"
              :post="post"
              :category="category"
              query_pagination
            >
            </Comments>
          </n-card>
        </LazyClientOnly>
      </n-space>
      <n-space vertical v-else>
        <n-card size="small">
          <h4 v-if="apiError?.code == DetailErrorCode.NoPermission">
            You no permission to read the post.
          </h4>
          <h4 v-else-if="apiError?.code == DetailErrorCode.CategoryStopped">
            Category is stopped!
          </h4>
          <h4 v-else-if="apiError?.code == DetailErrorCode.BannedStatus">
            Post is banned!
          </h4>
          <h4 v-else-if="apiError?.code == DetailErrorCode.PostNotFound">
            Post not found...
          </h4>
          <n-space vertical v-else>
            <span>Get target post failed...</span>
            <span>({{ apiError?.code }}) {{ apiError?.msg }}</span>
          </n-space>
          <n-button @click="handleBack">
            <template #icon>
              <n-icon :component="ArrowLeftOutlined" />
            </template>
            Go back
          </n-button>
        </n-card>
      </n-space>
    </n-space>
  </div>
</template>
