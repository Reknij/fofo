<script setup lang="ts">
import { MenuOutlined } from "@vicons/antd";
import {
  NMenu,
  type MenuOption,
  NIcon,
  NDrawer,
  NButton,
  useMessage,
} from "naive-ui";
import { isLogined, useCurrentUser } from "~/states/auth";

const router = useRouter();
const route = router.currentRoute;
const config = useRuntimeConfig();
const drawerActive = ref(false);
const currentUser = useCurrentUser();

const props = defineProps<{
  vertical?: boolean;
}>();

const NuxtLink = defineNuxtLink({
  componentName: "MyNuxtLink",
});

function getMenuOptions(vertical?: boolean) {
  const menuOptions: MenuOption[] = [
    {
      label: () =>
        h(
          NuxtLink,
          {
            to: "/categories",
          },
          () => "Categories"
        ),
      key: "/categories",
    },
    {
      label: () =>
        h(
          NuxtLink,
          {
            to: "/groups",
          },
          () => "Groups"
        ),
      key: "/groups",
    },
    {
      label: () =>
        h(
          NuxtLink,
          {
            to: "/about",
          },
          () => "About"
        ),
      key: "/about",
    },
  ];
  if (vertical) {
    return [
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: "/",
            },
            () =>
              h(
                "span",
                {
                  style: "font-size: 30px; font-weight: bold;",
                },
                config.public.forumName
              )
          ),
        key: "/",
        //icon: renderIcon(BookIcon)
      },
      {
        key: "divider-1",
        type: "divider",
      },
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: `/login`,
            },
            () => "Login"
          ),
        key: `/login`,
        show: !isLogined(),
      },
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: `/register`,
            },
            () => "Register"
          ),
        key: `/register`,
        show: !isLogined(),
      },
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: `/user/${currentUser.value?.id}`,
            },
            () => "UserInfo"
          ),
        key: `/user/${currentUser.value?.id}`,
        show: isLogined(),
      },
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: "/notifications",
            },
            () => "Notifications"
          ),
        key: "/notifications",
        show: isLogined(),
      },
      {
        key: "divider-1",
        type: "divider",
      },
      ...menuOptions,
    ] as MenuOption[];
  } else {
    return [
      {
        label: () =>
          h(
            NuxtLink,
            {
              to: "/",
            },
            () =>
              h(
                "span",
                {
                  style: "font-size: 30px; font-weight: bold;",
                },
                config.public.forumName
              )
          ),
        key: "/",
        //icon: renderIcon(BookIcon)
      },
      ...menuOptions,
    ] as MenuOption[];
  }
}

const verticalOptions: MenuOption[] = [
  {
    label: () =>
      h(
        NButton,
        {
          onClick: () => (drawerActive.value = !drawerActive.value),
        },
        {
          icon: () =>
            h(NIcon, {
              component: MenuOutlined,
            }),
        }
      ),
    key: "drawerBtn",
  },
  {
    label: () =>
      h(
        NuxtLink,
        {
          to: "/",
        },
        () =>
          h(
            "span",
            {
              style: "font-size: 30px; font-weight: bold;",
            },
            config.public.forumName
          )
      ),
    key: "/",
  },
];
</script>

<template>
  <div>
    <div v-if="vertical">
      <n-menu
        mode="horizontal"
        v-model:value="route.path"
        :options="verticalOptions"
      ></n-menu>
      <n-drawer v-model:show="drawerActive" placement="left">
        <n-menu v-model:value="route.path" :options="getMenuOptions(true)" />
      </n-drawer>
    </div>
    <n-menu
      v-else
      v-model:value="route.path"
      mode="horizontal"
      :options="getMenuOptions()"
    />
  </div>
</template>

<style>
#fofoTitle {
  font-size: large;
}
</style>
