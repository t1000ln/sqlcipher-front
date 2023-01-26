<template>
  <div>
    <div class="action-bar">
      <el-tooltip :show-after="1000" content="删除选中的行 (Delete)" placement="top">
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
        <!--        <el-icon class="icons" @click="commitEdit">-->
        <!--          <Expand></Expand>-->
        <!--        </el-icon>-->
        <i class="vxe-icon-download icons" @click="commitActions"></i>
      </el-tooltip>

      <el-tooltip :show-after="1000" content="在提交前撤销修改" placement="top">
        <el-icon class="icons" @click="revertEdit">
          <RefreshLeft></RefreshLeft>
        </el-icon>
      </el-tooltip>

    </div>

    <vxe-table ref="contentTable" :column-config="{resizable: true}" :data="tableDataState.rows"
               :edit-config="{trigger: 'click', mode: 'cell', showStatus: true}"
               :row-config="{isHover: true, height: 30}"
               align="center" border height="300" keep-source max-height="600"
               show-overflow
               stripe @edit-closed="editClosedEvent">
      <vxe-column type="checkbox" width="60"></vxe-column>
      <vxe-column title="序号" type="seq" width="60"></vxe-column>
      <vxe-column v-for="(item, index) in tableDataState.cols" :key="index"
                  :edit-render="{autofocus: '.vxe-input--inner'}" :field="item"
                  :title="item">
        <template #edit="{ row }">
          <vxe-input v-model="row[item]" type="text"></vxe-input>
        </template>
      </vxe-column>
    </vxe-table>
  </div>


</template>

<script lang="ts" name="TableContent" setup>

import emitter, {ApiResp, backApi, CurrentDbAndTable, EditApiParams, TableData} from "../types/common";
import {reactive, ref} from "vue";
import {VxeTableEvents, VxeTableInstance} from "vxe-table";
import {ElMessage, ElMessageBox} from "element-plus";
import * as _ from 'lodash'

const contentTable = ref<VxeTableInstance>();
const tableDataState = reactive({
  cols: [] as string[],
  rows: [] as object[]
});

const pageCache = reactive({current: {} as CurrentDbAndTable});
emitter.on('fetch_table_data', (current) => {
  pageCache.current = current as CurrentDbAndTable;
});

emitter.on('refresh_table_data', (newData) => {
  let nd = newData as TableData;
  if (nd.cols !== undefined) {
    tableDataState.cols = nd.cols;
  }
  if (nd.rows !== undefined) {
    tableDataState.rows = nd.rows;
  }
})

/**
 * 缓存修改过的行和字段。
 * 其key为目标行的_X_ROW_KEY值，其value为目标行所修改的字段新值。
 */
const editCache = new Map<string, object>();

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
    let editRow: object | undefined = editCache.get(row._X_ROW_KEY);
    if ($table.isUpdateByRow(row, field)) {
      // 目标字段被修改了
      if (editRow !== undefined) {
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
const commitActions = () => {
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
        return r.rowid
      });
    }

    /*
    加工待新增的数据行数组。
     */
    let tobeAdded = $table.getInsertRecords();
    if (tobeAdded.length > 0) {
      apiParams.newRows = tobeAdded.map((r) => {
        let row = _.clone(r);
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
      apiParams.editRows = {};
      for (let i = 0; i < tobeUpdated.length; i++) {
        let r = tobeUpdated[i];
        let editRow = _.clone(editCache.get(r._X_ROW_KEY) as object);
        let rowid = r.rowid;
        apiParams.editRows[rowid] = editRow;
      }
    }

    backApi("update_table_data", apiParams, (resp) => {
      let r: ApiResp = JSON.parse(resp as string);
      if (r.success) {
        ElMessage.success('提交成功')
      } else {
        ElMessage.error(r.message);
      }
    })
  }
}

/**
 * 撤回尚未提交的变更数据。
 */
const revertEdit = async () => {
  const $table = contentTable.value;
  if ($table) {
    await ElMessageBox.confirm('确认撤回尚未提交的改动吗？', '提醒').then((_) => {
      $table.revertData();
    }).catch((cancel) => {
    })
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
</style>
