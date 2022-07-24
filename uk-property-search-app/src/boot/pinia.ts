import { createPinia } from 'pinia'
import persist from 'pinia-plugin-persistedstate'
import { boot } from 'quasar/wrappers'
import { useLastUpdatedStore } from '../stores/last-updated'
import { usePropertyStore } from '../stores/property'
import { useSchoolStore } from '../stores/school'
import { useTubeStore } from '../stores/tube'

export default boot(({ app }) => {
  const pinia = createPinia()
  pinia.use(persist)
  app.use(pinia)

  void Promise.all([
    usePropertyStore().init(),
    useSchoolStore().init(),
    useTubeStore().init(),
    useLastUpdatedStore().init()
  ])
})
