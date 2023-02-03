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
      <el-tooltip :show-after="1000" content="注释单行或多行 (Ctrl+B)" placement="top">
        <el-icon class="icons" @click="commentLine">
          <Expand></Expand>
        </el-icon>
      </el-tooltip>
      <el-tooltip :show-after="1000" content="移除单行或多行前面的注释 (Ctrl+Shift+B)" placement="top">
        <el-icon class="icons" @click="uncommentLine">
          <Fold></Fold>
        </el-icon>
      </el-tooltip>
    </div>
    <splitpanes :dbl-click-splitter="false" class="custom-area" horizontal>
      <pane class="sql-content-pane" max-size="80" min-size="5" size="40">
        <!--        <div ref="sqlContent" class="sql-content" contenteditable="true"-->
        <!--             @mouseup="rememberSelection"-->
        <!--             @keydown.ctrl.enter="execSql" @keydown.ctrl.shift.f="formatSql"-->
        <!--             @keydown.ctrl.shift.b="uncommentLine" @keydown.ctrl.b="commentLine"-->
        <!--        ></div>-->
        <!--        <splitpanes :dbl-click-splitter="false">-->
        <!--          <pane max-size="90" min-size="10" size="50">-->
        <!--            <Toolbar :default-config="toolbarConfig" :editor="editorRef"></Toolbar>-->
        <!--            <Editor v-model="sqlHtml" :default-config="editorConfig" mode="simple"-->
        <!--                    @onChange="handleChange" @onCreated="handleCreated"-->
        <!--                    @keydown.ctrl.enter="execCurrentSql"></Editor>-->
        <!--          </pane>-->
        <!--          <pane>-->
        <!--            <highlightjs ref="hlSqlBlock" :code="sqlText" class="echo-sql-pre"></highlightjs>-->
        <!--          </pane>-->
        <!--        </splitpanes>-->
        <Toolbar :default-config="toolbarConfig" :editor="editorRef"></Toolbar>
        <Editor v-model="sqlHtml" :default-config="editorConfig" mode="simple"
                @onChange="handleChange" @onCreated="handleCreated"
                @keydown.ctrl.enter="execCurrentSql"></Editor>
      </pane>
      <pane size="60">
        <div class="last-db-path">{{ lastExecOnDbPath }}</div>
        <div v-show="showDataArea" ref="dataArea" class="result-content">
          <el-table v-show="showArrayTable" :data="dataState.arrayResult" :height="resultTableHeight" border
                    class="result-table"
                    stripe>
            <el-table-column v-for="item in dataState.arrayResultCols" :label="item" :prop="item"
                             align="center"></el-table-column>
          </el-table>
          <span v-show="showActionResult">{{ dataState.actionResult }}</span>
        </div>
      </pane>
    </splitpanes>
  </div>
</template>

<script lang="ts" name="CustomSQL" setup>

import {onBeforeUnmount, reactive, ref, shallowRef} from "vue";
import {ApiResp, backApi, CurrentDbAndTable, emitter, ExecParam, SelectedLines, SqlSelection} from "../types/common";
import {ElMessage} from "element-plus";
import {format} from 'sql-formatter';
import {Pane, Splitpanes} from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'
import '@wangeditor/editor/dist/css/style.css'
import {Editor, Toolbar} from '@wangeditor/editor-for-vue'
import {IEditorConfig, IToolbarConfig} from "@wangeditor/editor";
import {AlertType} from "@wangeditor/core/dist/core/src/config/interface"
import {IDomEditor} from "@wangeditor/core/dist/core/src/editor/interface";
import _ from 'lodash'

const sqlContent = ref();
const dataArea = ref();
const resultTableHeight = ref('40vh')
const showDataArea = ref(false);
const showArrayTable = ref(false);
const showActionResult = ref(false);
const pageCache = reactive({current: {} as CurrentDbAndTable});
const sqlSelection = reactive<SqlSelection>({
  fromLineNum: -1,
  toLineNum: -1,
});

const hlSqlBlock = ref();
const editorRef = shallowRef();
const sqlHtml = ref('');
const sqlText = ref('');
const toolbarConfig: Partial<IToolbarConfig> = {
  toolbarKeys: ['fontSize', 'fontFamily', 'lineHeight', 'codeBlock']
};
const editorConfig: Partial<IEditorConfig> = {
  placeholder: '请输入SQL语句',
  customAlert: function (info: string, type: AlertType): void {
    throw new Error('Function not implemented.');
  },
  scroll: true,
  readOnly: false,
  autoFocus: false,
  MENU_CONF: {
    'codeSelectLang': {
      'codeLangs': [{text: 'SQL', value: 'sql'}]
    }
  }
};
const handleCreated = (editor: IDomEditor) => {
  editorRef.value = editor // 记录 editor 实例，重要！
}
const customPaste = (editor: IDomEditor, event: ClipboardEvent, callback: Function) => {
  if (event.clipboardData) {
    // const html = event.clipboardData.getData('text/html') // 获取粘贴的 html
    const text = event.clipboardData.getData('text/plain') // 获取粘贴的纯文本
    // const rtf = event.clipboardData.getData('text/rtf') // 获取 rtf 数据（如从 word wsp 复制粘贴）

    // 自定义插入内容
    editor.insertText(text)

    // 返回 false ，阻止默认粘贴行为
    event.preventDefault()
    callback(false) // 返回值（注意，vue 事件的返回值，不能用 return）

    // 返回 true ，继续默认的粘贴行为
    // callback(true)
  }
}

const handleChange = (editor: IDomEditor) => {
  onChange(editor);
}

const onChange = _.debounce((editor: IDomEditor) => {
  sqlText.value = editor.getText();
}, 50);


const execCurrentSql = () => {
  let editor: IDomEditor = editorRef.value as IDomEditor;
  let selection = editor.selection;
  if (selection?.anchor !== undefined) {
    let fromLineNum: number | undefined = selection.anchor.path[0];
    let sql = '';
    if (_.isEqual(selection.focus, selection.anchor)) {
      // 未选择任何文本
      // 计算光标的绝对字符位置
      let text = editor.getText();
      console.log(text)

      let ahead = 0;
      // 计算光标之前所有行的字符串长度，不包括光标所在行
      for (let i = 0, lastRetPos = 0; i < fromLineNum; i++) {
        lastRetPos = text.indexOf('\n', ahead);
        if (lastRetPos != -1) {
          ahead = lastRetPos + 1;
        }
      }
      ahead += selection.anchor.offset;
      
      // 处理从开始到光标处的字符串
      let realStart = 0, realEnd = text.length;
      let preHalf = text.substring(0, ahead).trimEnd();
      if (selection.anchor.offset != 0 && preHalf.charAt(preHalf.length - 1) == ';') {
        preHalf = preHalf.substring(0, preHalf.length - 1);
        /*
         光标在分号后面，最近的SQL语句起始位置在最近的两个分号之间，
         或从0开始到前面最近一个分号处。
         */
        let lastSemiPos = preHalf.lastIndexOf(';');
        if (lastSemiPos != -1) {
          realStart = lastSemiPos + 1;
        }
        sql = preHalf.substring(realStart).trim();
      } else {
        // 光标在语句中间
        preHalf = text.substring(0, ahead);
        let lastSemiPos = preHalf.lastIndexOf(';');
        if (lastSemiPos != -1) {
          realStart = lastSemiPos + 1;
        }
        let firstSemiPos = text.indexOf(';', ahead);
        if (firstSemiPos != -1) {
          realEnd = firstSemiPos;
        }
        sql = text.substring(realStart, realEnd).trim();
      }
    } else {
      sql = editor.getSelectionText().trim();
    }
    console.log('sql', sql.trim().replaceAll('\n', ' '));
  }
}

onBeforeUnmount(() => {
  const editor = editorRef.value;
  if (editor) {
    editor.destroy();
  }
})


const dataState = reactive({
  arrayResult: [] as any[],
  actionResult: '',
  arrayResultCols: new Set<string>(),
});
const lastExecOnDbPath = ref('');

emitter.on('meta_objects_refreshed', (newCurrent) => {
  pageCache.current = newCurrent as CurrentDbAndTable;
});

const watermarkText = ref('');
const execSql = () => {
  if (pageCache.current.db !== undefined) {
    lastExecOnDbPath.value = pageCache.current.db;

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
 */
const commentLine = (e: Event) => {
  /*
  解决快捷键多重激发的问题：约定当前函数的快捷键为`Ctrl+b`，
  当按下`Ctrl+Shift+b`时也会触发本方法，所以需要进行检测拦截。
   */
  if (e instanceof KeyboardEvent && (e as KeyboardEvent).shiftKey) {
    return;
  }

  let sqlDiv = sqlContent.value;

  /*
  根据当前光标所在位置，或所选区域，计算出将要添加注释字符的行。
   */
  let fromLineNum = -1, toLineNum = -1;
  if (e instanceof MouseEvent) {
    fromLineNum = sqlSelection.fromLineNum;
    toLineNum = sqlSelection.toLineNum;
    console.log(fromLineNum, toLineNum)
  } else {
    let s = window.getSelection();
    if (s) {
      let selected = calcSelectionRange(sqlDiv, s);
      fromLineNum = selected.min;
      toLineNum = selected.max;

      s.collapseToEnd();
    }
  }

  /*
  在目标行开头处添加"--"字符串。
   */
  if (toLineNum > -1) {
    if (fromLineNum > -1) {
      for (let i = fromLineNum; i <= toLineNum; i++) {
        let n = sqlDiv.childNodes[i];
        if (n.nodeName == '#text') {
          n.nodeValue = '--' + n.nodeValue;
        } else {
          n.innerText = '--' + n.innerText;
        }
      }
    } else {
      let n = sqlDiv.childNodes[toLineNum];
      if (n.nodeName == '#text') {
        n.nodeValue = '--' + n.nodeValue;
      } else {
        n.innerText = '--' + n.innerText;
      }
    }
  }
}

/**
 * 移除光标所在行或选中的多个行前面的注释字符串。
 */
const uncommentLine = (e: Event) => {
  let sqlDiv = sqlContent.value;
  /*
  根据当前光标所在位置，或所选区域，计算出将要移除注释字符的行。
   */
  let fromLineNum = -1, toLineNum = -1;
  if (e instanceof MouseEvent) {
    fromLineNum = sqlSelection.fromLineNum;
    toLineNum = sqlSelection.toLineNum;
  } else {
    let s = window.getSelection();
    if (s) {
      let selected = calcSelectionRange(sqlDiv, s);
      fromLineNum = selected.min;
      toLineNum = selected.max;

      s.collapseToEnd();
    }
  }

  /*
  移除目标行开头处添加"--"字符串。
   */
  if (toLineNum > -1) {
    if (fromLineNum > -1) {
      for (let i = fromLineNum; i <= toLineNum; i++) {
        let n = sqlDiv.childNodes[i];
        let text = (n.nodeName == '#text') ? n.nodeValue : n.innerText;
        if (text.startsWith('--')) {
          if (n.nodeName == '#text') {
            n.nodeValue = n.nodeValue.replace(/\s*--/, '');
          } else {
            n.innerText = n.innerText.replace(/\s*--/, '');
          }
        }
      }
    } else {
      let n = sqlDiv.childNodes[toLineNum];
      let text = (n.nodeName == '#text') ? n.nodeValue : n.innerText;
      if (text.trim().startsWith('--')) {
        if (n.nodeName == '#text') {
          n.nodeValue = n.nodeValue.replace(/\s*--/, '');
        } else {
          n.innerText = n.innerText.replace(/\s*--/, '');
        }
      }
    }
  }
}


/**
 * 编辑框失去焦点时，记录选区位置。
 */
const rememberSelection = function (e: MouseEvent) {
  // console.log('开始处理选区');

  let sqlDiv = sqlContent.value;
  let s = window.getSelection();

  if (s) {
    let selected = calcSelectionRange(sqlDiv, s);
    sqlSelection.fromLineNum = selected.min;
    sqlSelection.toLineNum = selected.max;
    // 这里不能收缩选区，否则会导致无法选择的BUG。
  }
  // console.log('完成处理选区', sqlSelection.fromLineNum, sqlSelection.toLineNum);
}

/**
 * 计算DIV选区的起始行号和结束行号，若未选择则给出光标所在行号。
 * @param sqlDiv SQL编辑区的DIV对象。
 * @param s 选区对象。
 */
const calcSelectionRange = (sqlDiv: any, s: Selection): SelectedLines => {
  let y1 = -1, y2 = -1;
  for (let i = 0; i < sqlDiv.childNodes.length; i++) {
    let n = sqlDiv.childNodes[i];
    let text = (n.nodeName == '#text') ? n.nodeValue : n.innerText;
    if (s.anchorNode && s.anchorNode.textContent == text) {
      y1 = i;
    } else if (s.focusNode && s.focusNode.textContent == text) {
      y2 = i;
    }
  }
  return {
    min: Math.min(y1, y2),
    max: Math.max(y1, y2),
  }
}

const trimTagOnPaste = (e: Event) => {
  // e.preventDefault();
  // let text = '';
  // let ce = e as ClipboardEvent;
  // let clp = ce.clipboardData;
  // if (clp) {
  //   text = clp.getData('text/plain') || '';
  //   if (text) {
  //     let coloredText = highlight(text, {html: true});
  //     console.log(coloredText)
  //     sqlContent.value.innerHTML = coloredText;
  //     // let newText = highlight(text, {html: true});
  //     let s = window.getSelection();
  //     if (s) {
  //       let nodes = sqlContent.value.childNodes;
  //       let node = nodes[nodes.length - 1];
  //       let r = s.getRangeAt(0);
  //       r.selectNodeContents(node);
  //       s.collapse(node, text.length);
  //     }
  //   }
  // }

}

</script>

<style scoped>
.custom-area {
  height: 95vh;
}

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

.sql-content-pane {
  border-radius: 5px;
}

.sql-content {
  font-family: monospace;
  font-size: 1.1em;
  background-color: floralwhite;
  width: 100%;
  height: 100%;
  padding: .5em;
  overflow: scroll;
  border-radius: 5px;
  box-shadow: 0 0 10px #000000 inset;
  /*box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.1), 0 0 8px rgba(82, 168, 236, 0.6);*/
}

.last-db-path {
  margin-top: 1em;
  color: lightslategray;
}

.result-content {
  font-family: monospace;
}

.result-table {
  width: 100%;
}

.echo-sql-pre {
  height: 97%;
  width: 100%;
  overflow: scroll;
  margin-top: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/*.echo-sql-pre > code {*/
/*  min-height: 88%;*/
/*}*/
</style>
