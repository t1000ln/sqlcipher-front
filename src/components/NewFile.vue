<template>
  <div class="new-path-bar">
    <el-tooltip :show-after="500" content="打开已有的sqlite/sqlcipher文件">
      <el-icon class="new-path-icon" @click="open_exist_file_dialog">
        <FolderOpened/>
      </el-icon>
    </el-tooltip>

    <el-tooltip :show-after="500" content="新建sqlite/sqlcipher文件">
      <el-icon class="new-path-icon" @click="create_new_file_dialog">
        <DocumentAdd/>
      </el-icon>
    </el-tooltip>
    <el-dialog v-model="showKeyDialog">
      <el-input v-model="params.key" show-password type="password" @keyup.enter="confirmOpen"></el-input>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="ignoreKey">不设置</el-button>
          <el-button type="primary" @click="confirmOpen">设置</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts" name="NewFile" setup>
import {confirm, open, save} from '@tauri-apps/api/dialog';
import {ApiResp, backApi, emitter} from "../types/common";
import {reactive, ref} from "vue";

const showKeyDialog = ref(false);
const params = reactive({
  path: '',
  key: '',
});
const ignoreKey = async () => {
  params.key = '';
  await confirmOpen();
}

const confirmOpen = async () => {
  showKeyDialog.value = false;
  await backApi("add_history", params, (resp) => {
    params.key = '';
    params.path = '';
    let r: ApiResp<null> = JSON.parse(resp as string);
    if (r.success) {
      emitter.emit('add_history_success');
    }
  });
}

const open_exist_file_dialog = async () => {
  const selected: string | string[] | null = await open({
    multiple: false,
    filters: []
  });

  if (selected !== null) {
    params.path = selected as string;
    showKeyDialog.value = true;
  }
}

const create_new_file_dialog = async () => {
  const filePath: string | null = await save({
    filters: [{
      name: 'sqlite/sqlcipher',
      extensions: ['db', 'sqlite', '*']
    }]
  });

  if (filePath !== null) {
    params.path = filePath;
    let cfm = await confirm('新建数据库将保存为 ' + filePath + '，继续吗？');
    if (cfm) {
      showKeyDialog.value = true;
    } else {
      params.key = '';
      params.path = '';
    }
  }
}

</script>

<style scoped>
.new-path-bar {
  text-align: center;
}

.new-path-icon {
  margin-left: 1em;
  margin-right: 1em;
  font-size: 1.5em;
}

.new-path-icon:hover {
  cursor: pointer;
  color: chocolate;
  filter: drop-shadow(0 0 2em #747bff);
}

.dialog-footer:first-child {
  margin-right: 1em;
}
</style>
