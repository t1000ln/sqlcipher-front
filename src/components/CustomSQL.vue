<template>
  <div>
    <div class="action-bar">
      <el-tooltip content="执行当前输入的SQL (Ctrl+Enter)" placement="top">
        <el-icon class="icons" @click="execSql">
          <CaretRight/>
        </el-icon>
      </el-tooltip>

      <el-tooltip content="格式化输入的SQL (Ctrl+Shift+F)" placement="top-start">
        <el-icon class="icons" @click="formatSql">
          <Finished/>
        </el-icon>
      </el-tooltip>

    </div>
    <div ref="sqlContent" class="sql-content" contenteditable="true" @keydown.ctrl.enter="execSql"
         @keydown.ctrl.shift.f="formatSql"
    ></div>
  </div>

</template>

<script lang="ts" name="CustomSQL" setup>

import {reactive, ref} from "vue";
import {ApiResp, backApi, CurrentDbAndTable, emitter} from "../types/common";
import {ElMessage} from "element-plus";
import {format} from 'sql-formatter';

const sqlContent = ref();
const pageCache = reactive({current: {} as CurrentDbAndTable});

emitter.on('meta_objects_refreshed', (newCurrent) => {
  pageCache.current = newCurrent as CurrentDbAndTable;
});

declare type ExecParam = {
  dbPath: string,
  sql: string,
  key?: string
}

const execSql = () => {
  let params: ExecParam = {
    dbPath: pageCache.current.db,
    sql: sqlContent.value.innerText.trim(),
  };
  if (pageCache.current.key != undefined) {
    params.key = pageCache.current.key;
  }

  backApi("exec_custom_sql", params, (resp) => {
    let r: ApiResp = JSON.parse(resp as string);
    if (r.success) {
      console.log(r.data)
    } else {
      ElMessage.error(r.message);
    }
  });
}


const formatSql = () => {
  sqlContent.value.innerText = format(sqlContent.value.innerText.trim(), {
    language: 'sqlite',
    tabWidth: 2,
    keywordCase: 'upper',
    linesBetweenQueries: 1
  })
}
</script>

<style scoped>
.action-bar {
  margin-bottom: .2em;
}

.icons {
  line-height: 2em;
  font-size: 1.5em;
  margin-right: .5em;
  margin-left: .5em;
  padding-left: .5em;
  padding-right: .5em;
}

.icons:hover {
  cursor: pointer;
  color: limegreen;
}

.sql-content {
  font-family: monospace;
  font-size: 1.1em;
  height: 50vh;
  padding: .2em;
  overflow: scroll;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1), 0 0 8px rgba(82, 168, 236, 0.6);
}
</style>
