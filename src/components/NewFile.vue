<template>
  <div class="new-path-icon" @click="open_dialog">
    <el-icon :size="20">
      <DocumentAdd/>
    </el-icon>
  </div>
</template>

<script lang="ts" name="NewFile" setup>
import {open} from '@tauri-apps/api/dialog';
import {ApiResp, backApi, emitter} from "../types/common";
import {ElMessageBox} from "element-plus";

const open_dialog = async () => {
  const selected = await open({
    multiple: false,
    filters: []
  });

  if (selected !== null) {
    let params: { [key: string]: string } = {'path': selected as string};
    await ElMessageBox.prompt('请输入密钥字符串，若未加密则取消即可', '提示').then(({value}) => {
      params['key'] = value;
    }).catch(() => {
    })
    await backApi("add_history", params, (resp) => {
      let r: ApiResp<null> = JSON.parse(resp as string);
      if (r.success) {
        emitter.emit('add_history_success');
      }
    });
  }
}


</script>

<style scoped>
.new-path-icon {
  text-align: center;
  cursor: pointer;
}

.new-path-icon:hover {
  color: limegreen;
}
</style>
