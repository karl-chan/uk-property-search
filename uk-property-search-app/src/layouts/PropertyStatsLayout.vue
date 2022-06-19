<template lang='pug'>
q-page(padding)
  .row.q-col-gutter-x-lg.items-end
    .col-6.col-sm-1
      q-select(v-model='numBeds' :options='numBedsOptions' label='Beds')
    .col-6.col-sm-1
      q-select(v-model='action' :options='actionOptions', label='Action')
    .col
      q-range(v-model='sliderRange' label-always markers
             :min='currentSliderOption.min'
             :max='currentSliderOption.max'
             :step='currentSliderOption.step'
             :left-label-value='formatSliderLabel(sliderRange.min)'
             :right-label-value='formatSliderLabel(sliderRange.max)')
    .col-shrink
      q-checkbox(v-model='includeBeyondRange' :label='`Include ${formatSliderLabel(currentSliderOption.max)}+`')

  .row.q-my-sm
    q-btn(label='Search' color='secondary' icon-right='search' @click='search' :loading='isLoading')
    q-checkbox(v-model='showDetailedTooltip' label='Show detailed tooltip')

  leaflet-map.map(:markers='markers')

  .row.q-my-sm
    q-table(:title='title' :rows='stationProperties' :columns='columns' :filter='tableFilter' :pagination='paginationOptions' row-key='postcode')
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
import { PropertyAction, PropertySummary, Stats } from '../models/property'
import { TubeStation } from '../models/tube'
import { usePropertyStore } from '../stores/property'
import { useTubeStore } from '../stores/tube'
import { sleep } from '../util/sleep'

interface StationProperty {
  station: TubeStation,
  property : PropertySummary
}

interface SliderOption {
  min: number,
  max: number,
  step: number,
  multiplier: number
  formatter: (value: number) => string
}

export type SliderOptions = {
  [action in PropertyAction]: SliderOption
}

interface FormatOption {
  formatShortValue: (value: number) => string,
  formatValue: (value: number) => string
  markerWidth: number
}

export type FormatOptions = {
  [action in PropertyAction]: FormatOption
}

interface Props {
  statsGetter: (p: PropertySummary) => Stats,
  sliderOptions: SliderOptions,
  formatOptions: FormatOptions
}

export default defineComponent({
  name: 'PropertyStatsLayout',
  components: {
    LeafletMap
  },
  props: {
    title: {
      type: String,
      required: true
    },
    statsGetter: {
      type: Function,
      required: true
    },
    sliderOptions: {
      type: Object,
      required: true
    },
    formatOptions: {
      type: Object,
      required: true
    }
  },
  setup (props) {
    const { statsGetter, sliderOptions, formatOptions } = props as unknown as Props

    const tubeStore = useTubeStore()
    const propertyStore = usePropertyStore()

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
    const currentFormatOption: Ref<FormatOption> = computed(() => formatOptions[action.value.value])
    const currentSliderOption: Ref<SliderOption> = computed(() => sliderOptions[action.value.value])
    const sliderRange: Ref<{min: number, max:number}> = ref({
      min: currentSliderOption.value.min,
      max: currentSliderOption.value.max
    })
    const includeBeyondRange: Ref<boolean> = ref(true)
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
      { name: 'median', label: 'Median', field: (row: StationProperty) => statsGetter(row.property).median, format: formatValue, sortable: true },
      { name: 'min', label: 'Min', field: (row: StationProperty) => statsGetter(row.property).min, format: formatValue, sortable: true },
      { name: 'max', label: 'Max', field: (row: StationProperty) => statsGetter(row.property).max, format: formatValue, sortable: true },
      { name: 'q1', label: 'Q1', field: (row: StationProperty) => statsGetter(row.property).q1, format: formatValue, sortable: true },
      { name: 'q3', label: 'Q3', field: (row: StationProperty) => statsGetter(row.property).q3, format: formatValue, sortable: true },
      { name: 'count', label: 'Count', field: (row: StationProperty) => statsGetter(row.property).count, sortable: true },
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
      const isValid = (stationProperty: StationProperty) => statsGetter(stationProperty.property).count > 0
      const hasAction = (stationProperty: StationProperty) => stationProperty.property.action === action.value.value
      const hasBeds = (stationProperty: StationProperty) => stationProperty.property.numBeds === numBeds.value
      const withinSliderRange = (stationProperty: StationProperty) => {
        const median = statsGetter(stationProperty.property).median
        const multiplier = currentSliderOption.value.multiplier
        const searchMin = sliderRange.value.min * multiplier
        const searchMax = sliderRange.value.max * multiplier
        const beyondMax = currentSliderOption.value.max
        return (searchMin <= median && median <= searchMax) || (includeBeyondRange.value && median >= beyondMax)
      }

      isLoading.value = true
      stationProperties.value = allStationProperties.value
        .filter(hasBeds)
        .filter(hasAction)
        .filter(withinSliderRange)
        .filter(isValid)

      await sleep(100)
      markers.value = updateMarkers()
      isLoading.value = false
    }

    function getDetailedTooltipText (stationProperty: StationProperty): string {
      const { station, property } = stationProperty
      const { min, q1, median, q3, max, count } = statsGetter(property)
      if (showDetailedTooltip.value) {
        return `<b>${station.name}</b> (Zone ${station.zone.toString()})<br>
        <table><tbody>
        <tr><td>Avg</td><td>${formatValue(median)}</td></tr>
        <tr><td>Range</td><td>${formatValue(min)} - ${formatValue(max)}</td></tr>
        <tr><td>IQR</td><td>${formatValue(q1)} - ${formatValue(q3)}</td></tr>
        <tr><td>Count</td><td>${count}</td></tr> 
        <tr><td>Lines</td><td><ul>${station.lines.map(l => `<li>${l}</li>`).join('')}</ul></td></tr>
        </tbody></table>`
      } else {
        return formatValue(median)
      }
    }

    function formatSliderLabel (sliderValue: number): string {
      return currentSliderOption.value.formatter(sliderValue)
    }

    function formatShortValue (value: number): string {
      return currentFormatOption.value.formatShortValue(value)
    }

    function formatValue (value: number): string {
      return currentFormatOption.value.formatValue(value)
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
          const width = currentFormatOption.value.markerWidth, height = 20
          return new L.Marker(
            { lat: property.coordinates[1], lng: property.coordinates[0] },
            {
              icon: L.icon({
                iconUrl: `data:image/svg+xml,
                <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${width}" height="${height}">
                  <rect width="100%" height="100%" style="fill:white; stroke:black; stroke-width:1;" />
                  <text x="${width / 2}" y="${height / 2}" text-anchor="middle" alignment-baseline="central" font-family="sans-serif">${formatShortValue(statsGetter(property).median)}</text>
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
      actionOptions,
      numBedsOptions,
      paginationOptions,
      columns,

      isLoading,
      action,
      numBeds,
      sliderRange,
      includeBeyondRange,
      showDetailedTooltip,
      tableFilter,
      currentSliderOption,

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
