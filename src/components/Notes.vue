<template>
  <div ref="noteContent" class="note-content" contenteditable="true" @keydown.ctrl.s="saveNote">
  </div>

</template>

<script lang="ts" name="Notes" setup>

import {onMounted, ref} from "vue";
import {ApiResp, backApi} from "../types/common";
import {ElMessage} from "element-plus";
import {appLocalDataDir} from '@tauri-apps/api/path';

const noteContent = ref();


const saveNote = async () => {
  let appLocalDataDirPath = await appLocalDataDir();
  let note = noteContent.value.innerText;
  let params = {tempFilePath: appLocalDataDirPath, note: note}
  await backApi("save_temp_notes", params, (resp) => {
    let r: ApiResp = JSON.parse(resp as string);
    if (r.success) {
      ElMessage.success("临时笔记已暂存");
    }
  });
}

onMounted(() => {
  appLocalDataDir().then((appLocalDataDirPath) => {
    let params = {tempFilePath: appLocalDataDirPath};
    backApi("load_temp_notes", params, (resp) => {
      let r: ApiResp<string> = JSON.parse(resp as string);
      if (r.success) {
        noteContent.value.innerText = r.data;
      }
    });
  }).catch((e) => {
    console.error(e);
    ElMessage.error(e.message);
  })

})
</script>

<style scoped>
.note-content {
  height: 90vh;
  overflow: scroll;
  padding: .2em;
  background-color: lightyellow;
  border: 1px solid #a0b3d6;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1), 0 0 8px rgba(82, 168, 236, 0.6);
}
</style>
