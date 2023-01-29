<template>
  <div ref="noteContent" class="note-content" contenteditable="true"></div>

</template>

<script lang="ts" name="Notes" setup>

import {ref} from "vue";

const noteContent = ref('');
</script>

<style scoped>
.note-content {
  height: 90vh;
  overflow: scroll;
  padding: .2em;
  border: 1px solid #a0b3d6;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1), 0 0 8px rgba(82, 168, 236, 0.6);
}
</style>
