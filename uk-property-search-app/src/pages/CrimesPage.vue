<template lang='pug'>
q-page(padding)
  .row.q-col-gutter-x-lg.items-end
    .col-grow
      q-range(v-model='sliderRange' label-always markers
              :min='threeYearsAgoSliderValue'
              :max='twoMonthsAgoSliderValue'
              :step='1'
              :left-label-value='formatSliderLabel(sliderRange.min)'
              :right-label-value='formatSliderLabel(sliderRange.max)')
    .col-3
      q-select(v-model='crimesFilter' :options='crimesFilterOptions' label='Crimes filter' multiple use-chips)

  .row.q-my-sm.justify-between.items-start
    .row
      q-btn(label='Search' color='secondary' icon-right='search' @click='search' :loading='isLoading')

  leaflet-map.map(:markers='markers' @onBoundsChanged='onMapBoundsChanged')

</template>

<script lang="ts">
import axios from 'axios'
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { chunk, debounce, fromPairs, range } from 'lodash'
import { date, useQuasar } from 'quasar'
import type { Ref } from 'vue'
import { defineComponent, ref, watch } from 'vue'
import { Crime } from '../models/crime'

export default defineComponent({
  label: 'CrimesPage',
  components: {
    LeafletMap
  },
  setup () {
    const $q = useQuasar()

    const twoMonthsAgo = date.subtractFromDate(new Date(), { months: 2 })
    const threeYearsAgo = date.subtractFromDate(new Date(), { years: 3 })
    const twoMonthsAgoSliderValue = twoMonthsAgo.getFullYear() * 12 + twoMonthsAgo.getMonth()
    const threeYearsAgoSliderValue = threeYearsAgo.getFullYear() * 12 + threeYearsAgo.getMonth()

    const sliderRange: Ref<{min: number, max:number}> = ref({
      min: twoMonthsAgoSliderValue,
      max: twoMonthsAgoSliderValue
    })

    const isLoading: Ref<boolean> = ref(false)
    const crimesFilter: Ref<{label: string, value: string}[]> = ref([])
    const markers: Ref<L.Layer[]> = ref([])

    const crimesFilterOptions = [
      { label: 'Anti-social behaviour', value: 'anti-social-behaviour' },
      { label: 'Bicycle theft', value: 'bicycle-theft' },
      { label: 'Burglary', value: 'burglary' },
      { label: 'Criminal damage and arson', value: 'criminal-damage-arson' },
      { label: 'Drugs', value: 'drugs' },
      { label: 'Other theft', value: 'other-theft' },
      { label: 'Possession of weapons', value: 'possession-of-weapons' },
      { label: 'Public order', value: 'public-order' },
      { label: 'Robbery', value: 'robbery' },
      { label: 'Shoplifting', value: 'shoplifting' },
      { label: 'Theft from the person', value: 'theft-from-the-person' },
      { label: 'Vehicle crime', value: 'vehicle-crime' },
      { label: 'Violence and sexual offences', value: 'violent-crime' },
      { label: 'Other crime', value: 'other-crime' }
    ]
    const crimeNameLookup = fromPairs(crimesFilterOptions.map(e => [e.value, e.label]))

    let currentMapBounds: object|null = null
    let crimes: Crime[] = []
    let dismissNotifyWarning : (() => void) | null = null

    function formatSliderLabel (sliderRange: number): string {
      const year = Math.floor(sliderRange / 12)
      const month = sliderRange % 12
      return date.formatDate(new Date(year, month), 'MMM YYYY')
    }

    async function search () {
      // @ts-ignore
      const { bounds } = currentMapBounds
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const { _northEast, _southWest } = bounds
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const { lat: neLat, lng: neLng } = _northEast
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      const { lat: swLat, lng: swLng } = _southWest

      const urls = range(sliderRange.value.min, sliderRange.value.max + 1)
        .map(sliderValue => {
          const year = Math.floor(sliderValue / 12)
          const month = sliderValue % 12 + 1 // This is passed to URL, not new Date(), so we need to +1.
          const yearMonth = `${year}-${String(month).padStart(2, '0')}`
          // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
          const polyString = `${neLat},${neLng}:${neLat},${swLng}:${swLat},${swLng}:${swLat},${neLng}`
          return `https://data.police.uk/api/crimes-street/all-crime?date=${yearMonth}&poly=${polyString}`
        })

      try {
        isLoading.value = true

        const crimesBuffer: Crime[] = []
        const urlBatches = chunk(urls, 8) // Send requests in batches of N to work around 429 http error (too many requests)
        for (const urlBatch of urlBatches) {
          const results = await Promise.all(urlBatch.map(url => axios.get(url)))
          const crimes = results.flatMap(result => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            const { data } = result
            return data as Crime[]
          })
          crimesBuffer.push(...crimes)
        }
        crimes = crimesBuffer

        updateMarkers()

        isLoading.value = false

        if (dismissNotifyWarning) {
          dismissNotifyWarning()
          dismissNotifyWarning = null
        }
      } catch (err) {
        dismissNotifyWarning = $q.notify({
          message: 'Couldn\'t load data because the area is too large. Please zoom in.',
          color: 'red'
        })
        isLoading.value = false
      }
    }

    function updateMarkers () {
      const markerCluster = L.markerClusterGroup({ chunkedLoading: true })
      const crimesSet = new Set(crimesFilter.value.map(e => e.value))
      crimes.filter(crime => !crimesSet.size || crimesSet.has(crime.category))
        .forEach(crime =>
          markerCluster.addLayer(new L.Marker({
            lat: +crime.location.latitude,
            lng: +crime.location.longitude
          }).bindTooltip(formatCrimeTooltip(crime)))
        )
      markers.value = [markerCluster]
    }

    function formatCrimeTooltip (crime: Crime) {
      const crimeName = crimeNameLookup[crime.category]
      const crimeLocation = crime.location.street.name
      const crimeMonth = crime.month
      return `<b>${crimeName}</b>
      <br>${crimeLocation}
      <br>${crimeMonth}`
    }

    function onMapBoundsChanged (newBounds: object) {
      currentMapBounds = newBounds
    }

    const debouncedUpdateMarkers = debounce(updateMarkers, 2000)
    watch(crimesFilter, debouncedUpdateMarkers)

    return {
      threeYearsAgoSliderValue,
      twoMonthsAgoSliderValue,
      crimesFilterOptions,

      isLoading,
      sliderRange,
      crimesFilter,

      crimes,
      markers,

      formatSliderLabel,
      onMapBoundsChanged,
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
