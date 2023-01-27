<template>
  <div class="objects-area">
    <div v-for="item in obj_lists.table_names">
      <span class="table-name" @click="reloadTableData(item)">{{ item }}</span>
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

const reloadTableData = (table_name: string) => {
  pageCache.current.table = table_name;
  emitter.emit('fetch_table_data_evt', pageCache.current);
}

</script>

<style scoped>
.objects-area {
  height: 76%;
  overflow-y: scroll;
}

.table-name:hover {
  background-color: lightskyblue;
  border-radius: 4px;
}
</style>
