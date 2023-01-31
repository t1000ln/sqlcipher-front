<template>
  <div class="objects-area">
    <div v-for="item in obj_lists.table_names" class="table-name" @click="reloadTableData(item, false)">
      <span>{{ item }}</span>
      <el-tooltip :show-after="500" content="编辑表结构">
        <el-icon class="edit-icon" @click="alterTable(item)">
          <Edit></Edit>
        </el-icon>
      </el-tooltip>
    </div>
    <div v-for="item in obj_lists.view_names" class="table-name" @click="reloadTableData(item, true)">
      <span>{{ item }}</span>
    </div>
    <el-dialog v-model="showAlterTableDialog"></el-dialog>
  </div>
</template>

<script lang="ts" name="Objects" setup>
import emitter, {CurrentDbAndTable} from "../types/common";
import {reactive, ref} from "vue";
import {ObjectNames} from "../types/metas";

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

const alterTable = (tableName: string) => {
  showAlterTableDialog.value = true;
}

</script>

<style scoped>
.objects-area {
  height: 76%;
  overflow-y: scroll;
}

.table-name {
  width: 100%;
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
</style>
