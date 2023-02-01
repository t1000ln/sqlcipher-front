<template>
  <el-dialog v-model="showEditDDLDialog"></el-dialog>
</template>

<script lang="ts" name="TableAlter" setup>

import emitter from "../types/common";
import {ref} from "vue";

const showEditDDLDialog = ref(false);

emitter.on('show-edit-ddl-dialog', (tableName) => {
  console.log(tableName);
  showEditDDLDialog.value = true;
})
</script>

<style lang="scss" scoped>

</style>
