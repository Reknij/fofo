<script setup lang="ts">
import { getGroup } from "~/api/group";
import type { Group } from "~/models/group";
import { NCard, useLoadingBar } from "naive-ui";
import { UserType } from "~/models/user";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();
const loadingBar = useLoadingBar();
loadingBar.start();
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

onMounted(() => loadingBar.finish());
</script>

<template>
  <ClientOnly>
    <GroupEditor
      v-if="user?.user_type == UserType.Administrator"
      :edit="edit"
    ></GroupEditor>
    <n-card v-else>No permission to access...</n-card>
  </ClientOnly>
</template>
