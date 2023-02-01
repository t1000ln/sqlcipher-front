<template>
  <splitpanes :dbl-click-splitter="false" class="default-theme layout">
    <pane max-size="60" min-size="5" size="30">
      <splitpanes :dbl-click-splitter="false" horizontal>
        <pane max-size="5" min-size="5" size="5">
          <NewFile></NewFile>
        </pane>
        <pane size="20">
          <History></History>
        </pane>
        <pane size="75">
          <Objects></Objects>
        </pane>
      </splitpanes>
    </pane>
    <pane size="70">
      <MainContent></MainContent>
    </pane>
  </splitpanes>
</template>

<script lang="ts" setup>

import {Pane, Splitpanes} from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'</script>

<style lang="css">
.layout {
  height: 97vh;
  background-color: #a0cfff;
}
</style>
