<script setup lang="ts">
import { MessageOutlined, SettingOutlined } from "@vicons/antd";
import { NSpace, NCard, NButton, NBadge, NIcon, NText } from "naive-ui";
import { getUserNotifications } from "~/api/notification";
import { getPostsNoContent } from "~/api/post";
import { PostAlgorithmOrder } from "~/models/post";
import type { SafeUserInfo, UserTag } from "~/models/user";
import { useCurrentUser, logout, isLogined } from "~/states/auth";

const router = useRouter();
const props = defineProps<{
  user: SafeUserInfo;
  tag?: UserTag | undefined;
}>();
const currentUser = useCurrentUser();

const { data: notifications, refresh: refreshNotifications } =
  await getUserNotifications({
    index: 0,
    limit: 0,
    extended: false,
    only_unread: true,
  });

watch(router.currentRoute, async () => {
  await refreshNotifications();
});

async function goLogout() {
  await logout();
  location.replace("/");
}

async function goNotifications() {
  await router.push("/notifications");
}

async function goSettings() {
  await router.push("/settings");
}
</script>

<template>
  <div>
    <n-card size="small">
      <n-space vertical align="center">
        <n-space align="center">
          <FofoUserAvatar :user="user" :tag="tag"></FofoUserAvatar>
          <n-button
            text
            @click="goNotifications"
            v-if="currentUser?.id == user.id && notifications"
          >
            <template #icon>
              <n-badge type="info" :max="99" :value="notifications?.data.total">
                <n-icon :size="25">
                  <MessageOutlined />
                </n-icon>
              </n-badge>
            </template>
          </n-button>
          <n-button text v-if="currentUser?.id == user.id" @click="goSettings">
            <template #icon>
              <n-icon :size="25">
                <SettingOutlined />
              </n-icon>
            </template>
          </n-button>
        </n-space>
        <span v-if="user.signature">{{ user.signature }}</span>
        <n-text code>{{ currentUser?.total_post ?? 0 }} created posts.</n-text>
        <n-text code
          >{{ currentUser?.total_comment ?? 0 }} created comments.</n-text
        >
        <n-button v-if="currentUser?.id == user.id" @click="goLogout"
          >Logout..</n-button
        >
      </n-space>
    </n-card>
  </div>
</template>
