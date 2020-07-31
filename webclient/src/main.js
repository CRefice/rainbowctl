import Vue from "vue";
import BoostrapVue from "bootstrap-vue";
import App from "./App.vue";

import "bootstrap/dist/css/bootstrap.css";
import "bootstrap-vue/dist/bootstrap-vue.css";

Vue.config.productionTip = false;
Vue.use(BoostrapVue);

new Vue({
  render: function(h) {
    return h(App);
  }
}).$mount("#app");
