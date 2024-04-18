<script setup lang="ts">
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
watch(() => props.user, () => {
  tags.value = getUserTags();
});
async function goUser(e: Event) {
  e.preventDefault();
  await router.push(`/user/${props.user?.id}`);
}
</script>

<template>
  <div>
    <div class="flex flex-wrap items-center gap-x-1 gap-y-1">
      <img v-if="disableAvatar !== true" class="clickable rounded-lg size-8" :src="getAvatar(user)" @click="goUser" />
      <a class="userIdentity" @click="goUser" :href="`/user/${user?.id}`">
        <span class="text-sm">{{ `${user?.alias}` ?? "Unknown" }}</span>
        <span class="code !px-0">{{ `@${user?.username}` }}</span>
      </a>
      <UBadge variant="subtle" v-if="tags.length > 0">{{
        tags.join(" | ")
        }}</UBadge>
      <slot></slot>
      </div>
  </div>
</template>

<style>
.userIdentity {
  text-decoration: none;
}

.userIdentity:hover {
  text-decoration: underline;
  @apply decoration-black dark:decoration-white
}
</style>
