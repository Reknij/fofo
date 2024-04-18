<script setup lang="ts">
import { NImage, NButton } from "naive-ui";
import { getVerification } from "~/api/verification";
import { getApiDetailError } from "~/helper";

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
  <div>
    <lazy-client-only>
      <n-image
        class="clickable"
        v-if="verification?.secret_key_picture_url"
        @click="refreshVerification()"
        preview-disabled
        :src="`${verification!.secret_key_picture_url}?t=${Date.now()}`"
      />
      <span v-else>
        {{ getErrorText() }}
        <n-button text type="info" @click="refreshVerification()"
          >Click to Refresh</n-button
        >
      </span>
    </lazy-client-only>
  </div>
</template>
