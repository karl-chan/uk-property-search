<template lang='pug'>
q-page(padding)
  .row.q-gutter-x-lg.items-end
    .col-1
      q-select(v-model='numBeds' :options='numBedsOptions' label='Beds')
    .col-1
      q-select(v-model='action' :options='actionOptions', label='Action')
    .col
      q-range(v-model='priceRange' :min='minDuration' :max='maxDuration' label-always
             :left-label-value='formatDuration(priceRange.min)' :right-label-value='formatDuration(priceRange.max)')
    .col-shrink
      q-checkbox(v-model='includeBeyondPriceRange' :label='`Include ${formatDuration(maxDuration)}+`')

  .row.q-my-sm
    q-btn(label='Search' color='secondary' icon-right='search' @click='search' :loading='isLoading')
    q-checkbox(v-model='showDetailedTooltip' label='Show detailed tooltip')

  leaflet-map.map(:markers='markers')

  .row.q-my-sm
    q-table(title='Listings age' :rows='stationProperties' :columns='columns' :filter='tableFilter' :pagination='paginationOptions' row-key='postcode')
      template(v-slot:top-right)
        q-input(borderless dense debounce='300' v-model='tableFilter' placeholder='Search')
          template(v-slot:append)
            q-icon(name='search')
</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'
import { PropertyAction, PropertySummary } from '../models/property'
import { TubeStation } from '../models/tube'
import { usePropertyStore } from '../stores/property'
import { useTubeStore } from '../stores/tube'
import { sleep } from '../util/sleep'

interface StationProperty {
  station: TubeStation,
  property : PropertySummary
}

export default defineComponent({
  name: 'TurnoverPage',
  components: {
    LeafletMap
  },
  setup () {
    const tubeStore = useTubeStore()
    const propertyStore = usePropertyStore()

    const minDuration = 0
    const maxDuration: ComputedRef<number> = computed(() => {
      switch (action.value.value) {
      case PropertyAction.Buy:
        return 720
      case PropertyAction.Rent:
        return 84
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

    const isLoading: Ref<boolean> = ref(false)
    const action: Ref<{label: string, value: PropertyAction}> = ref(actionOptions[0])
    const numBeds: Ref<number> = ref(2)
    const priceRange: Ref<{min: number, max:number}> = ref({
      min: minDuration,
      max: maxDuration
    })
    const includeBeyondPriceRange: Ref<boolean> = ref(true)
    const showDetailedTooltip: Ref<boolean> = ref(false)
    const tableFilter: Ref<string> = ref('')
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
      { name: 'median', label: 'Median', field: (row: StationProperty) => row.property.stats.listedDays.median, format: formatDuration, sortable: true },
      { name: 'min', label: 'Min', field: (row: StationProperty) => row.property.stats.listedDays.min, format: formatDuration, sortable: true },
      { name: 'max', label: 'Max', field: (row: StationProperty) => row.property.stats.listedDays.max, format: formatDuration, sortable: true },
      { name: 'q1', label: 'Q1', field: (row: StationProperty) => row.property.stats.listedDays.q1, format: formatDuration, sortable: true },
      { name: 'q3', label: 'Q3', field: (row: StationProperty) => row.property.stats.listedDays.q3, format: formatDuration, sortable: true },
      { name: 'count', label: 'Count', field: (row: StationProperty) => row.property.stats.listedDays.count, sortable: true },
      {
        name: 'lines',
        label: 'Lines',
        field: (row: StationProperty) => row.station.lines.length,
        format: (count: number, row: StationProperty) => row.station.lines.join(', '),
        sortable: true,
        align: 'left'
      }
    ]

    async function search () {
      const isValid = (stationProperty: StationProperty) => stationProperty.property.stats.listedDays.count > 0
      const hasAction = (stationProperty: StationProperty) => stationProperty.property.action === action.value.value
      const hasBeds = (stationProperty: StationProperty) => stationProperty.property.numBeds === numBeds.value
      const withinPriceRange = (stationProperty: StationProperty) =>
        priceRange.value.min <= stationProperty.property.stats.listedDays.median &&
         (stationProperty.property.stats.listedDays.median <= priceRange.value.max || includeBeyondPriceRange.value)

      isLoading.value = true
      stationProperties.value = allStationProperties.value
        .filter(hasBeds)
        .filter(hasAction)
        .filter(withinPriceRange)
        .filter(isValid)

      await sleep(100)
      markers.value = updateMarkers()
      isLoading.value = false
    }

    function getDetailedTooltipText (stationProperty: StationProperty): string {
      const { station, property } = stationProperty
      if (showDetailedTooltip.value) {
        return `<b>${station.name}</b> (Zone ${station.zone.toString()})<br>
        <table><tbody>
        <tr><td>Avg</td><td>${formatDuration(property.stats.listedDays.median)}</td></tr>
        <tr><td>Range</td><td>${formatDuration(property.stats.listedDays.min)} - ${formatDuration(property.stats.listedDays.max)}</td></tr>
        <tr><td>IQR</td><td>${formatDuration(property.stats.listedDays.q1)} - ${formatDuration(property.stats.listedDays.q3)}</td></tr>
        <tr><td>Count</td><td>${property.stats.listedDays.count}</td></tr>
        <tr><td>Lines</td><td><ul>${station.lines.map(l => `<li>${l}</li>`).join('')}</ul></td></tr>
        </tbody></table>`
      } else {
        return formatDuration(property.stats.listedDays.median)
      }
    }

    function formatDuration (days: number): string {
      switch (action.value.value) {
      case PropertyAction.Buy:
        return `${(days / 30).toFixed(1)} months`
      case PropertyAction.Rent:
        return `${(days / 7).toFixed(1)} weeks`
      }
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
          const width = 100, height = 20
          return new L.Marker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            {
              icon: L.icon({
                iconUrl: `data:image/svg+xml,
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${height}">
                  <rect width="100%" height="100%" style="fill:white; stroke:black; stroke-width:1;" />
                  <text x="${width / 2}" y="${height / 2}" text-anchor="middle" alignment-baseline="central" font-family="sans-serif">${formatDuration(property.stats.listedDays.median)}</text>
                </svg>`,
                iconSize: [width, height],
                iconAnchor: [width / 2, height / 2]
              })
            }
          )
        }
      }
      )
    }

    return {
      minDuration,
      maxDuration,
      actionOptions,
      numBedsOptions,
      paginationOptions,
      columns,

      isLoading,
      action,
      numBeds,
      priceRange,
      includeBeyondPriceRange,
      showDetailedTooltip,
      tableFilter,

      stationProperties,
      markers,

      formatDuration,
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
