<script setup lang="ts">
import type { SafeUserInfo, UserTag } from "~/models/user";

const props = defineProps<{
  user: SafeUserInfo;
}>();
</script>

<template>
  <UCard>
    <template #header>
      <div class="flex items-center justify-center">
        <FofoUserAvatar :user="user"></FofoUserAvatar>
      </div>
    </template>
    <span class="code">{{ user.signature }}</span>
    <UDivider class="my-2" />
    <span class="code">{{ user.total_post }}</span>
    created posts.
    <UDivider class="my-2" />
    <span class="code">{{ user.total_comment }}</span>
    created comments.
  </UCard>
</template>
