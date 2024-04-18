<script setup lang="ts">
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <FofoHeader></FofoHeader>
    <div class="bg-neutral-300 dark:bg-slate-800 flex-1">
      <div class="flex max-w-screen-xl mx-auto p-1 xl:p-2 xl:gap-x-2">
        <div class="flex-grow max-w-full">
          <slot></slot>
        </div>
        <SiderSpace class="hidden xl:block max-w-[256px] w-full"></SiderSpace>
      </div>
    </div>
    <FofoFooter></FofoFooter>
  </div>
</template>
