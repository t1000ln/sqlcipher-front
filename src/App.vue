<template>
  <div ref="outerContainer" class="layout">
    <div ref="leftArea" class="left-area">
      <div class="his-block">
        <NewFile></NewFile>
        <hr class="vertical-splitter">
        <History></History>
      </div>
      <hr class="vertical-splitter">
      <Objects></Objects>
    </div>
    <div ref="rightArea" class="right-area">
      <MainContent></MainContent>
    </div>
    <div ref="verticalLine" class="vertical-line" @mousedown="mouseDown"></div>
  </div>
</template>

<script lang="ts" setup>

import {ref} from "vue";
import History from "./components/History.vue";
import NewFile from "./components/NewFile.vue";
import Objects from "./components/Objects.vue";
import MainContent from "./components/MainContent.vue";

const outerContainer = ref();
const verticalLine = ref();
const leftArea = ref();
const rightArea = ref();

const leftWidth = 54;
const rightToLeftGap = 4;
const lineWidth = 6;
const splitMinLeft = 56;
const splitMaxLeft = 380;
const leftLineGap = splitMinLeft - leftWidth;

const mouseDown = (e: MouseEvent) => {
  let disX = e.clientX;
  verticalLine.value.left = verticalLine.value.offsetLeft;

  outerContainer.value.onmousemove = function (e2: MouseEvent) {
    let moveX = e2.clientX - disX;
    let iT = verticalLine.value.left + moveX;
    let maxT = outerContainer.value.clientWidth - verticalLine.value.offsetWidth;

    iT < 0 && (iT = 0);
    iT > maxT && (iT = maxT);

    if (iT <= splitMinLeft || iT >= splitMaxLeft) {
      return false;
    }

    let oLeftWidth = iT - leftLineGap;
    let oRightMarginLeft = oLeftWidth + lineWidth + rightToLeftGap;

    verticalLine.value.style.left = `${iT}px`;
    leftArea.value.style.width = `${oLeftWidth}px`;
    rightArea.value.style.marginLeft = `${oRightMarginLeft}px`;

    return false;
  }

  outerContainer.value.onmouseup = function () {
    outerContainer.value.onmousemove = null;
    outerContainer.value.onmouseup = null;
  }
}


</script>

<style scoped>
.layout {
  /*border: 1px solid #24c8db;*/
  height: 93vh;
  position: relative;
  margin: 20px auto;
  box-sizing: border-box;
}

.left-area {
  width: 200px;
  position: absolute;
  top: 0;
  bottom: 0;
  height: 100%;
}

.his-block {
  height: 24%;
  overflow-y: scroll;
}

.right-area {
  margin-left: 210px;
  background-color: pink;
  height: 95vh;
}

.vertical-line {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 202px;
  width: 6px;
  background-color: #e7e7e7;
  box-shadow: 0 0 8px #ccc;
  cursor: col-resize;
  text-align: center;
  line-height: 95vh;
}

hr.vertical-splitter {
  border: 0;
  height: 1px;
  background-color: #333;
  background-image: linear-gradient(to right, #ccc, #333, #ccc);
}
</style>
