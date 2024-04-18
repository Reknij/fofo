<script setup lang="ts">
import { getGroup } from "~/api/group";
import type { Group } from "~/models/group";
import { UserType } from "~/models/user";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();
const query = router.currentRoute.value.query as any as {
  edit_id?: string;
};
const editMode = computed(() => query.edit_id !== undefined);
const edit = ref<Group>();
const user = useCurrentUser();
if (editMode.value) {
  const { data } = await getGroup(Number.parseInt(query.edit_id!));
  if (data.value) {
    edit.value = data.value;
  }
}
const links = [
  {
    label: 'Groups',
    to: '/groups',
  },
  {
    label: edit.value?.title ?? "Create group",
    to: edit.value ? `/group/${edit.value.id}` : undefined
  },
]
</script>

<template>
  <ClientOnly>
    <div class="space-y-2">
      <FofoBreadcrumb :links="links" />
      <GroupEditor v-if="user?.user_type == UserType.Administrator" :edit="edit"></GroupEditor>
      <UAlert v-else title="No permission to access..." />
    </div>
  </ClientOnly>
</template>
