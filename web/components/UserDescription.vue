<script setup lang="ts">
import { timeAgo } from "~/helper";
import type { SafeUserInfo } from "~/models/user";

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
    <div class="flex flex-col items-center justify-center">
      <span class="code">{{ user.signature }}</span>
      <UDivider class="my-2" />
      <div>
        <span class="code">{{ user.total_post }}</span>
        created posts.
      </div>
      <UDivider class="my-2" />
      <div>
        <span class="code">{{ user.total_comment }}</span>
        created comments.
      </div>
      <UDivider class="my-2" />
      <div>
        Joined time
        <span class="code">{{ timeAgo(user.created_at) }}</span>
      </div>
    </div>
  </UCard>
</template>
