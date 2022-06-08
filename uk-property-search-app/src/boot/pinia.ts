import { createPinia } from 'pinia'
import { boot } from 'quasar/wrappers'
import { usePropertyStore } from '../stores/property'
import { useSchoolStore } from '../stores/school'
import { useTubeStore } from '../stores/tube'

export default boot(async ({ app }) => {
  app.use(createPinia())

  await Promise.all([
    usePropertyStore().init(),
    useSchoolStore().init(),
    useTubeStore().init()
  ])
})
