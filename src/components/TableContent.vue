<template>
  <el-table :data="tableDataState.rows" border height="90vh" stripe style="width: 100%">
    <el-table-column v-for="(item, index) in tableDataState.cols" :key="index" :label="item" :prop="item"
                     align="center"></el-table-column>
  </el-table>
</template>

<script lang="ts" name="TableContent" setup>

import emitter, {TableData} from "../types/common";
import {reactive} from "vue";

const tableDataState = reactive({
  cols: [] as string[],
  rows: [] as object[]
});

emitter.on('refresh_table_data', (newData) => {
  console.log(newData);
  let nd = newData as TableData;
  if (nd.cols !== undefined) {
    tableDataState.cols = nd.cols;
  }
  if (nd.rows !== undefined) {
    tableDataState.rows = nd.rows;
  }
})
</script>

<style lang="scss" scoped>

</style>
