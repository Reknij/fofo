<script setup lang="ts">
import { PostStatus, PostAlgorithmOrder } from "~/models/post";
import { useCurrentUser, AUTH_COOKIE_NAME, logout, revertInsideNuxt } from "~/states/auth";
import { usePostUser } from "~/states/post";
import { getServerInfo } from "~/api/server";
import { getPostsNoContent } from "~/api/post";
import { getUserNotifications } from "~/api/notification";

const currentUser = useCurrentUser();
const postUser = usePostUser();
const isPostUser = computed(
  () => currentRoute.value.path.startsWith("/post/") && postUser.value
);
const router = useRouter();
const currentRoute = router.currentRoute;
const toast = useToast();
const { data: serverInfo } = await getServerInfo();

const notificationQuery = {
  index: 0,
  limit: 0,
  extended: false,
  only_unread: true,
};

const totalMessageText = ref('0')

const refreshNotificationsCount = async () => {
  if (!useCookie(AUTH_COOKIE_NAME).value) return;

  const { data: notifications } = await getUserNotifications(notificationQuery);

  if (notifications.value) {
    if (notifications.value.data.total > 99) totalMessageText.value = '99+';
    else totalMessageText.value = notifications.value.data.total.toString();
  } else {
    totalMessageText.value = '??'
  }
}
await refreshNotificationsCount();

watch(router.currentRoute, async () => {
  await refreshNotificationsCount();
});

const { data: posts } = await getPostsNoContent({
  index: 0,
  limit: 10,
  sort: PostAlgorithmOrder.Hot,
  time: "day",
  time_num: 1,
  top_order_enable: false,
  distinct: true,
  extended: false,
});
const topPostsTitle = computed(() => {
  if (posts.value?.data.total) {
    return "24 Hours Hot Posts";
  } else {
    return "Currently don't have any post";
  }
})

function goLogout() {
  toast.add({
    description: "Are you sure you want to log out now?",
    actions: [{
      label: 'Yes!',
      async click() {
        await logout();
        location.replace("/");
      }
    }, {
      label: 'No'
    }]
  })
}
</script>

<template>
  <div class="space-y-1.5">
    <UCard v-if="!isPostUser && !currentUser">
      <template #header>
        <div class="flex items-center justify-center">
          <span>Login to explore more.</span>
        </div>
      </template>
      <template #footer>
        <div class="flex items-center justify-center gap-1.5">
          <UButton to="/login">Login</UButton>
          <UButton v-if="serverInfo?.open_register" to="/register">Register</UButton>
        </div>
      </template>
    </UCard>
    <UserDescription v-else :user="isPostUser ? postUser! : currentUser!" />
    <UCard v-if="currentUser">
      <div class="flex flex-wrap items-center gap-1.5">
        <UButton size="sm" square variant="soft" to="/settings" icon="i-heroicons-adjustments-horizontal" />
        <UChip :text="totalMessageText" size="2xl">
          <UButton size="sm" variant="soft" square to="/notifications"
            icon="i-heroicons-chat-bubble-bottom-center-text" />
        </UChip>
        <UButton size="sm" icon="i-heroicons-arrow-left-on-rectangle" square variant="soft" @click="goLogout" />
      </div>
    </UCard>
    <UAlert :title="topPostsTitle" />
    <UCard v-if="posts" v-for="post in posts.data.items">
      <div class="flex gap-1.5 items-center mx-2">
        <UIcon name="i-heroicons-fire-20-solid" />
        <UIcon v-if="post.status === PostStatus.Banned" name="i-heroicons-lock-closed" class="text-red-500" />
        <UIcon v-else-if="post.status === PostStatus.Archived" name="i-heroicons-lock-closed" class="text-yellow-500" />
        <ULink class="line-clamp-1" :to="`/post/${post.id}`" active-class="text-primary"
          inactive-class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
          {{ post.title }}
        </ULink>
      </div>
    </UCard>
  </div>
</template>
