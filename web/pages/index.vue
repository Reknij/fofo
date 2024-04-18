<script setup lang="ts">
import { PostAlgorithmOrder } from "~/models/post";
import { NCard, NSpace, NAlert, useLoadingBar } from "naive-ui";
import { isLogined } from "~/states/auth";
import { getUserNotifications } from "~/api/notification";
import type { GetDatasExtended } from "~/models/util";
import type { UserNotification } from "~/models/notification";

const router = useRouter();
const loadingBar = useLoadingBar();
loadingBar.start();

const config = useRuntimeConfig();
let notifications: GetDatasExtended<UserNotification> | undefined = undefined;
if (isLogined()) {
  const { data } = await getUserNotifications({
    index: 0,
    limit: 0,
    extended: false,
    only_unread: true,
  });
  if (data.value) notifications = data.value;
}

async function goNotifications() {
  await router.push("/notifications");
}

useHead({
  title: `Home`,
});

onMounted(() => loadingBar.finish());
</script>
<template>
  <n-space vertical>
    <n-space v-if="(notifications?.data.total ?? 0) > 0" vertical>
      <n-alert
        class="clickable"
        @click="goNotifications"
        title="Note"
        type="info"
      >
        You have {{ notifications?.data.total }} unread notifications.
      </n-alert>
    </n-space>
    <n-card size="small">This week's post.</n-card>
    <n-card size="small">
      <PostList
        :sort="PostAlgorithmOrder.Newest"
        :distinct="config.public.default.distinct"
        query_pagination
        :top_order_enable="true"
      ></PostList>
    </n-card>
  </n-space>
</template>
