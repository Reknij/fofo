<script setup lang="ts">
import { logout, useCurrentUser } from "~/states/auth";
import { UInput } from "#components";

const currentUser = useCurrentUser();
const isOpen = ref(false)
const config = useRuntimeConfig();
const colorMode = useColorMode();
const toast = useToast();
const isDark = computed({
  get() {
    return colorMode.value === 'dark'
  },
  set(toDark) {
    colorModeLink.icon = toDark ? "i-heroicons-moon" : "i-heroicons-sun";
    colorMode.preference = toDark ? 'dark' : 'light'
  }
})
onMounted(() => {
  if (isDark.value) {
    colorModeLink.icon = "i-heroicons-moon"
  }
})

const searchValue = ref('');

const fofoTitleLink = {
  badge: {
    color: 'primary',
    variant: 'subtle',
    class: 'py-0 text-2xl',
    label: config.public.forumName,
  },
  to: '/'
};
const googleSearchLink = {
  label: 'googleSearch',
};
const colorModeLink = reactive({
  icon: "i-heroicons-sun",
  label: '',
  isSwitch: true,
  click() {
    isDark.value = !isDark.value;
  },
})
const mainLinks = [
  {
    label: "Categories",
    icon: 'i-heroicons-square-3-stack-3d',
    to: '/categories'
  },
  {
    label: 'Groups',
    icon: 'i-heroicons-user-group',
    to: "/groups"
  },
  {
    label: 'About',
    icon: 'i-heroicons-information-circle',
    to: '/about'
  }
]

const desktopLinks = [
  [fofoTitleLink as any],
  [googleSearchLink as any],
  [
    colorModeLink,
    ...mainLinks,
  ],
]

const mobileLinks = [
  [googleSearchLink as any],
  mainLinks,
  [
    {
      label: "Login",
      icon: 'i-heroicons-user-circle',
      to: '/login'
    }, {
      label: "Register",
      icon: 'i-heroicons-user-plus',
      to: '/register'
    },
  ]
]

const loginedMobileLinks = [
  ...mobileLinks,
  [{
    label: "Notifications",
    icon: 'i-heroicons-chat-bubble-bottom-center-text',
    to: '/notifications'
  }, {
    label: "Settings",
    icon: 'i-heroicons-adjustments-horizontal',
    to: '/settings'
  }, {
    label: "Logout",
    icon: 'i-heroicons-arrow-left-on-rectangle',
    async click() {
      toast.add({
        description: "Are you sure you want to log out now?",
        actions: [{
          label: 'Yes!',
          async click() {
            await logout();
            location.replace("/");
          }
        }, {
          label: 'No'
        }]
      })
    }
  }]
]

mobileLinks.forEach(links => links.forEach((link: any) => {
  if (link.label === 'googleSearch') return;
  link.click = () => {
    isOpen.value = false;
  }
}))
loginedMobileLinks.forEach(links => links.forEach((link: any) => {
  if (link.label === 'googleSearch') return;
  link.click = () => {
    isOpen.value = false;
  }
}))

const headerLinks = [
  [fofoTitleLink as any],
  [
    colorModeLink,
    {
      label: '',
      icon: 'i-heroicons-bars-3-bottom-right-20-solid',
      iconClass: 'w-8',
      click() {
        isOpen.value = true
      }
    },
  ]
]
</script>

<template>
  <div>
    <div
      class="border-b border-neutral-300 dark:border-neutral-900 shadow-md">
      <UHorizontalNavigation :links="desktopLinks" :ui="{ base: 'p-2' }" class="hidden xl:flex max-w-screen-xl mx-auto">
        <template #default="{ link }">
          <SearchComponent v-model="searchValue" v-if="link.label === 'googleSearch'" />
          <span v-else class="group-hover:text-primary relative">{{ link.label }}</span>
        </template>
      </UHorizontalNavigation>
      <UHorizontalNavigation class="xl:hidden" :ui="{ base: 'p-2' }" :links="headerLinks" />
    </div>

    <USlideover v-model="isOpen" prevent-close>
      <UCard class="flex flex-col flex-1"
        :ui="{ body: { base: 'flex-1' }, ring: '', divide: 'divide-y divide-gray-100 dark:divide-gray-800' }">
        <template #header>
          <div class="flex items-center justify-between p-2">
            <NuxtLink class="text-base font-semibold leading-6 text-gray-900 dark:text-white" to="/">
              {{ $config.public.forumName }}
            </NuxtLink>
            <UButton color="gray" variant="ghost" icon="i-heroicons-x-mark-20-solid" class="-my-1 shadow-none"
              @click="isOpen = false" />
          </div>
        </template>
        <UVerticalNavigation :links="currentUser ? loginedMobileLinks : mobileLinks">
          <template #default="{ link }">
            <SearchComponent v-model="searchValue" v-if="link.label === 'googleSearch'" />
            <span v-else class="group-hover:text-primary relative">{{ link.label }}</span>
          </template>
        </UVerticalNavigation>
      </UCard>
    </USlideover>
  </div>
</template>