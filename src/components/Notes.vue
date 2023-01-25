<template>
  <el-input v-model="noteContent" :autosize="noteSize" class="note-content"
            placeholder="在这里临时记录一些信息"
            type="textarea"></el-input>

</template>

<script lang="ts" name="Notes" setup>

import {reactive, ref} from "vue";

const noteContent = ref('');
const noteSize = reactive({minRows: 23, maxRows: 80})
</script>

<style scoped>
.note-content {
  height: 100%;
}
</style>
