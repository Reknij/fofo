<script setup lang="ts">
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <FofoHeader class="z-[1]"></FofoHeader>
    <div class="bg-neutral-100 dark:bg-slate-800 flex-1">
      <div class="flex max-w-screen-xl mx-auto p-1 xl:p-2 xl:gap-1.5">
        <div class="flex-grow max-w-full">
          <slot></slot>
        </div>
        <SiderSpace class="hidden xl:block max-w-[340px] w-full"></SiderSpace>
      </div>
    </div>
    <FofoFooter></FofoFooter>
  </div>
</template>
