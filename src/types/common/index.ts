import {invoke, InvokeArgs} from "@tauri-apps/api/tauri";
import {ElMessage} from "element-plus";
// https://www.npmjs.com/package/mitt
import mitt from 'mitt';

/**
 * 调用后端接口的公用异步函数。
 * @param apiName 接口函数名。
 * @param params 接口参数。参数是一个Object对象，其中属性名称对应后端接口的形参名。注意这里传入的参数名是驼峰命名方式，会自动转换为后端接口的蛇形命名方式。例如后端接口声明为"pub async fn select_dept(dept_name: String, oper: String) -> String"，
 * 那面前端在这里调用时，需要给出参数为"{deptName: '某部门', oper: 'zhangsan'}"。
 *
 * @param callback 异步回调函数。
 */
export async function backApi(apiName: string, params: InvokeArgs, callback: (value: string) => void) {
    await invoke<string>(apiName, params)
        .then(callback, (reject) => ElMessage.warning(reject))
        .catch((err) => {
            console.error(err);
            ElMessage.error(err);
        })
}

// tauri接口调用返回的数据类型
export declare type ApiResp<T = any> = {
    success: boolean;
    code: number;
    message: string;
    data: T;
}

// 类型
export const emitter = mitt();

// 导出
export default emitter;
