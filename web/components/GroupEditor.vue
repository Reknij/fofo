<script setup lang="ts">
import { GroupStatus, type Group, type GroupToCreateUpdate } from "~/models/group";
import { ContentType } from "~/models/util";
import { createGroup, updateGroup } from "~/api/group";
import { getApiDetailError, bytemdPlugins } from "~/helper";
import MarkdownEditor from "./MarkdownEditor.vue";
import { object, string } from "yup";

const now = Date.now();
const router = useRouter();
const toast = useToast();
const props = defineProps<{
  edit?: Group;
}>();
const state = reactive<GroupToCreateUpdate>({
  title: props.edit?.title ?? "",
  description: props.edit?.description ?? "",
  description_content_type:
    props.edit?.description_content_type ?? ContentType.Markdown,
  status: props.edit?.status ?? GroupStatus.Active,
});
const schema = object({
  title: string().required("Required"),
})

async function runCreate() {
  if (props.edit) {
    let { data: group, error } = await updateGroup(props.edit.id, {
      target: {
        ...state,
      },
    });
    if (group.value) {
      toast.add({
        description: "Success update the group!"
      })
      await router.push(`/group/${group.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Update failed. ${err?.msg}`
      })
    }
  } else {
    let { data: group, error } = await createGroup({
      target: {
        ...state,
      },
    });
    if (group.value) {
      toast.add({
        description: "Success create the group!"
      })
      await router.push(`/group/${group.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      toast.add({
        color: 'red',
        description: `(${err?.code}) Create failed. ${err?.msg}`,
      })
    }
  }
}
</script>

<template>
  <UCard>
    <UForm class="space-y-1.5" :schema="schema" :state="state">
      <UFormGroup label="Title" path="title">
        <UInput v-model="state.title" placeholder="Group title" />
      </UFormGroup>
      <UFormGroup label="Description" path="description">
        <MarkdownEditor v-model:value="state.description" placeholder="Your group description content." />
      </UFormGroup>
      <UFormGroup label="Status" path="status">
        <GroupStatusSelectMenu v-model:value="state.status" />
      </UFormGroup>
      <UFormGroup>
        <UButton @click="runCreate"> {{ edit ? "Update" : "Create" }} </UButton>
      </UFormGroup>
    </UForm>
  </UCard>
</template>
