<template>
  <div>
    <div class="action-bar">
      <el-tooltip :show-after="1000" content="执行当前输入的SQL (Ctrl+Enter)" placement="top">
        <el-icon class="icons" @click="execCurrentSql">
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
        <Toolbar :default-config="toolbarConfig" :editor="editorRef"></Toolbar>
        <Editor v-model="sqlHtml" :default-config="editorConfig" mode="simple"
                @customPaste="customPaste" @onCreated="handleCreated"
                @keydown.ctrl.b="commentLine" @keydown.ctrl.shift.b="uncommentLine"
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
import {IEditorConfig, IToolbarConfig, SlateEditor, SlateElement, SlateNode} from "@wangeditor/editor";
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

const editorRef = shallowRef();
const sqlHtml = ref('');
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
/**
 * 自定义粘贴处理方式，去除来源样式，仅保留文字。
 * @param editor
 * @param event
 * @param callback
 */
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

/**
 * 执行光标所在位置的SQL语句。
 */
const execCurrentSql = () => {
  if (pageCache.current.db) {
    lastExecOnDbPath.value = pageCache.current.db;
    let editor: IDomEditor = editorRef.value as IDomEditor;
    let normalizedSql = extractSql(editor);

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

/**
 * 从选区或光标所在位置优先向前查找完整的SQL语句，若不完整则追加向后查找到完整的语句。
 * 在光标前后两个分号之间的字符串被认为是完整的SQL语句。
 * 或者在贴近光标前的分号(忽略光标到分号之间的空白)到编辑区头部的字符串也被当作完整的SQL，
 * 最后或者从光标前的最近一个分号或区首，到光标后的一个分号或区末，被当作一个SQL。
 * @param editor
 */
const extractSql = (editor: IDomEditor): string => {
  let sql = '';
  let selection = editor.selection;
  if (selection && _.isEqual(selection.focus, selection.anchor)) {
    /*
    未选择任何文本，需要计算光标所在位置的一维坐标。
     */
    let text = editor.getText();

    /*
    设置新选区：从当前光标位置到编辑区起始0坐标处，
    再根据新选区内的文本长度，计算出当前光标的一维坐标。
     */
    let newSelection = {
      anchor: selection.anchor,
      focus: {
        path: [0, 0],
        offset: 0
      }
    };
    editor.select(newSelection);
    let prePart = editor.getSelectionText();
    // 当前光标的一维坐标
    let ahead = prePart.length;

    {
      /*
      BUG修复代码段，在目前的wangedit版本
      "@wangeditor/editor": "^5.1.23",
      "@wangeditor/editor-for-vue": "^5.1.12"
      中，getSelectionText()会有漏掉换行符的BUG，
      这里专门进行修正。
      以后的wangeditor版本或是修复了该BUG，则可以考虑去掉这块修复代码。
       */
      ahead += selection.anchor.path[0];
    }

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
      sql = text.substring(realStart, ahead);
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
      sql = text.substring(realStart, realEnd);
    }

    editor.deselect();
    //editor.focus();
  } else {
    // 选中了一些文本
    sql = editor.getSelectionText();
  }
  return normalizeSql(sql);
}
/**
 * 去除字符串中的换行符和冗余的空白，保留行首的注释符号。
 * @param sql
 */
const normalizeSql = (sql: string): string => {
  let normalizedSql = '';
  let origLines: string[] = sql.split('\n');
  for (let i = 0; i < origLines.length; i++) {
    if (!origLines[i].trim().startsWith('--')) {
      normalizedSql += origLines[i] + ' ';
    }
  }
  normalizedSql = normalizedSql.replaceAll(/\s+/g, ' ').trim();
  if (normalizedSql.charAt(normalizedSql.length - 1) == ';') {
    normalizedSql = normalizedSql.substring(0, normalizedSql.length - 1);
  }
  return normalizedSql;
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
  console.log('开始添加注释')

  let editor = editorRef.value as IDomEditor;
  let selection = editor.selection;
  let nodeEntries = SlateEditor.nodes(editor, {
    match: (node: SlateNode) => {
      if (SlateElement.isElement(node)) {
        console.log('节点类型', node)
        debugger
        if (node.type == 'paragraph') {
          return true;
        }
      }
      return false;
    },
    universal: true
  });
  if (nodeEntries) {
    for (let nodeEntry of nodeEntries) {
      const [node, path] = nodeEntry
      console.log('选中了 paragraph 节点', node)
      console.log('节点 path 是', path)
    }
  } else {
    console.log('当前未选中的 paragraph')
  }

  /*
  解决快捷键多重激发的问题：约定当前函数的快捷键为`Ctrl+b`，
  当按下`Ctrl+Shift+b`时也会触发本方法，所以需要进行检测拦截。
   */
  // if (e instanceof KeyboardEvent && (e as KeyboardEvent).shiftKey) {
  //   return;
  // }

  // let sqlDiv = sqlContent.value;
  // let editor: IDomEditor = editorRef.value as IDomEditor;
  // let sqlDiv = editor.getEditableContainer();

  /*
  根据当前光标所在位置，或所选区域，计算出将要添加注释字符的行。
   */
  // let fromLineNum = -1, toLineNum = -1;
  // if (e instanceof MouseEvent) {
  //   fromLineNum = sqlSelection.fromLineNum;
  //   toLineNum = sqlSelection.toLineNum;
  //   console.log(fromLineNum, toLineNum)
  // } else {
  //   let s = window.getSelection();
  //   if (s) {
  //     let selected = calcSelectionRange(sqlDiv, s);
  //     fromLineNum = selected.min;
  //     toLineNum = selected.max;
  //
  //     s.collapseToEnd();
  //   }
  // }

  /*
  在目标行开头处添加"--"字符串。
   */
  // if (toLineNum > -1) {
  //   if (fromLineNum > -1) {
  //     for (let i = fromLineNum; i <= toLineNum; i++) {
  //       let n = sqlDiv.childNodes[i];
  //       if (n.nodeName == '#text') {
  //         n.nodeValue = '--' + n.nodeValue;
  //       } else {
  //         n.innerText = '--' + n.innerText;
  //       }
  //     }
  //   } else {
  //     let n = sqlDiv.childNodes[toLineNum];
  //     if (n.nodeName == '#text') {
  //       n.nodeValue = '--' + n.nodeValue;
  //     } else {
  //       n.innerText = '--' + n.innerText;
  //     }
  //   }
  // }
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
</style>
