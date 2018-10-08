import Vue from 'vue'
import Vuesax from 'vuesax'

import 'vuesax/dist/vuesax.css'
import 'material-icons/iconfont/material-icons.css'
Vue.use(Vuesax)

import App from './App'
import store from './store'

if (!process.env.IS_WEB) Vue.use(require('vue-electron'))
Vue.config.productionTip = false

/* eslint-disable no-new */
new Vue({
  components: { App },
  store,
  template: '<App/>'
}).$mount('#app')
