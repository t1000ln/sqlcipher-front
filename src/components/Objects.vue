<template>
  <div class="objects-area">
    <div v-for="item in obj_lists.table_names" class="table-name" @click="reloadTableData(item, false)">
      <span>{{ item }}</span>
      <el-tooltip :show-after="500" content="修改表结构">
        <el-icon class="edit-icon" @click="editDDL(item)">
          <Edit></Edit>
        </el-icon>
      </el-tooltip>
      <el-tooltip :show-after="500" content="查看建表语句">
        <el-icon class="edit-icon" @click="showDDL(item)">
          <Memo></Memo>
        </el-icon>
      </el-tooltip>
    </div>
    <div v-for="item in obj_lists.view_names" class="table-name" @click="reloadTableData(item, true)">
      <span>{{ item }}</span>
      <el-tooltip :show-after="500" content="查看建表语句">
        <el-icon class="edit-icon" @click="showDDL(item)">
          <Memo></Memo>
        </el-icon>
      </el-tooltip>
    </div>
    <el-dialog v-model="showAlterTableDialog">
      <div>
        <div style="position: relative;">
          <pre ref="ddlPre" class="ddl-pre"></pre>
          <el-icon class="copy-icon" @click="copyDdl">
            <CopyDocument></CopyDocument>
          </el-icon>
        </div>
      </div>
    </el-dialog>
    <TableAlter></TableAlter>
  </div>
</template>

<script lang="ts" name="Objects" setup>


import emitter, {ApiResp, backApi, CurrentDbAndTable} from "../types/common";
import {reactive, ref} from "vue";
import {ObjectNames} from "../types/metas";
import {writeText} from "@tauri-apps/api/clipboard";
import {ElMessage} from "element-plus";
import highlight from "../sql-color";


const showAlterTableDialog = ref(false);

const pageCache = reactive({current: {} as CurrentDbAndTable});

const obj_lists = reactive<ObjectNames>({
  table_names: [],
  view_names: []
})

emitter.on('meta_objects_refreshed', (newCurrent) => {
  pageCache.current = newCurrent as CurrentDbAndTable;
  let data = pageCache.current.data as ObjectNames;
  obj_lists.table_names = data.table_names;
  obj_lists.view_names = data.view_names;
});

emitter.on('remove_history_success', () => {
  obj_lists.table_names = [];
  obj_lists.view_names = [];
  pageCache.current = {} as CurrentDbAndTable;
});

const reloadTableData = (table_name: string, isView: boolean) => {
  pageCache.current.table = table_name;
  pageCache.current.isView = isView;
  emitter.emit('fetch_table_data_evt', pageCache.current);
}

const createTableSql = ref('');
const ddlPre = ref();
const showDDL = async (tableName: string) => {
  showAlterTableDialog.value = true;
  let params: { [keysOf: string]: string | undefined } = {
    dbPath: pageCache.current.db,
    tableName: tableName,
  };
  if (pageCache.current.key) {
    params.key = pageCache.current.key;
  }
  await backApi("get_table_sql", params, (resp) => {
    let r: ApiResp<string> = JSON.parse(resp as string);
    if (r.success) {
      createTableSql.value = r.data;
      let p = highlight(r.data, {html: true});
      ddlPre.value.innerHTML = p;
    }
  });
}

const editDDL = async (tableName: string) => {
  emitter.emit('show-edit-ddl-dialog', tableName);
}

const copyDdl = async () => {
  await writeText(createTableSql.value);
  ElMessage.info({
    message: '已复制DDL语句',
    grouping: true,
    type: 'info'
  })
}
</script>

<style scoped>
.objects-area {
  /*height: 76%;*/
  /*overflow-y: scroll;*/
}

.table-name {
  padding: 0 0 .2em .2em;
  width: 98%;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.table-name:hover {
  background-color: lightskyblue;
  border-radius: 4px;
  cursor: pointer;
}

.edit-icon {
  margin-right: .5em;
}

.copy-icon {
  position: absolute;
  right: .3em;
  top: .3em;
  cursor: pointer;
  -webkit-user-select: none;
  user-select: none;
  line-height: 1;
  color: hsla(0, 0%, 54.9%, .8);
  transition: color .1s;
}

.ddl-pre {
  font-family: monospace;
  color: #0f0f0f;
  /*font-weight: bold;*/
  background-color: #f5f5f5;
  border: 1px solid lightgray;
  border-radius: 4px;
  padding: .5em;
  white-space: pre-wrap;
}
</style>
