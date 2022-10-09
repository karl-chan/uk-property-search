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
          .row.justify-between.items-center
            q-item-label.text-grey-8(header) {{menuSection.section}}
            q-badge {{formatLastUpdated(menuSection.lastUpdated)}}
          template(v-for='(menuItem, index) in menuSection.children' :key='index')
            q-item(:to='menuItem.link' exact)
              q-item-section(avatar)
                q-icon(:name='menuItem.icon')
              q-item-section {{menuItem.label}}

  q-page-container
    router-view

</template>

<script lang="ts">
import { date } from 'quasar'
import { computed, defineComponent, ref } from 'vue'
import { useLastUpdatedStore } from '../stores/last-updated'

export default defineComponent({
  name: 'MainLayout',
  setup () {
    const lastUpdatedStore = useLastUpdatedStore()

    const leftDrawerOpen = ref(false)
    const menuList = computed(() => [
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
            label: 'Listings Age',
            link: '/property/listings-age'
          },
          {
            icon: 'star',
            label: 'Popularity',
            link: '/property/popularity'
          }
        ],
        lastUpdated: lastUpdatedStore.lastUpdated.property
      }, {
        section: 'Schools',
        children: [
          {
            icon: 'school',
            label: 'Ratings',
            link: '/schools'
          }
        ],
        lastUpdated: lastUpdatedStore.lastUpdated.schools
      }, {
        section: 'Crimes',
        children: [
          {
            icon: 'crisis_alert',
            label: 'Crimes',
            link: '/crimes'
          }
        ],
        lastUpdated: new Date()
      }
    ])

    function formatLastUpdated (lastUpdatedMs: number | undefined) : string {
      if (!lastUpdatedMs) {
        return 'Missing data'
      }
      const diffDays = date.getDateDiff(new Date(), lastUpdatedMs, 'days')
      switch (diffDays) {
      case 0: return 'Last updated today'
      case 1: return 'Last updated yesterday'
      default: return `Last updated ${diffDays} days ago`
      }
    }

    return {
      leftDrawerOpen,
      menuList,
      formatLastUpdated,
      toggleLeftDrawer () {
        leftDrawerOpen.value = !leftDrawerOpen.value
      }
    }
  }
})
</script>
