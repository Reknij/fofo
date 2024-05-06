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

useHead({
  title: `Home`,
});

</script>

<template>
  <div class="space-y-1.5">
    <UAlert v-if="notifications?.data.total" color="yellow" variant="subtle" title="Notifications"
      :description="`You have ${notifications.data.total} unread notifications.`" :actions="[{
        label: 'View now',
        color: 'yellow',
        variant: 'solid',
        click: async () => await router.push(`/notifications`),
      }]">
    </UAlert>
    <UAlert title="This week's post." color="primary" variant="subtle" />
    <PostList :sort="PostAlgorithmOrder.Newest" :distinct="config.public.default.distinct" :limit="20" time="week"
      :time_num="1" :top_order_enable="true" disable_query></PostList>
    <UAlert title="All posts." color="primary" variant="subtle" />
    <PostList :sort="PostAlgorithmOrder.Hot" :top_order_enable="false" :distinct="config.public.default.distinct"
      :limit="10"></PostList>
  </div>
</template>
