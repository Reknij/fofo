<script setup lang="ts">
import { getCategory } from "~/api/category";
import { getLikeStatus } from "~/api/like";
import { getPost } from "~/api/post";
import { getUser } from "~/api/user";
import { getApiDetailError } from "~/helper";
import { DetailErrorCode } from "~/models/detailError";
import { type LikeStatus, LikeStatusFlag } from "~/models/like";
import { PostStatus } from "~/models/post";
import { CategoryStatus } from "~/models/category";
import { UserTag } from "~/models/user";
import { useCurrentUser } from "~/states/auth";
import { usePostUser } from "~/states/post";

const router = useRouter();
const routeQuery = router.currentRoute.value.query as any as {
  comment_page?: string;
};
const showComment = ref(routeQuery.comment_page ? !isNaN(Number.parseInt(routeQuery.comment_page)) : false);
const currentUser = useCurrentUser();
const id = Number.parseInt(router.currentRoute.value.params.id as string);
const {
  data: post,
  error: postError,
  refresh: refreshPost,
} = await getPost(id, {
  full: true,
});
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

const links = [
  {
    label: category.value?.title ?? "Unknown",
    to: `/category/${category.value?.id}`,
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
  usePostUser().value = createdBy.value;

onUnmounted(() => {
  usePostUser().value = null;
});

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
    <div v-if="post && createdBy && category" class="space-y-1.5">
      <FofoBreadcrumb :links="links"></FofoBreadcrumb>
      <UAlert title="Archived" description="Category is archived. Means you can't create, update or comment post on it."
        color="yellow" v-if="category.status === CategoryStatus.Archived" />
      <UAlert title="Archived" description="Post is archived. Means you can't update or comment on it." color="yellow"
        v-if="post.status === PostStatus.Archived" />
      <UAlert title="Banned" description="Post is banned! Content invisible and you can't update or comment on it."
        color="red" v-if="post.status === PostStatus.Banned" />
      <PostInfo :category="category" :post="post" :tag="getTag()" :created_by="createdBy" :last_edit_by="lastEditBy"
        :likeStatus="likeStatus" @statusChanged="statusChanged">
      </PostInfo>
      <UCard :ui="{ body: { padding: '' } }">
        <template #header>
          <div class="flex flex-col justify-center">
            <div v-if="post.total_comment > 0" class="flex items-center gap-1.5 flex-wrap">
              <span>Post have {{ post.total_comment }} comments.</span>
              <div class="flex items-center gap-1.5">
                <UButton variant="link" @click="showComment = !showComment" :padded="false">
                  {{ showComment ? "Hide comments" : "View comments" }}
                </UButton>
                <UDivider orientation="vertical" label="OR" />
                <UButton variant="link" @click="goComment(post.id)" :padded="false" label="Reply" />
              </div>
            </div>
            <div v-else align="center" class="flex items-center">
              <span>Don't have any comments.</span>
              <UButton v-if="post.status === PostStatus.Active" variant="link" @click="goComment(post!.id)">Reply post!
              </UButton>
            </div>
          </div>
        </template>
        <Comments v-if="showComment" :post="post" :category="category" query_pagination>
        </Comments>
      </UCard>
    </div>
    <UCard size="small" v-else>
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
      <UAlert v-else title="Get target post failed" :description="`(${apiError?.code}) ${apiError?.msg}`" />
    </UCard>
  </div>
</template>
