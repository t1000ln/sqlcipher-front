<template>
  <div>
    <div class="action-bar">
      <el-tooltip :show-after="1000" content="删除选中的行 (Ctrl+Delete)" placement="top">
        <el-icon class="icons" @click="removeCheckedRows">
          <Delete/>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="新增数据 (Alt+Insert)" placement="top-start">
        <el-icon class="icons" @click="insertNewRow">
          <Plus/>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="提交修改 (Ctrl+Enter)" placement="top">
        <i class="vxe-icon-download icons" @click="commitActions"></i>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="在提交前撤销修改" placement="top">
        <el-icon class="icons" @click="revertEdit">
          <RefreshLeft></RefreshLeft>
        </el-icon>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="最多100条数据" placement="top">
        <el-button @click="changeLimit(100)">0..100</el-button>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="最多10w条数据" placement="top">
        <el-button @click="changeLimit(100000)">0..100000</el-button>
      </el-tooltip>

    </div>

    <vxe-table ref="contentTable" :column-config="{resizable: true}" :data="tableDataState.rows"
               :edit-config="editConfig.cfg"
               :keyboard-config="{isEsc: true, isTab: true, isEnter: true, isArrow: true}"
               :row-config="{isHover: true}"
               align="center" border height="300" keep-source max-height="600"
               show-overflow stripe @edit-closed="editClosedEvent">
      <vxe-column v-if="!pageCache.current.isView" type="checkbox" width="50"></vxe-column>
      <vxe-column v-if="pageCache.current.isView" title="序号" type="seq" width="60"></vxe-column>
      <vxe-column v-for="(item, index) in tableDataState.cols" :key="index"
                  :edit-render="editConfig.render" :field="item.name"
                  :title="item.name">
        <template v-if="!pageCache.current.isView" #edit="{ row }">
          <vxe-input v-model="row[item.name]" type="text"></vxe-input>
        </template>
      </vxe-column>
    </vxe-table>
  </div>


</template>

<script lang="ts" name="TableContent" setup>

import emitter, {
  ApiResp,
  backApi,
  ColumnMeta,
  CurrentDbAndTable,
  EditApiParams,
  RowType,
  TableData
} from "../types/common";
import {reactive, ref} from "vue";
import {VxeTableEvents, VxeTableInstance, VxeTablePropTypes} from "vxe-table";
import {ElMessage, ElMessageBox} from "element-plus";
import * as _ from 'lodash'

const currentLimit = ref(100);
const contentTable = ref<VxeTableInstance>();
const tableDataState = reactive({
  cols: [] as ColumnMeta[],
  rows: [] as object[]
});

const editConfig = reactive({
  cfg: {trigger: 'dblclick', mode: 'cell', showStatus: true} as VxeTablePropTypes.EditConfig,
  render: {autofocus: '.vxe-input--inner'}
});

const pageCache = reactive({current: {} as CurrentDbAndTable});
emitter.on('fetch_table_data_evt', (current) => {
  pageCache.current = current as CurrentDbAndTable;
  if (pageCache.current.isView) {
    editConfig.cfg = {trigger: 'manual', mode: 'row', showStatus: true};
    editConfig.render = {autofocus: ''};
  } else {
    editConfig.cfg = {trigger: 'dblclick', mode: 'cell', showStatus: true};
    editConfig.render = {autofocus: '.vxe-input--inner'};
  }
  fetchTableData(current as CurrentDbAndTable);
});

emitter.on('remove_history_success', () => {
  tableDataState.cols = [];
  tableDataState.rows = [];
  pageCache.current = {} as CurrentDbAndTable;
});

emitter.on('meta_objects_refreshed', (current) => {
  let new_current = current as CurrentDbAndTable;
  if (new_current.db != pageCache.current.db) {
    tableDataState.cols = [];
    tableDataState.rows = [];
    pageCache.current = {} as CurrentDbAndTable;
  }
});

const changeLimit = (limit: number) => {
  currentLimit.value = limit;
  fetchTableData(pageCache.current);
}

const fetchTableData = (currentMeta: CurrentDbAndTable) => {
  let params: { [key: string]: string | undefined | number } = {
    'dbPath': currentMeta.db,
    'tableName': currentMeta.table,
    'limit': currentLimit.value,
    'key': currentMeta.key
  };
  backApi("fetch_table_data", params, (resp) => {
    let r: ApiResp<TableData> = JSON.parse(resp as string);
    if (r.success) {
      let nd = r.data as TableData;
      if (nd.cols !== undefined) {
        tableDataState.cols = nd.cols;
      }
      if (nd.rows !== undefined) {
        tableDataState.rows = nd.rows;
      }
    } else {
      ElMessage.error(r.message);
    }
  });
}

/**
 * 缓存修改过的行和字段。
 * 其key为目标行的_X_ROW_KEY值，其value为目标行所修改的字段新值。
 */
const editCache = new Map<string, RowType>();

/**
 * 实时跟踪表格字段修改情况，并更新editCache。
 * @param row 当前修改的行。
 * @param column 当前修改的列。
 */
const editClosedEvent: VxeTableEvents.EditClosed = ({row, column}) => {
  const $table = contentTable.value
  const field = column.field
  const cellValue = row[field]
  // 判断单元格值是否被修改
  if ($table) {
    let editRow: RowType | undefined = editCache.get(row._X_ROW_KEY);
    if ($table.isUpdateByRow(row, field)) {
      // 目标字段被修改了
      if (editRow) {
        // 在目标行缓存中新增修改的字段
        editRow[field] = cellValue;
      } else {
        // 新增目标行修改记录，并添加目标行第一个被修改的字段
        editRow = {};
        editRow[field] = cellValue;
        editCache.set(row._X_ROW_KEY, editRow);
      }
    } else {
      if (editRow !== undefined) {
        // 目标字段被还原了，从editCache中取消记录。
        delete editRow[field];
        if (_.isEmpty(editRow)) {
          // 若目标行所有修改的字段都被还原了，则删除目标行的修改缓存。
          editCache.delete(row._X_ROW_KEY);
        }
      }
    }
  }
}


/**
 * 临时删除选中的行数据。
 */
const removeCheckedRows = async () => {
  let $table = contentTable.value;
  if ($table) {
    await $table.removeCheckboxRow();
  }
}

/**
 * 临时新增数据行。
 */
const insertNewRow = async () => {
  let $table = contentTable.value;
  if ($table) {
    let record = {};
    const {row: newRow} = await $table.insert(record);
    await $table.setEditRow(newRow);
  }
}

/**
 * 提交更删改数据。
 */
const commitActions = async () => {
  let ld = _;
  if (!pageCache.current.table) {
    return;
  }

  await ElMessageBox.confirm('确认提交改动吗？', '提醒').then((_) => {
    let $table = contentTable.value;
    if ($table) {
      let apiParams: EditApiParams = {
        dbPath: pageCache.current.db,
        tableName: pageCache.current.table,
      }
      if (pageCache.current.key) {
        apiParams.key = pageCache.current.key;
      }

      /*
      加工待删除的数据rowid数组。
       */
      let tobeRemoved = $table.getRemoveRecords();
      if (tobeRemoved.length > 0) {
        apiParams.delRows = tobeRemoved.map((r) => {
          return r.rowid.toString();
        });
      }

      /*
      加工待新增的数据行数组。
       */
      let tobeAdded = $table.getInsertRecords();
      if (tobeAdded.length > 0) {
        apiParams.newRows = tobeAdded.map((r) => {
          let row = ld.clone(r);
          delete row._X_ROW_KEY; // 去除vxe-table组件附加的属性。
          return row
        });
      }

      /*
      加工待更新的数据行map，
      其key为rowid，值为被修改的字段集合。
       */
      let tobeUpdated = $table.getUpdateRecords();
      if (tobeUpdated.length > 0) {
        apiParams.editRows = {} as RowType;
        for (let i = 0; i < tobeUpdated.length; i++) {
          let r = tobeUpdated[i];
          let editRow = ld.clone(editCache.get(r._X_ROW_KEY) as object);
          let rowid: string = r.rowid;
          apiParams.editRows[rowid] = editRow;
        }
      }

      backApi("update_table_data", apiParams, (resp) => {
        let r: ApiResp = JSON.parse(resp as string);
        if (r.success) {
          ElMessage.success('提交成功');
          fetchTableData(pageCache.current);
        } else {
          ElMessage.error(r.message);
        }
      })
    }
  }).catch((cancel) => {
  })


}

/**
 * 撤回尚未提交的变更数据。
 */
const revertEdit = async () => {
  const $table = contentTable.value;
  if ($table) {
    await ElMessageBox.confirm('撤回尚未提交的改动吗？', '提醒').then((_) => {
      $table.revertData();
    }).catch((cancel) => {
    })
  }
}

emitter.on('keyboard-action', (keys) => {
  let pressedKeys = keys as string;
  if (pressedKeys == 'ctrl-delete') {
    removeCheckedRows();
  } else if (pressedKeys == 'alt-insert') {
    insertNewRow();
  } else if (pressedKeys == 'ctrl-enter') {
    commitActions();
  }
})
</script>

<style scoped>
.action-bar {
  /*margin-bottom: .2em;*/
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
</style>
