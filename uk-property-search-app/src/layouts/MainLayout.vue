<template lang='pug'>
q-layout(view='hHh Lpr lFr')
  q-header(elevated)
    q-toolbar
      q-btn(flat dense round icon='menu' aria-label='Menu' @click='toggleLeftDrawer')
      q-toolbar-title UK Property Search App
  q-drawer.bg-grey-1(v-model="leftDrawerOpen" show-if-above bordered)
    q-scroll-area.fit
      q-list
        q-item-label.text-grey-8(header) Research
        template(v-for='(menuItem, index) in menuList' :key='index')
          q-item(:to='menuItem.link' exact)
            q-item-section(avatar)
              q-icon(:name='menuItem.icon')
            q-item-section {{menuItem.label}}

  q-page-container
    router-view

</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  name: 'MainLayout',
  setup () {
    const leftDrawerOpen = ref(false)
    const menuList = [
      {
        icon: 'apartment',
        label: 'Property',
        link: '/property'
      },
      {
        icon: 'school',
        label: 'Schools',
        link: '/schools'
      }
    ]

    return {
      leftDrawerOpen,
      menuList,
      toggleLeftDrawer () {
        leftDrawerOpen.value = !leftDrawerOpen.value
      }
    }
  }
})
</script>
