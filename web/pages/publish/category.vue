<script setup lang="ts">
import type { Category } from "~/models/category";
import { getCategory } from "~/api/category";
import { UserType } from "~/models/user";
import { useCurrentUser } from "~/states/auth";

const router = useRouter();
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
const links = [
  {
    label: 'Categories',
    to: '/categories',
  },
  {
    label: edit.value?.title ?? "Create category",
    to: edit.value ? `/category/${edit.value.id}` : undefined
  },
]
</script>

<template>
  <ClientOnly>
    <div class="space-y-2">
      <FofoBreadcrumb :links="links" />
      <CategoryEditor v-if="user?.user_type == UserType.Administrator" :edit="edit"></CategoryEditor>
      <UAlert v-else title="No permission to access..." />
    </div>
  </ClientOnly>
</template>
