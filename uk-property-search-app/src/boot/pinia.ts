import { createPinia } from 'pinia'
import persist from 'pinia-plugin-persistedstate'
import { Notify } from 'quasar'
import { boot } from 'quasar/wrappers'
import { useLastUpdatedStore } from '../stores/last-updated'
import { usePropertyStore } from '../stores/property'
import { useSchoolStore } from '../stores/school'
import { useTubeStore } from '../stores/tube'

export default boot(({ app }) => {
  const pinia = createPinia()
  pinia.use(persist)
  app.use(pinia)

  const progress = Notify.create({
    message: 'Downloading latest data...',
    spinner: true,
    color: 'negative',
    group: false, // required to be updatable
    timeout: 0 // we want to be in control when it gets dismissed
  })

  void Promise.all([
    usePropertyStore().init(),
    useSchoolStore().init(),
    useTubeStore().init(),
    useLastUpdatedStore().init()
  ])
    .then(() => {
      progress({
        message: 'Done!',
        color: 'positive',
        icon: 'done', // we add an icon
        spinner: false, // we reset the spinner setting so the icon can be displayed
        timeout: 2500 // we will timeout it in 2.5s
      })
    })
})
