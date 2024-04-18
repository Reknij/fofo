<script setup lang="ts">
import { NMessageProvider, NConfigProvider, type GlobalThemeOverrides, NDialogProvider, NLoadingBarProvider } from 'naive-ui';


const config = useRuntimeConfig();
useHead({
  titleTemplate: (title) => {
    return title
      ? `${title} - ${config.public.forumName}`
      : config.public.forumName
  }
});

const themeOverrides: GlobalThemeOverrides = {
  common: {
    baseColor: '#e2e2e2',
    bodyColor: '#e2e2e2',
    dividerColor: '#BBBBBB',
    borderColor: '#BBBBBB',
  },
}
</script>

<template>
  <div>
    <n-config-provider :theme-overrides="themeOverrides">
      <n-loading-bar-provider>
        <n-message-provider>
          <n-dialog-provider>
            <NuxtLayout>
              <NuxtPage />
            </NuxtLayout>
          </n-dialog-provider>
        </n-message-provider>
      </n-loading-bar-provider>
    </n-config-provider>
  </div>
</template>

<style>
.clickable:hover {
  cursor: pointer;
}

.n-icon {
  display: flex;
  align-items: center;
}
</style>