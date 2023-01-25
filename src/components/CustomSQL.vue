<template>
  <div>
    <div class="action-bar">
      <el-tooltip :show-after="1000" content="执行当前输入的SQL (Ctrl+Enter)" placement="top">
        <el-icon class="icons" @click="execSql">
          <CaretRight/>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="格式化输入的SQL (Ctrl+Shift+F)" placement="top-start">
        <el-icon class="icons" @click="formatSql">
          <Finished/>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="注释单行或多行 (Ctrl+b)" placement="top">
        <el-icon class="icons" @click="commentLine">
          <Expand></Expand>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="移除单行或多行前面的注释 (Alt+b)" placement="top">
        <el-icon class="icons" @click="uncommentLine">
          <Fold></Fold>
        </el-icon>
      </el-tooltip>

    </div>
    <div ref="sqlContent" class="sql-content" contenteditable="true" @keydown.ctrl.enter="execSql"
         @keydown.ctrl.shift.f="formatSql" @keydown.alt.b="uncommentLine"
         @keydown.ctrl.b="commentLine"
    ></div>
    <div v-show="showDataArea" ref="dataArea" class="result-content">
      <el-table v-show="showArrayTable" :data="dataState.arrayResult" :height="resultTableHeight" border
                class="result-table"
                stripe>
        <el-table-column v-for="item in dataState.arrayResultCols" :label="item" :prop="item"
                         align="center"></el-table-column>
      </el-table>
      <span v-show="showActionResult">{{ dataState.actionResult }}</span>
    </div>
  </div>

</template>

<script lang="ts" name="CustomSQL" setup>

import {reactive, ref} from "vue";
import {ApiResp, backApi, CurrentDbAndTable, emitter} from "../types/common";
import {ElMessage} from "element-plus";
import {format} from 'sql-formatter';

const sqlContent = ref();
const dataArea = ref();
const resultTableHeight = ref('40vh')
const showDataArea = ref(false);
const showArrayTable = ref(false);
const showActionResult = ref(false);
const pageCache = reactive({current: {} as CurrentDbAndTable});


const dataState = reactive({
  arrayResult: [] as any[],
  actionResult: '',
  arrayResultCols: new Set<string>(),
})

emitter.on('meta_objects_refreshed', (newCurrent) => {
  pageCache.current = newCurrent as CurrentDbAndTable;
});

declare type ExecParam = {
  dbPath: string,
  sql: string,
  key?: string
}

const execSql = () => {
  if (pageCache.current.db !== undefined) {
    /*
    重组SQL语句，忽略注释行，合并冗余的空白和换行符。
     */
    let normalizedSql = '';
    let origLines: string[] = sqlContent.value.innerText.trim().split('\n');
    for (let i = 0; i < origLines.length; i++) {
      if (!origLines[i].trim().startsWith('--')) {
        normalizedSql += origLines[i] + ' ';
      }
    }
    normalizedSql = normalizedSql.replaceAll(/\s+/g, ' ')

    /*
    组织调用参数。
     */
    let params: ExecParam = {
      dbPath: pageCache.current.db,
      sql: normalizedSql,
    };
    if (pageCache.current.key != undefined) {
      params.key = pageCache.current.key;
    }

    backApi("exec_custom_sql", params, (resp) => {
      let r: ApiResp = JSON.parse(resp as string);
      if (r.success) {
        showDataArea.value = true;

        if (r.data instanceof Array) {
          // 遍历结果集，检出所有的字段名
          dataState.arrayResultCols.clear();
          for (let i = 0; i < r.data.length; i++) {
            for (let p in r.data[i]) {
              dataState.arrayResultCols.add(p);
            }
          }

          dataState.arrayResult = r.data;
          showArrayTable.value = true;
          showActionResult.value = false;
        } else if (r.data instanceof Object) {
          dataState.actionResult = '本次操作更新 ' + r.data.rows_affected + ' 行数据';
          showArrayTable.value = false;
          showActionResult.value = true;
        }

      } else {
        ElMessage.error(r.message);
      }
    });
  } else {
    ElMessage.error("请先选择数据库");
  }
}


const formatSql = () => {
  sqlContent.value.innerText = format(sqlContent.value.innerText.trim(), {
    language: 'sqlite',
    tabWidth: 2,
    keywordCase: 'upper',
    linesBetweenQueries: 1
  })
}

/**
 * 注释光标所在行或选中的多个行。
 * 注意：快捷键冲突-_-!
 * 注释和反注释两个快捷键不能使用`Ctrl+b`和`Ctrl+Shift+b`，只能使用`Ctrl+b`和`Alt+b`这样的组合。
 * 目前原因不明。
 */
const commentLine = () => {
  let sqlDiv = sqlContent.value;

  /*
  根据当前光标所在位置，或所选区域，计算出将要添加注释字符的行。
   */
  let fromLineNum = -1, toLineNum = -1;
  let s = window.getSelection();
  if (s !== null) {
    let y1 = -1, y2 = -1;
    for (let i = 0; i < sqlDiv.childNodes.length; i++) {
      if (s.anchorNode.textContent == sqlDiv.childNodes[i].innerText) {
        y1 = i;
      } else if (s.focusNode.textContent == sqlDiv.childNodes[i].innerText) {
        y2 = i;
      }
    }
    fromLineNum = Math.min(y1, y2);
    toLineNum = Math.max(y1, y2);

    s.removeAllRanges();
  }

  /*
  在目标行开头处添加"--"字符串。
   */
  if (toLineNum > -1) {
    if (fromLineNum > -1) {
      for (let i = fromLineNum; i <= toLineNum; i++) {
        sqlDiv.childNodes[i].innerText = '--' + sqlDiv.childNodes[i].innerText;
      }
    } else {
      sqlDiv.childNodes[toLineNum].innerText = '--' + sqlDiv.childNodes[toLineNum].innerText;
    }
  }
}

/**
 * 移除光标所在行或选中的多个行前面的注释字符串。
 * 注意：快捷键冲突-_-!
 * 注释和反注释两个快捷键不能使用`Ctrl+b`和`Ctrl+Shift+b`，只能使用`Ctrl+b`和`Alt+b`这样的组合。
 * 目前原因不明。
 */
const uncommentLine = () => {
  let sqlDiv = sqlContent.value;
  /*
  根据当前光标所在位置，或所选区域，计算出将要添加注释字符的行。
   */
  let fromLineNum = -1, toLineNum = -1;
  let s = window.getSelection();
  if (s !== null) {
    let y1 = -1, y2 = -1;
    for (let i = 0; i < sqlDiv.childNodes.length; i++) {
      if (s.anchorNode.textContent == sqlDiv.childNodes[i].innerText) {
        y1 = i;
      } else if (s.focusNode.textContent == sqlDiv.childNodes[i].innerText) {
        y2 = i;
      }
    }
    fromLineNum = Math.min(y1, y2);
    toLineNum = Math.max(y1, y2);

    s.removeAllRanges();
  }

  /*
  移除目标行开头处添加"--"字符串。
   */
  if (toLineNum > -1) {
    if (fromLineNum > -1) {
      for (let i = fromLineNum; i <= toLineNum; i++) {
        if (sqlDiv.childNodes[i].innerText.trim().startsWith('--')) {
          sqlDiv.childNodes[i].innerText = sqlDiv.childNodes[i].innerText.replace(/\s*--/, '');
        }
      }
    } else {
      if (sqlDiv.childNodes[toLineNum].innerText.trim().startsWith('--')) {
        sqlDiv.childNodes[toLineNum].innerText = sqlDiv.childNodes[toLineNum].innerText.replace(/\s*--/, '');
      }
    }
  }
}
</script>

<style scoped>
.action-bar {
  margin-bottom: .2em;
}

.icons {
  line-height: 2em;
  font-size: 1.5em;
  margin-right: .5em;
  margin-left: .5em;
  padding-left: .5em;
  padding-right: .5em;
}

.icons:hover {
  cursor: pointer;
  color: limegreen;
}

.sql-content {
  font-family: monospace;
  font-size: 1.1em;
  height: 40vh;
  padding: .2em;
  overflow: scroll;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1), 0 0 8px rgba(82, 168, 236, 0.6);
}

.result-content {
  font-family: monospace;
  margin-top: 1em;
}

.result-table {
  width: 100%;
}
</style>
