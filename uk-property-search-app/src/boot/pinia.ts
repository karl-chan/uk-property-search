import { createPinia } from 'pinia'
import { boot } from 'quasar/wrappers'
import { usePropertyStore } from '../stores/property'
import { useSchoolStore } from '../stores/school'
import { useTubeStore } from '../stores/tube'

export default boot(({ app }) => {
  app.use(createPinia())

  void Promise.all([
    usePropertyStore().init(),
    useSchoolStore().init(),
    useTubeStore().init()
  ])
})
