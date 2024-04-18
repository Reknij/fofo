<script setup lang="ts">
import '~/styles/global.css'

const config = useRuntimeConfig();
useHead({
  titleTemplate: (title) => {
    return title
      ? `${title} - ${config.public.forumName}`
      : config.public.forumName
  }
});

</script>

<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <UNotifications />
    <UModals />
  </div>
</template>