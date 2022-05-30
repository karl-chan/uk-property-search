import { boot } from 'quasar/wrappers'
import schools from 'src/api/schools-api'

export default boot(({ app }) => {
  app.config.globalProperties.$api = {
    schools
  }
})
