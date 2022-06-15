
<template lang='pug'>
q-page(padding)
  .row.q-gutter-x-lg.items-end
    .col-1
      q-select(v-model='numBeds' :options='numBedsOptions' label='Beds')
    .col-1
      q-select(v-model='action' :options='actionOptions', label='Action')
    .col
      q-range(v-model='sizeRange' :min='minSize' :max='maxSize' :step='step' label-always markers
             :left-label-value='formatSliderLabel(sizeRange.min)' :right-label-value='formatSliderLabel(sizeRange.max)')
    .col-shrink
      q-checkbox(v-model='includeBeyondSizeRange' :label='`Include ${formatSliderLabel(maxSize)}+`')

  .row.q-my-sm
    q-btn(label='Search' color='secondary' icon-right='search' @click='search' :loading='isLoading')
    q-checkbox(v-model='showDetailedTooltip' label='Show detailed tooltip')

  leaflet-map.map(:markers='markers')

  .row.q-my-sm
    q-table(title="Property sizes" :rows='stationProperties' :columns='columns' :filter='tableFilter' :pagination='paginationOptions' row-key='postcode')
      template(v-slot:top-right)
        q-input(borderless dense debounce='300' v-model='tableFilter' placeholder='Search')
          template(v-slot:append)
            q-icon(name='search')
</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { round } from 'lodash'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'
import { PropertyAction, PropertySummary } from '../../models/property'
import { TubeStation } from '../../models/tube'
import { usePropertyStore } from '../../stores/property'
import { useTubeStore } from '../../stores/tube'
import { sleep } from '../../util/sleep'

interface StationProperty {
  station: TubeStation,
  property : PropertySummary
}

export default defineComponent({
  name: 'PropertySizesPage',
  components: {
    LeafletMap
  },
  setup () {
    const tubeStore = useTubeStore()
    const propertyStore = usePropertyStore()

    const minSize = 0
    const maxSize = 1000
    const step = 100
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
    const sizeRange: Ref<{min: number, max:number}> = ref({
      min: minSize,
      max: maxSize
    })
    const includeBeyondSizeRange: Ref<boolean> = ref(true)
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
      { name: 'zone', label: 'Zone', field: (row: StationProperty) => row.station.zone, format: (zones: number[]) => zones.join(','), sortable: true, sort: sortZones },
      { name: 'median', label: 'Median', field: (row: StationProperty) => row.property.stats.squareFeet.median, format: formatSize, sortable: true },
      { name: 'min', label: 'Min', field: (row: StationProperty) => row.property.stats.squareFeet.min, format: formatSize, sortable: true },
      { name: 'max', label: 'Max', field: (row: StationProperty) => row.property.stats.squareFeet.max, format: formatSize, sortable: true },
      { name: 'q1', label: 'Q1', field: (row: StationProperty) => row.property.stats.squareFeet.q1, format: formatSize, sortable: true },
      { name: 'q3', label: 'Q3', field: (row: StationProperty) => row.property.stats.squareFeet.q3, format: formatSize, sortable: true },
      { name: 'count', label: 'Count', field: (row: StationProperty) => row.property.stats.squareFeet.count, sortable: true },
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
      const isValid = (stationProperty: StationProperty) => stationProperty.property.stats.squareFeet.count > 0
      const hasAction = (stationProperty: StationProperty) => stationProperty.property.action === action.value.value
      const hasBeds = (stationProperty: StationProperty) => stationProperty.property.numBeds === numBeds.value
      const withinSizeRange = (stationProperty: StationProperty) =>
        sizeRange.value.min <= stationProperty.property.stats.squareFeet.median &&
         (stationProperty.property.stats.squareFeet.median <= sizeRange.value.max || includeBeyondSizeRange.value)

      isLoading.value = true
      stationProperties.value = allStationProperties.value
        .filter(hasBeds)
        .filter(hasAction)
        .filter(withinSizeRange)
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
        <tr><td>Avg</td><td>${formatSize(property.stats.squareFeet.median)}</td></tr>
        <tr><td>Range</td><td>${formatSize(property.stats.squareFeet.min)} - ${formatSize(property.stats.squareFeet.max)}</td></tr>
        <tr><td>IQR</td><td>${formatSize(property.stats.squareFeet.q1)} - ${formatSize(property.stats.squareFeet.q3)}</td></tr>
        <tr><td>Count</td><td>${property.stats.squareFeet.count}</td></tr> 
        <tr><td>Lines</td><td><ul>${station.lines.map(l => `<li>${l}</li>`).join('')}</ul></td></tr>
        </tbody></table>`
      } else {
        return formatSize(property.stats.squareFeet.median)
      }
    }

    function formatSliderLabel (sliderValue: number) {
      return `${sliderValue} sq. ft.`
    }

    function formatShortSize (size: number): string {
      return `${round(size)} sf`
    }

    function formatSize (size: number): string {
      return `${round(size)} sq. ft.`
    }

    function sortZones (a: string, b: string): number {
      return parseInt(a, 10) - parseInt(b, 10)
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
          const width = 60, height = 20
          return new L.Marker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            {
              icon: L.icon({
                iconUrl: `data:image/svg+xml,
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${height}">
                  <rect width="100%" height="100%" style="fill:white; stroke:black; stroke-width:1;" />
                  <text x="${width / 2}" y="${height / 2}" text-anchor="middle" alignment-baseline="central" font-family="sans-serif">${formatShortSize(property.stats.squareFeet.median)}</text>
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
      minSize,
      maxSize,
      step,
      actionOptions,
      numBedsOptions,
      paginationOptions,
      columns,

      isLoading,
      action,
      numBeds,
      sizeRange,
      includeBeyondSizeRange,
      showDetailedTooltip,
      tableFilter,

      stationProperties,
      markers,

      formatSliderLabel,
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
