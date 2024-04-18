<script setup lang="ts">
import {
  useMessage,
  NForm,
  NFormItem,
  NInput,
  NCard,
  NButton,
  NSelect,
  type FormRules,
} from "naive-ui";
import { GroupStatus, type Group, type GroupToCreateUpdate } from "~/models/group";
import { ContentType } from "~/models/util";
import { createGroup, updateGroup } from "~/api/group";
import { getApiDetailError, bytemdPlugins } from "~/helper";
import MarkdownEditor from "./MarkdownEditor.vue";

const now = Date.now();
const router = useRouter();
const message = useMessage();
const props = defineProps<{
  edit?: Group;
}>();
const formValue = ref<GroupToCreateUpdate>({
  title: props.edit?.title ?? "",
  description: props.edit?.description ?? "",
  description_content_type:
    props.edit?.description_content_type ?? ContentType.Markdown,
  status: props.edit?.status ?? GroupStatus.Active,
});
const rules = ref<FormRules>({
  title: {
    validator(rule, value: string) {
      return !value.startsWith(" ") && value.length > 0;
    },
    required: true,
    message: "Please enter group title, must unique",
    trigger: "blur",
  },
  status: {
    validator(rule, value) {
      return true;
    },
    message: "Please enter the group status.",
    trigger: "blur",
  },
});

const GroupStatusOptions = [
  {
    label: "Active",
    value: GroupStatus.Active,
  },
  {
    label: "Observer",
    value: GroupStatus.Observer,
  },
  {
    label: "OnlyComment",
    value: GroupStatus.OnlyComment,
  },
];

const ContentTypeOptions = [
  {
    label: "Markdown",
    value: ContentType.Markdown,
  },
];

async function runCreate() {
  if (props.edit) {
    let { data: group, error } = await updateGroup(props.edit.id, {
      target: {
        ...formValue.value,
      },
    });
    if (group.value) {
      message.success("Success update the group!");
      await router.push(`/group/${group.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      message.error(`(${err?.code}) Update failed. ${err?.msg}`);
    }
  } else {
    let { data: group, error } = await createGroup({
      target: {
        ...formValue.value,
      },
    });
    if (group.value) {
      message.success("Success create the group!");
      await router.push(`/group/${group.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      message.error(`(${err?.code}) Create failed. ${err?.msg}`);
    }
  }
}
</script>

<template>
  <n-card size="small">
    <n-form ref="formRef" :label-width="80" :model="formValue" :rules="rules">
      <n-form-item label="Title" path="title">
        <n-input v-model:value="formValue.title" placeholder="Group title" />
      </n-form-item>
      <n-form-item label="Description" path="description">
        <MarkdownEditor
          v-model:value="formValue.description"
          placeholder="Your group description content."
        />
      </n-form-item>
      <n-form-item label="Status" path="status">
        <n-select
          v-model:value="formValue.status"
          :options="GroupStatusOptions"
        />
      </n-form-item>
      <n-form-item>
        <n-button @click="runCreate"> {{ edit? "Update": "Create" }} </n-button>
      </n-form-item>
    </n-form>
  </n-card>
</template>
