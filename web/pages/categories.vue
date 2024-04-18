<script setup lang="ts">
import { NSpace, NCard, NButton, useLoadingBar } from 'naive-ui';
import type { SubPath } from '~/components/FofoBreadcrumb/model';
import { UserType } from '~/models/user';
import { useCurrentUser } from '~/states/auth';

const router = useRouter();
const loadingBar = useLoadingBar()
loadingBar.start()
const user = useCurrentUser();

const subpaths: SubPath[] = [
  {
    label: 'Categories'
  }
]

async function goCreate() {
  await router.push('/publish/category')
}

onMounted(()=>loadingBar.finish())
</script>

<template>
  <n-space vertical>
    <FofoBreadcrumb :subpath="subpaths"></FofoBreadcrumb>
    <n-card size="small" v-if="user?.user_type == UserType.Administrator">
      <n-button round @click="goCreate">Create category</n-button>
    </n-card>
    <n-card size="small">
      <Categories></Categories>
    </n-card>
  </n-space>
</template>