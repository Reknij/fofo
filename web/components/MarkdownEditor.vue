<script setup lang="ts">
import "bytemd/dist/index.css";
import '~/styles/bytemd.css'
// @ts-ignore
import { Editor } from "@bytemd/vue-next";
import { bytemdPlugins } from "~/helper";

defineProps<{
  value: string;
  placeholder?: string;
}>();
defineEmits<{
  (e: "update:value", v: string): void;
}>();
</script>

<template>
  <Editor
    style="width: 100%"
    :value="value"
    mode="auto"
    :plugins="bytemdPlugins"
    :placeholder="placeholder"
    @change="(v: string) => $emit('update:value', v)"
  />
</template>
