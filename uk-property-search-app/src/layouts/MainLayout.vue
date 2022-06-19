<template lang='pug'>
q-layout(view='hHh Lpr lFr')
  q-header(elevated)
    q-toolbar
      q-btn(flat dense round icon='menu' aria-label='Menu' @click='toggleLeftDrawer')
      q-toolbar-title UK Property Search App
  q-drawer.bg-grey-1(v-model="leftDrawerOpen" show-if-above bordered)
    q-scroll-area.fit
      q-list
        template(v-for='menuSection in menuList' :key='menuSection.section')
          q-item-label.text-grey-8(header) {{menuSection.section}}
          template(v-for='(menuItem, index) in menuSection.children' :key='index')
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
        section: 'Property',
        children: [
          {
            icon: 'currency_pound',
            label: 'Prices',
            link: '/property/prices'
          },
          {
            icon: 'square_foot',
            label: 'Sizes',
            link: '/property/sizes'
          },
          {
            icon: 'timer',
            label: 'Turnover',
            link: '/property/turnover'
          }
        ]
      }, {
        section: 'Schools',
        children: [{
          icon: 'school',
          label: 'Ratings',
          link: '/schools'
        }
        ]
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
