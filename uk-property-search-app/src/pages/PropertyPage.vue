<template lang='pug'>
q-page(padding)
  .row.q-gutter-x-lg
    q-select(v-model='numBeds' :options='[0, 1, 2, 3]' label='Number of bedrooms')
    q-range(v-model='priceRange' :min='minPrice' :max='maxPrice' :step='step' label-always markers)

  leaflet-map.map(:markers='markers')

</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'

export default defineComponent({
  name: 'PropertyPage',
  components: {
    LeafletMap
  },
  setup () {
    const numBeds: Ref<number> = ref(2)
    const minPrice = 0
    const maxPrice = 5_000_000
    const step = 50_000
    const priceRange: Ref<{min: number, max:number}> = ref({
      min: minPrice,
      max: maxPrice
    })

    const markers: ComputedRef<L.Marker[]> = computed(() => {
      return []
    })

    return {
      numBeds,
      minPrice,
      maxPrice,
      step,
      priceRange,
      markers
    }
  }
})
</script>

<style lang="scss" scoped>
.map {
  height: 500px;
}
</style>
