<template>
  <div class="his_area">
    <div v-for="(item, index) in cmp_data.his_data" :key="index">
      <div>
        <el-tooltip :show-after="500" content="查看数据库" placement="top">
          <el-icon class="open-his-act" @click="refresh_db(item)">
            <ZoomIn/>
          </el-icon>
        </el-tooltip>
        <el-icon v-if="item.key" class="security-icon">
          <Key></Key>
        </el-icon>
        <el-icon v-else class="security-icon">
          <View></View>
        </el-icon>
        <span>{{ item.name }}</span>
        <el-tooltip :content="item.path" :show-after="500" placement="top">
          <el-icon class="path-icon">
            <Location></Location>
          </el-icon>
        </el-tooltip>
      </div>
      <span class="delete-icon">
        <el-tooltip :show-after="500" content="移除缓存信息，不会删除数据库文件" placement="top">
          <el-icon @click="deleteEntry(item.path, index)">
            <Delete/>
          </el-icon>
        </el-tooltip>
      </span>
    </div>
  </div>
</template>

<script lang="ts" name="History" setup>

import {onMounted, reactive} from "vue";
import {History} from "../types/history";
import {ApiResp, backApi, CurrentDbAndTable, emitter} from "../types/common";
import {ObjectNames} from "../types/metas";
import {ElMessage} from "element-plus";
import {confirm} from '@tauri-apps/api/dialog';

const cmp_data = reactive({
  his_data: [] as History[]
});

const refresh_db = async (item: History) => {
  let params: { [k: string]: string } = {'dbPath': item.path};
  if (item.key) {
    params['key'] = item.key;
  }
  await backApi("open_db", params, (resp) => {
    let r: ApiResp<ObjectNames> = JSON.parse(resp as string);
    if (r.success) {
      let current: CurrentDbAndTable = {db: item.path, data: r.data, key: item.key}
      emitter.emit('meta_objects_refreshed', current)
    } else {
      ElMessage.error(r.message);
    }
  });
}

emitter.on('add_history_success', _ => {
  load_history();
})

const load_history = () => {
  backApi("load_history", {}, (resp) => {
    let r: ApiResp<History[]> = JSON.parse(resp as string);
    cmp_data.his_data = r.data;
  });
}

const deleteEntry = async (path: string, index: number) => {
  let cfm = await confirm('要取消<' + path + '>缓存吗？该操作不会删除数据库文件。');
  if (cfm) {
    await backApi("remove_history_entry", {index: index}, (resp) => {
      let r: ApiResp<null> = JSON.parse(resp as string);
      if (r.success) {
        load_history();
        emitter.emit('remove_history_success');
      }
    });
  }
}
onMounted(() => {
  load_history();
})
</script>

<style scoped>

.his_area {
  /*position: relative;*/
  /*border: 1px solid lightgray;*/
}

.his_area div {
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.his_area div:hover {
  border-radius: 4px;
  background-color: lightskyblue;
}

.delete-icon {
  float: right;
  padding: .1em .5em;
}

.delete-icon:hover {
  cursor: pointer;
}

.open-his-act {
  margin-right: .2em;
}

.open-his-act:hover {
  cursor: pointer;
  color: purple;
  font-size: 1.1em;
}

.path-icon {
  margin-left: .5em;
}

.security-icon {
  color: gray;
  margin-right: .1em;
}
</style>
