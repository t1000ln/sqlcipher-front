<template>
  <div class="objects-area">
    <div v-for="item in obj_lists.table_names">
      {{ item }}
    </div>
  </div>
</template>

<script lang="ts" name="Objects" setup>
import emitter from "../types/common";
import {reactive} from "vue";
import {ObjectNames} from "../types/metas";

const obj_lists = reactive<ObjectNames>({
  table_names: [],
  view_names: []
})

emitter.on('meta_objects_refreshed', (new_objs) => {
  console.log(new_objs);
  obj_lists.table_names = (new_objs as ObjectNames).table_names;
  obj_lists.view_names = (new_objs as ObjectNames).view_names;
});

</script>

<style scoped>
.objects-area {
  height: 76%;
  overflow-y: scroll;
}
</style>
