<script setup lang="ts">
import { getGroups } from '~/api/group';
import { GetGroupsSort, type GetGroupsQuery } from '~/models/group';
import { UserType } from '~/models/user';
import { useCurrentUser } from '~/states/auth';

const router = useRouter();
const user = useCurrentUser();
const links = [
  {
    label: 'Groups',
  }
]

const query = reactive<GetGroupsQuery>({
  index: 0,
  limit: 20,
  sort: GetGroupsSort.Id,
  desc: false,
  extended: false,
})
const { data } = await getGroups(query);

async function goCreate() {
  await router.push('/publish/group')
}

</script>

<template>
  <div class="space-y-1.5">
    <FofoBreadcrumb :links="links"></FofoBreadcrumb>
    <UButton variant="soft" v-if="user?.user_type == UserType.Administrator" @click="goCreate">Create group</UButton>
    <UPagination :model-value="query.index + 1" @update:model-value="(v: number) => query.index = v - 1" :page-count="query.limit"
      :total="data?.data.total ?? 0" />
    <UCard v-for="item in data?.data.items">
      <template #header>
        <div class="flex justify-between">
          <div class="flex items-center gap-1.5">
            <UIcon name="i-heroicons-user-group" />
            <ULink active-class="text-primary"
              inactive-class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
              :to="`/group/${item.id}`">{{ item.title }}</ULink>
          </div>
        </div>
      </template>
    </UCard>
  </div>
</template>