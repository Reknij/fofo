<script setup lang="ts">
import {
  NLayout,
  NLayoutHeader,
  NLayoutFooter,
  NLayoutContent,
  NBackTop,
} from "naive-ui";

const { isMobileOrTablet } = useDevice();

const layoutStyle = {
  display: "grid",
  gridTemplateAreas: "'header' 'main' 'footer'",
  minHeight: "100vh",
  gridTemplateRows: "auto 1fr auto",
};
</script>

<template>
  <div>
    <n-layout :content-style="layoutStyle">
      <n-layout-header bordered>
        <div class="gridContainer">
          <div class="contentSide"></div>
          <div class="contentMain">
            <FofoHeader :vertical="isMobileOrTablet"></FofoHeader>
          </div>
          <div class="contentMainSide"></div>
          <div class="contentSide"></div>
        </div>
      </n-layout-header>
      <n-layout-content :content-style="{ padding: '6px' }">
        <div class="gridContainer">
          <div class="contentSide"></div>
          <div class="contentMain">
            <slot></slot>
          </div>
          <div class="contentMainSide">
            <SiderSpace></SiderSpace>
          </div>
          <div class="contentSide"></div>
        </div>
      </n-layout-content>
      <n-layout-footer class="layoutFooter" bordered>
        <FofoFooter></FofoFooter>
      </n-layout-footer>
    </n-layout>
    <n-back-top :right="15" />
  </div>
</template>

<style>
.gridContainer {
  display: grid;
  grid-template-columns: repeat(5, minmax(0px, 1fr));
  gap: 6px;
}

@media only screen and (max-width: 1023px) {
  .contentMain {
    grid-column: span 5 !important;
  }

  .contentMainSide {
    display: none;
  }

  .contentSide {
    display: none;
  }
}

@media only screen and (min-width: 1024px) {
  .contentMain {
    grid-column: span 2 !important;
  }

  .contentMainSide {
    grid-column: span 1 !important;
  }

  .contentSide {
    grid-column: span 1 !important;
  }
}
</style>
