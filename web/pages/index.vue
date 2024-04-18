<script setup lang="ts">
import { PostAlgorithmOrder } from "~/models/post";
import { isLogined } from "~/states/auth";
import { getUserNotifications } from "~/api/notification";
import type { GetDatasExtended } from "~/models/util";
import type { UserNotification } from "~/models/notification";

const router = useRouter();

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

</script>

<template>
  <div class="space-y-2">
    <UAlert v-if="notifications?.data.total" color="primary" variant="subtle" title="Notifications"
      :description="`You have ${notifications.data.total} unread notifications.`" :actions="[{
        label: 'View now',
        color: 'primary',
        variant: 'solid',
        async click() {
          await goNotifications()
        }
      }]">
    </UAlert>
    <UAlert title="This week's post."></UAlert>
    <PostList :sort="PostAlgorithmOrder.Newest" :distinct="config.public.default.distinct" :limit="20" time="week"
      :time_num="1" :top_order_enable="true" disable_query></PostList>
    <UAlert title="Hot post."></UAlert>
    <PostList :sort="PostAlgorithmOrder.Hot" :top_order_enable="false" :distinct="config.public.default.distinct" :limit="10"></PostList>
  </div>
</template>
