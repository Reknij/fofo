<script setup lang="ts">
import { type Category, CategoryStatus } from "~/models/category";
import { UserType } from "~/models/user";
import { ContentType } from "~/models/util";
import { createCategory, updateCategory } from "~/api/category";
import { getApiDetailError } from "~/helper";
import MarkdownEditor from "./MarkdownEditor.vue";
import { object, string, type InferType } from 'yup'

const now = Date.now();
const router = useRouter();
const toast = useToast();

const props = defineProps<{
  edit?: Category;
}>();

const state = reactive({
  title: props.edit?.title ?? "",
  description: props.edit?.description ?? "",
  description_content_type:
    props.edit?.description_content_type ?? ContentType.Markdown,
  read_level: props.edit?.read_level ?? UserType.Guest,
  write_level: props.edit?.write_level ?? UserType.General,
  comment_level: props.edit?.comment_level ?? UserType.General,
  moderator_ids: props.edit?.moderator_ids.join(", ") ?? "",
  group_ids: props.edit?.group_ids.join(", ") ?? "",
  status: props.edit?.status ?? CategoryStatus.Active,
});
const schema = object({
  title: string().required("Required"),
  description: string().required("Required"),
});

async function runCreate() {
  const group_ids = state.group_ids
    .replaceAll(" ", "")
    .split(",")
    .map((g) => Number.parseInt(g))
    .filter((g) => !isNaN(g));
  const moderator_ids = state.moderator_ids
    .replaceAll(" ", "")
    .split(",")
    .map((g) => Number.parseInt(g))
    .filter((g) => !isNaN(g));

  if (props.edit) {
    let { data: category, error } = await updateCategory(props.edit.id, {
      target: {
        ...state,
        total_post: 0,
        group_ids,
        moderator_ids,
      },
    });
    if (category.value) {
      toast.add({
        description: "Success update the category!"
      })
      await router.push(`/category/${category.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Update failed. ${err?.msg}`
      })
    }
  } else {
    let { data: category, error } = await createCategory({
      target: {
        ...state,
        group_ids,
        moderator_ids,
      },
    });
    if (category.value) {
      toast.add({
        description: "Success create the category!"
      })
      await router.push(`/category/${category.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Create failed. ${err?.msg}`
      })
    }
  }
}
</script>

<template>
  <UCard>
    <UForm class="space-y-1.5" :schema="schema" :state="state" @keyup.enter="runCreate">
      <UFormGroup label="Title" path="title">
        <UInput v-model="state.title" placeholder="Category title" />
      </UFormGroup>
      <UFormGroup label="Description" path="description">
        <MarkdownEditor v-model:value="state.description" placeholder="Your category description content." />
      </UFormGroup>
      <UFormGroup label="Read level" path="read_level">
        <UserTypeSelectMenu v-model:value="state.read_level" />
      </UFormGroup>
      <UFormGroup label="Write level" path="write_level">
        <UserTypeSelectMenu v-model:value="state.write_level" />
      </UFormGroup>
      <UFormGroup label="Comment level" path="comment_level">
        <UserTypeSelectMenu v-model:value="state.comment_level" />
      </UFormGroup>
      <UFormGroup label="Status" path="status">
        <CategoryStatusSelectMenu v-model:value="state.status" />
      </UFormGroup>
      <UFormGroup label="Group ids" path="group_ids">
        <UInput v-model="state.group_ids" placeholder="Group ids, use `,` to split." />
      </UFormGroup>
      <UFormGroup label="Moderator ids" path="moderator_ids">
        <UInput v-model="state.moderator_ids" placeholder="Moderator ids, use `,` to split." />
      </UFormGroup>
      <UFormGroup>
        <UButton @click="runCreate"> {{ edit ? "Update" : "Create" }} </UButton>
      </UFormGroup>
    </UForm>
  </UCard>
</template>
