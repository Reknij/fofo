<script setup lang="ts">
import { NAvatar, NSpace, NTag, NText } from "naive-ui";
import { getAvatar } from "~/helper";
import { type SafeUserInfo, UserType, UserTag } from "~/models/user";
import { useCurrentUser } from "~/states/auth";

const props = defineProps<{
  disableAvatar?: boolean;
  user?: SafeUserInfo;
  tag?: UserTag;
}>();
const router = useRouter();
const currentUser = useCurrentUser();

function getUserTags() {
  const op = props.tag ? (props.tag & UserTag.OP) != 0 : false;
  const moderator = props.tag ? (props.tag & UserTag.Moderator) != 0 : false;
  const admin = props.user?.user_type == UserType.Administrator;
  const tags: string[] = [];
  if (admin) tags.push("Admin");
  else if (moderator) tags.push("Mod");
  if (op) tags.push("OP");
  if (currentUser.value?.id === props.user?.id) tags.push("You");
  return tags;
}
const tags = ref(getUserTags());
watch(() => props, () => {
  tags.value = getUserTags();
});
async function goUser(e: Event) {
  e.preventDefault();
  await router.push(`/user/${props.user?.id}`);
}
</script>

<template>
  <div>
    <n-space align="center" :size="[5, 0]">
      <n-space align="center" :size="[4, 0]">
        <n-avatar
          v-if="disableAvatar !== true"
          class="clickable"
          :src="getAvatar(user)"
          @click="goUser"
        ></n-avatar>
        <n-space align="center" :size="0">
          <a class="userIdentity" @click="goUser" :href="`/user/${user?.id}`">
            <n-text>{{ `${user?.alias}` ?? "Unknown" }}</n-text>
            <n-text code>{{ `@${user?.username}` }}</n-text>
          </a>
        </n-space>
      </n-space>
      <n-tag v-if="tags.length > 0" :bordered="false" type="info">{{
        tags.join(" | ")
      }}</n-tag>
      <slot></slot>
    </n-space>
  </div>
</template>

<style>
.userIdentity {
  text-decoration: none;
}

.userIdentity:hover {
  text-decoration: underline;
  text-decoration-color: black;
}
</style>
