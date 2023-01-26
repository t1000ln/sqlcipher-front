import {createApp} from "vue";

// import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import "./style.css";
import 'xe-utils'
// import XEUtils from "xe-utils";
// @ts-ignore
import MyApp from "./App.vue";
import VXETable from "vxe-table";
import "vxe-table/lib/style.css";

const app = createApp(MyApp);
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}
// app.use(useTable);
app.use(VXETable)
// app.use(ElementPlus);
app.mount("#app");
