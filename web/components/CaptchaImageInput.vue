<script setup lang="ts">
import { getVerification } from "~/api/verification";
import { getApiDetailError } from "~/helper";

defineProps<{
  modelValue: string;
  placeholder?: string;
}>()
defineEmits<{
  (e: 'update:modelValue', v: string): void,
}>();

const {
  data: verification,
  error,
  refresh: refreshVerification,
} = await getVerification();
function getErrorText() {
  if (error.value) {
    const err = getApiDetailError(error.value);
    return err?.msg ?? "Unknown error.";
  }

  return "No error.";
}
defineExpose({
  verification,
  refreshVerification,
});
</script>

<template>
  <lazy-client-only>
    <div class="flex flex-col justify-center space-y-2">
      <UButtonGroup>
        <img v-if="verification?.secret_key_picture_url"
          :src="`${verification!.secret_key_picture_url}?t=${Date.now()}`" />
        <span v-else>
          {{ getErrorText() }}
        </span>
        <UButton class="rounded-l-none" tabindex="-1" @click="refreshVerification()" color="gray"
          icon="i-heroicons-arrow-path" />
      </UButtonGroup>
      <UInput class="max-w-36" :model-value="value" @update:model-value="(v: string) => $emit('update:modelValue', v)"
        :placeholder="placeholder ?? 'Enter the captcha.'" />
    </div>
  </lazy-client-only>
</template>
