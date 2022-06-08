import { boot } from 'quasar/wrappers'
import property from 'src/api/property-api'
import schools from 'src/api/schools-api'
import tube from 'src/api/tube-api'

export default boot(({ app }) => {
  app.config.globalProperties.$api = {
    property,
    schools,
    tube
  }
})
