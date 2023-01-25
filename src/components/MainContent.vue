<template>
  <div>
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

import {ApiResp, backApi, CurrentDbAndTable, emitter, TableData} from "../types/common";
import {ElMessage} from "element-plus";
import {ref} from "vue";
import TableContent from "./TableContent.vue";
import CustomSQL from "./CustomSQL.vue";

const activeTabName = ref('custom')

const defaultLimit = 100;

const currentLimit = ref(defaultLimit);

emitter.on('fetch_table_data', (current) => {
  fetchTableData(current as CurrentDbAndTable);
});

const fetchTableData = (currentMeta: CurrentDbAndTable) => {
  backApi("fetch_table_data", {
    dbPath: currentMeta.db,
    tableName: currentMeta.table,
    limit: currentLimit.value
  }, (resp) => {
    let r: ApiResp<TableData> = JSON.parse(resp as string);
    if (r.success) {
      emitter.emit('refresh_table_data', r.data);
      activeTabName.value = 'explore';
    } else {
      ElMessage.error(r.message);
    }
  });
}
</script>

<style lang="scss" scoped>

</style>
