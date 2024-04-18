<script setup lang="ts">
import { NSpace, NCard, NIcon, NButton } from "naive-ui";
import { PostAlgorithmOrder } from "~/models/post";
import { FireOutlined } from "@vicons/antd";
import { useCurrentUser } from "~/states/auth";
import { usePostUserAndTag } from "~/states/post";
import { getServerInfo } from "~/api/server";

const currentUser = useCurrentUser();
const userAndTag = usePostUserAndTag();
const router = useRouter();
const currentRoute = router.currentRoute;
const { data: serverInfo } = await getServerInfo();

async function goToLogin() {
  await router.push("/login");
}

async function goToRegister() {
  await router.push("/register");
}

const isPostUser = computed(
  () => currentRoute.value.path.startsWith("/post/") && userAndTag.value
);
</script>

<template>
  <n-space vertical>
    <n-card v-if="!isPostUser && !currentUser" size="small">
      <n-space vertical align="center">
        <span>Login to explore more.</span>
        <n-space>
          <n-button @click="goToRegister" v-if="serverInfo?.open_register"
            >Register</n-button
          >
          <n-button @click="goToLogin">Login</n-button>
        </n-space>
      </n-space>
    </n-card>

    <UserDescription
      v-else
      :user="isPostUser? userAndTag!.user: currentUser!"
      :tag="isPostUser? userAndTag!.tag: undefined"
    ></UserDescription>

    <n-card size="small"> 24 Hours Hot Posts </n-card>
    <n-card size="small">
      <PostList
        :text_max_width="250"
        :sort="PostAlgorithmOrder.Hot"
        time="day"
        :distinct="false"
        :time_num="1"
        :limit="10"
        :max="10"
        simplification
      >
        <template #prefix>
          <n-icon :size="25">
            <FireOutlined></FireOutlined>
          </n-icon>
        </template>
      </PostList>
    </n-card>
  </n-space>
</template>
