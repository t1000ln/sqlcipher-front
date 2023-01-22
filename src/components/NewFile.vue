<template>
  <div class="new-path-icon" @click="open_dialog">
    <el-icon :size="20" color="green">
      <Plus/>
    </el-icon>
  </div>
</template>

<script lang="ts" name="NewFile" setup>
import {open} from '@tauri-apps/api/dialog';
import {ApiResp, backApi, emitter} from "../types/common";

const open_dialog = async () => {
  const selected = await open({
    multiple: false,
    filters: []
  });

  await backApi("add_history", {path: selected}, (resp) => {
    let r: ApiResp<null> = JSON.parse(resp as string);
    if (r.success) {
      emitter.emit('add_history_success');
    }
  });
}


</script>

<style scoped>
.new-path-icon {
  text-align: center;
  cursor: pointer;
}
</style>
