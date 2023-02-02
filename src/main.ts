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
import 'highlight.js/styles/atom-one-light.css'
import hljs from 'highlight.js/lib/core';
import sql from 'highlight.js/lib/languages/sql';
import hljsVuePlugin from "@highlightjs/vue-plugin";

hljs.registerLanguage('sql', sql);


const app = createApp(MyApp);
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}
// app.use(useTable);
app.use(VXETable)
app.use(hljsVuePlugin);
// app.use(ElementPlus);
app.mount("#app");
