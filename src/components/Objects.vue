<template>
  <div class="objects-area">
    <div v-for="item in obj_lists.table_names" class="table-name" @click="reloadTableData(item, false)">
      <span>{{ item }}</span>
    </div>
    <div v-for="item in obj_lists.view_names" class="table-name" @click="reloadTableData(item, true)">
      <span>{{ item }}</span>
    </div>
  </div>
</template>

<script lang="ts" name="Objects" setup>
import emitter, {CurrentDbAndTable} from "../types/common";
import {reactive} from "vue";
import {ObjectNames} from "../types/metas";

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

</script>

<style scoped>
.objects-area {
  height: 76%;
  overflow-y: scroll;
}

.table-name {
  width: 100%;
}

.table-name:hover {
  background-color: lightskyblue;
  border-radius: 4px;
  cursor: pointer;
}
</style>
