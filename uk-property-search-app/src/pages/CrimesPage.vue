<template lang='pug'>
q-page(padding)
  .row.q-col-gutter-x-lg.items-end
    .col-grow
      q-slider(v-model='sliderValue' label-always markers
              :min='threeYearsAgoSliderValue'
              :max='todaySliderValue'
              :step='1'
              :label-value='formatSliderLabel(sliderValue)')
    .col-3
      q-select(v-model='crimesFilter' :options='crimesFilterOptions' label='Crimes filter' multiple use-chips)

  leaflet-map.map(:markers='markers' @onBoundsChanged='onMapBoundsChanged')

</template>

<script lang="ts">
import axios from 'axios'
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { fromPairs } from 'lodash'
import { date, debounce, useQuasar } from 'quasar'
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

    const sliderValue: Ref<number> = ref(twoMonthsAgoSliderValue)

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

    function formatSliderLabel (sliderValue: number): string {
      const year = Math.floor(sliderValue / 12)
      const month = sliderValue % 12
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

      const year = Math.floor(sliderValue.value / 12)
      const month = sliderValue.value % 12 + 1 // This is passed to URL, not new Date(), so we need to +1.
      const yearMonth = `${year}-${String(month).padStart(2, '0')}`

      // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
      const polyString = `${neLat},${neLng}:${neLat},${swLng}:${swLat},${swLng}:${swLat},${neLng}`
      const url = `https://data.police.uk/api/crimes-street/all-crime?date=${yearMonth}&poly=${polyString}`

      try {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const { data } = await axios.get(url)
        crimes = data as Crime[]
        updateMarkers()

        if (dismissNotifyWarning) {
          dismissNotifyWarning()
          dismissNotifyWarning = null
        }
      } catch (err) {
        dismissNotifyWarning = $q.notify({
          message: 'Couldn\'t load data because the area is too large. Please zoom in.',
          color: 'red'
        })
      }
    }

    const debouncedSearch = debounce(search, 500)

    function updateMarkers () {
      const markerCluster = L.markerClusterGroup()
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

    async function onMapBoundsChanged (newBounds: object) {
      currentMapBounds = newBounds
      await debouncedSearch()
    }

    watch(sliderValue, debouncedSearch)
    watch(crimesFilter, updateMarkers)

    return {
      threeYearsAgoSliderValue,
      todaySliderValue: twoMonthsAgoSliderValue,
      crimesFilterOptions,

      sliderValue,
      crimesFilter,

      crimes,
      markers,

      formatSliderLabel,
      onMapBoundsChanged
    }
  }
})
</script>

<style lang="scss" scoped>
.map {
  height: 500px;
}
</style>
