<script setup lang="ts">
/* __placeholder__ */
import type { Category } from "~/models/category";
import { NCard, useLoadingBar } from "naive-ui";
import { getCategory } from "~/api/category";
import { UserType } from "~/models/user";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();
const loadingBar = useLoadingBar();
loadingBar.start();
const query = router.currentRoute.value.query as any as {
  edit_id?: string;
};
const editMode = computed(() => query.edit_id !== undefined);
const edit = ref<Category>();
const user = useCurrentUser();
if (editMode.value) {
  const { data } = await getCategory(Number.parseInt(query.edit_id!));
  if (data.value) {
    edit.value = data.value;
  }
}
onMounted(() => loadingBar.finish());
</script>

<template>
  <ClientOnly>
    <CategoryEditor
      v-if="user?.user_type == UserType.Administrator"
      :edit="edit"
    ></CategoryEditor>
    <n-card v-else>No permission to access...</n-card>
  </ClientOnly>
</template>
