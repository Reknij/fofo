<script setup lang="ts">
import {
  useMessage,
  NForm,
  NFormItem,
  NInput,
  NCard,
  NImage,
  NButton,
  NSelect,
  type FormRules,
} from "naive-ui";
import { type Category, CategoryStatus } from "~/models/category";
import { UserType } from "~/models/user";
import { ContentType } from "~/models/util";
import { createCategory, updateCategory } from "~/api/category";
import { getApiDetailError, bytemdPlugins } from "~/helper";
import MarkdownEditor from "./MarkdownEditor.vue";

const now = Date.now();
const router = useRouter();
const message = useMessage();

const props = defineProps<{
  edit?: Category;
}>();

const formValue = ref({
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
const rules = ref<FormRules>({
  title: {
    validator(rule, value: string) {
      return !value.startsWith(" ") && value.length > 0;
    },
    required: true,
    message: "Please enter category title, must unique",
    trigger: "blur",
  },
  read_level: {
    validator(rule, value) {
      return true;
    },
    message: "Please enter the minimum read level.",
    trigger: "blur",
  },
  write_level: {
    validator(rule, value) {
      return true;
    },
    message: "Please enter the minimum write level.",
    trigger: "blur",
  },
  comment_level: {
    validator(rule, value) {
      return true;
    },
    message: "Please enter the minimum comment level.",
    trigger: "blur",
  },
  status: {
    validator(rule, value) {
      return true;
    },
    message: "Please enter the category status.",
    trigger: "blur",
  },
});

const RWCLevelOptions = [
  {
    label: "Guest",
    value: UserType.Guest,
  },
  {
    label: "General",
    value: UserType.General,
  },
  {
    label: "Administrator",
    value: UserType.Administrator,
  },
];

const CategoryStatusOptions = [
  {
    label: "Active",
    value: CategoryStatus.Active,
  },
  {
    label: "Archived",
    value: CategoryStatus.Archived,
  },
  {
    label: "Stopped",
    value: CategoryStatus.Stopped,
  },
];

const ContentTypeOptions = [
  {
    label: "Markdown",
    value: ContentType.Markdown,
  },
];

async function runCreate() {
  const group_ids = formValue.value.group_ids
    .replaceAll(" ", "")
    .split(",")
    .map((g) => Number.parseInt(g))
    .filter((g) => !isNaN(g));
  const moderator_ids = formValue.value.moderator_ids
    .replaceAll(" ", "")
    .split(",")
    .map((g) => Number.parseInt(g))
    .filter((g) => !isNaN(g));

  if (props.edit) {
    let { data: category, error } = await updateCategory(props.edit.id, {
      target: {
        ...formValue.value,
        total_post: 0,
        group_ids,
        moderator_ids,
      },
    });
    if (category.value) {
      message.success("Success update the category!");
      await router.push(`/category/${category.value.id}`);
    } else {
      const err = getApiDetailError(error.value);
      message.error(`(${err?.code}) Update failed. ${err?.msg}`);
    }
  } else {
    let { data: category, error } = await createCategory({
      target: {
        ...formValue.value,
        group_ids,
        moderator_ids,
      },
    });
    if (category.value) {
      message.success("Success create the category!");
      await router.push(`/category/${category.value.id}`);
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
        <n-input v-model:value="formValue.title" placeholder="Category title" />
      </n-form-item>
      <n-form-item label="Description" path="description">
        <MarkdownEditor
          v-model:value="formValue.description"
          placeholder="Your category description content."
        />
      </n-form-item>
      <n-form-item label="Read level" path="read_level">
        <n-select
          v-model:value="formValue.read_level"
          :options="RWCLevelOptions"
        />
      </n-form-item>
      <n-form-item label="Write level" path="write_level">
        <n-select
          v-model:value="formValue.write_level"
          :options="RWCLevelOptions"
        />
      </n-form-item>
      <n-form-item label="Comment level" path="comment_level">
        <n-select
          v-model:value="formValue.comment_level"
          :options="RWCLevelOptions"
        />
      </n-form-item>
      <n-form-item label="Status" path="status">
        <n-select
          v-model:value="formValue.status"
          :options="CategoryStatusOptions"
        />
      </n-form-item>
      <n-form-item label="Group ids" path="group_ids">
        <n-input
          v-model:value="formValue.group_ids"
          placeholder="Group ids, use `,` to split."
        />
      </n-form-item>
      <n-form-item label="Moderator ids" path="moderator_ids">
        <n-input
          v-model:value="formValue.moderator_ids"
          placeholder="Moderator ids, use `,` to split."
        />
      </n-form-item>
      <n-form-item>
        <n-button @click="runCreate"> {{ edit? "Update": "Create" }} </n-button>
      </n-form-item>
    </n-form>
  </n-card>
</template>
