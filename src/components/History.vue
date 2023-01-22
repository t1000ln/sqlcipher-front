<template>
  <div class="his_area">
    <div v-for="item in cmp_data.his_data">
      <el-tooltip :content="item.path" effect="light" placement="top-end">
        <span>{{ item.name }}</span>
      </el-tooltip>
      <span class="open-his-act" @click="refresh_db(item.path)">
      <el-icon>
        <Refresh/>
      </el-icon>
    </span>
    </div>
  </div>
</template>

<script lang="ts" name="History" setup>

import {onMounted, reactive} from "vue";
import {History} from "../types/history";
import {ApiResp, backApi, emitter} from "../types/common";
import {ObjectNames} from "../types/metas";

const cmp_data = reactive({
  his_data: [] as History[]
});

const refresh_db = (path: string) => {
  backApi("open_db", {dataPath: path}, (resp) => {
    let r: ApiResp<ObjectNames> = JSON.parse(resp as string);
    if (r.success) {
      emitter.emit('meta_objects_refreshed', r.data)
    }
  });
}

emitter.on('add_history_success', _ => {
  load_history();
})

const load_history = () => {
  backApi("load_history", {}, (resp) => {
    let r: ApiResp<History[]> = JSON.parse(resp as string);
    console.log(r);
    cmp_data.his_data = r.data;
  });
}


onMounted(() => {
  load_history();
})
</script>

<style scoped>

.his_area {
  position: relative;
  /*border: 1px solid lightgray;*/
}

.his_area div {
  padding: .5em;
}

.his_area div:hover {
  border-radius: 4px;
  background-color: lightskyblue;
}

.open-his-act {
  float: right;
  padding: .1em .5em;
}

.open-his-act:hover {
  cursor: pointer;
}


</style>
