<template>
  <div class="main-content">
    <el-tabs v-model="activeTabName" type="border-card">
      <el-tab-pane label="浏览数据" name="explore">
        <TableContent></TableContent>
      </el-tab-pane>
      <el-tab-pane label="自定义SQL" name="custom">
        <CustomSQL></CustomSQL>
      </el-tab-pane>
      <el-tab-pane label="备忘" name="notes">
        <Notes></Notes>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script lang="ts" name="MainContent" setup>

import {emitter} from "../types/common";
import {ref} from "vue";
import TableContent from "./TableContent.vue";
import CustomSQL from "./CustomSQL.vue";

const activeTabName = ref('custom')

emitter.on('fetch_table_data_evt', (current) => {
  activeTabName.value = 'explore';
});
</script>

<style scoped>
.main-content {
  height: 94vh;
}
</style>
