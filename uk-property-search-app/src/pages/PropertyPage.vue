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

  .row.q-my-sm
    q-table(title="Property prices" :rows='stationProperties' :columns='columns' :pagination='paginationOptions' row-key='postcode')
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

interface StationProperty {
  station: TubeStation,
  property : PropertySummary
}

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
    const paginationOptions = {
      rowsPerPage: 100
    }

    const action: Ref<{label: string, value: PropertyAction}> = ref(actionOptions[0])
    const numBeds: Ref<number> = ref(2)
    const priceRange: Ref<{min: number, max:number}> = ref({
      min: minPrice,
      max: maxPrice
    })
    const includeBeyondPriceRange: Ref<boolean> = ref(false)
    const showDetailedTooltip: Ref<boolean> = ref(false)
    const stationProperties: Ref<StationProperty[]> = ref([])
    const markers: Ref<L.Layer[]> = ref([])

    const allStationProperties: ComputedRef<StationProperty[]> = computed(() => {
      return propertyStore.properties.map(property => {
        const station: TubeStation = tubeStore.postcodeToStations[property.postcode]
        return { station, property }
      })
    })

    const columns = [
      { name: 'station', label: 'Station', field: (row: StationProperty) => row.station.name, sortable: true, align: 'left' },
      { name: 'zone', label: 'Zone', field: (row: StationProperty) => row.station.zone, format: (zones: number[]) => zones.join(','), sortable: true },
      { name: 'median', label: 'Median', field: (row: StationProperty) => row.property.stats.price.median, format: formatPrice, sortable: true },
      { name: 'min', label: 'Min', field: (row: StationProperty) => row.property.stats.price.min, format: formatPrice, sortable: true },
      { name: 'max', label: 'Max', field: (row: StationProperty) => row.property.stats.price.max, format: formatPrice, sortable: true },
      { name: 'q1', label: 'Q1', field: (row: StationProperty) => row.property.stats.price.q1, format: formatPrice, sortable: true },
      { name: 'q3', label: 'Q3', field: (row: StationProperty) => row.property.stats.price.q3, format: formatPrice, sortable: true },
      { name: 'count', label: 'Count', field: (row: StationProperty) => row.property.stats.price.count, sortable: true },
      { name: 'lines', label: 'Lines', field: (row: StationProperty) => row.station.lines, format: (lines: string[]) => lines.join(','), sortable: true, align: 'left' }
    ]

    function search () {
      const isValid = (stationProperty: StationProperty) => stationProperty.property.stats.price.count > 0
      const hasAction = (stationProperty: StationProperty) => stationProperty.property.action === action.value.value
      const hasBeds = (stationProperty: StationProperty) => stationProperty.property.numBeds === numBeds.value
      const withinPriceRange = (stationProperty: StationProperty) =>
        priceRange.value.min <= stationProperty.property.stats.price.median &&
         (stationProperty.property.stats.price.median <= priceRange.value.max || includeBeyondPriceRange.value)

      stationProperties.value = allStationProperties.value
        .filter(hasBeds)
        .filter(hasAction)
        .filter(withinPriceRange)
        .filter(isValid)

      markers.value = updateMarkers()
    }

    function getDetailedTooltipText (stationProperty: StationProperty): string {
      const { station, property } = stationProperty
      if (showDetailedTooltip.value) {
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
      return stationProperties.value.map(stationProperty => {
        const { property } = stationProperty
        if (showDetailedTooltip.value) {
          return new L.CircleMarker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            { radius: 10 }
          ).bindTooltip(getDetailedTooltipText(stationProperty))
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
      paginationOptions,
      columns,

      action,
      numBeds,
      priceRange,
      includeBeyondPriceRange,
      showDetailedTooltip,

      stationProperties,
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
