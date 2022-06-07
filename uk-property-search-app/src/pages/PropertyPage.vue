<template lang='pug'>
q-page(padding)
  .row.q-gutter-x-lg.items-end
    .col-1
      q-select(v-model='numBeds' :options='numBedsOptions' label='Beds')
    .col-1
      q-select(v-model='action' :options='actionOptions', label='Action')
    .col
      q-range(v-model='priceRange' :min='minPrice' :max='maxPrice' :step='step' label-always markers)
    .col-shrink
      q-checkbox(v-model='includeBeyondPriceRange' :label='`Include £${maxPrice}+`')

  .row.q-my-sm
    q-btn(label='Search' color='secondary' icon-right='search' @click='search')

  leaflet-map.map(:markers='markers')

</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { round } from 'lodash'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'
import { PropertyAction, PropertySummary } from '../models/property'
import { useStore } from '../store'

export default defineComponent({
  name: 'PropertyPage',
  components: {
    LeafletMap
  },
  setup () {
    const store = useStore()

    const minPrice = 0
    const maxPrice: ComputedRef<number> = computed(() => {
      switch (action.value.value) {
      case PropertyAction.Buy:
        return 2_000_000
      case PropertyAction.Rent:
        return 4_000
      default:
        return 0
      }
    })
    const step: ComputedRef<number> = computed(() => {
      switch (action.value.value) {
      case PropertyAction.Buy:
        return 50_000
      case PropertyAction.Rent:
        return 100
      default:
        return 0
      }
    })
    const actionOptions = [
      {
        label: 'Buy',
        value: PropertyAction.Buy
      },
      {
        label: 'Rent',
        value: PropertyAction.Rent
      }
    ]
    const numBedsOptions = [0, 1, 2, 3] // Only consider Studio - 3 bedroom flats

    const action: Ref<{label: string, value: PropertyAction}> = ref(actionOptions[0])
    const numBeds: Ref<number> = ref(2)
    const priceRange: Ref<{min: number, max:number}> = ref({
      min: minPrice,
      max: maxPrice
    })
    const includeBeyondPriceRange: Ref<boolean> = ref(false)
    const properties: Ref<PropertySummary[]> = ref(store.state.property.properties)

    function search () {
      const hasAction = (property: PropertySummary) => property.action === action.value.value
      const hasBeds = (property: PropertySummary) => property.numBeds === numBeds.value
      const withinPriceRange = (property: PropertySummary) =>
        priceRange.value.min <= property.stats.price.median &&
         (property.stats.price.median <= priceRange.value.max || includeBeyondPriceRange.value)

      properties.value = store.state.property.properties
        .filter(hasAction)
        .filter(hasBeds)
        .filter(withinPriceRange)
    }

    function getTooltipText (price: number): string {
      return price ? `£${round(price)}` : 'N/A'
    }

    const markers: ComputedRef<L.CircleMarker[]> = computed(() => {
      return properties.value.map(property =>
        new L.CircleMarker(
          { lat: property.coordinates[1], lng: property.coordinates[0] },
          { radius: 1 }
        ).bindTooltip(getTooltipText(property.stats.price.median), { permanent: true })
      )
    })

    return {
      minPrice,
      maxPrice,
      step,
      actionOptions,
      numBedsOptions,

      action,
      numBeds,
      priceRange,
      includeBeyondPriceRange,
      markers,

      search
    }
  }
})
</script>

<style lang="scss" scoped>
.map {
  height: 500px;
}
</style>
