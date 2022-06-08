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
    q-checkbox(v-model='showDetailedTooltip' label='Show detailed tooltip')

  leaflet-map.map(:markers='markers')

</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { round } from 'lodash'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'
import { PropertyAction, PropertySummary } from '../models/property'
import { TubeStation } from '../models/tube'
import { usePropertyStore } from '../stores/property'
import { useTubeStore } from '../stores/tube'

export default defineComponent({
  name: 'PropertyPage',
  components: {
    LeafletMap
  },
  setup () {
    const tubeStore = useTubeStore()
    const propertyStore = usePropertyStore()

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
    const showDetailedTooltip: Ref<boolean> = ref(false)
    const properties: Ref<PropertySummary[]> = ref(propertyStore.properties)
    const markers: Ref<L.Layer[]> = ref([])

    function search () {
      const isValid = (property: PropertySummary) => property.stats.price.count > 0
      const hasAction = (property: PropertySummary) => property.action === action.value.value
      const hasBeds = (property: PropertySummary) => property.numBeds === numBeds.value
      const withinPriceRange = (property: PropertySummary) =>
        priceRange.value.min <= property.stats.price.median &&
         (property.stats.price.median <= priceRange.value.max || includeBeyondPriceRange.value)

      properties.value = propertyStore.properties
        .filter(hasBeds)
        .filter(hasAction)
        .filter(withinPriceRange)
        .filter(isValid)

      markers.value = updateMarkers()
    }

    function getDetailedTooltipText (property: PropertySummary): string {
      if (showDetailedTooltip.value) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
        const station: TubeStation = tubeStore.postcodeToStations[property.postcode]
        return `<b>${station.name}</b> (Zone ${station.zone.toString()})<br>
        <table><tbody>
        <tr><td>Avg</td><td>${formatPrice(property.stats.price.median)}</td></tr>
        <tr><td>Range</td><td>${formatPrice(property.stats.price.min)} - ${formatPrice(property.stats.price.max)}</td></tr>
        <tr><td>IQR</td><td>${formatPrice(property.stats.price.q1)} - ${formatPrice(property.stats.price.q3)}</td></tr>
        <tr><td>Count</td><td>${property.stats.price.count}</td></tr> 
        <tr><td>Lines</td><td><ul>${station.lines.map(l => `<li>${l}</li>`).join('')}</ul></td></tr>
        </tbody></table>`
      } else {
        return formatPrice(property.stats.price.median)
      }
    }

    function formatPrice (price: number): string {
      return `£${round(price)}`
    }

    function updateMarkers (): L.Layer[] {
      return properties.value.map(property => {
        if (showDetailedTooltip.value) {
          return new L.CircleMarker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            { radius: 5 }
          ).bindTooltip(getDetailedTooltipText(property))
        } else {
          return new L.CircleMarker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            { radius: 1 }
          ).bindTooltip(formatPrice(property.stats.price.median), { permanent: true })
        }
      }
      )
    }

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
      showDetailedTooltip,

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
