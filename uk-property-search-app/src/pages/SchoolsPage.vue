<template>
  <q-page padding>
    <div class="row q-gutter-x-lg">
      <div class="col-grow">
        <q-option-group
          v-model="ratings"
          :options="options"
          color="green"
          type="checkbox"
          inline
        />
      </div>
      <div class="col">
        <q-input
          v-model="cutoffDate"
          label="After Inpsection Date"
          debounce="500"
          filled
          mask="date"
          :rules="['date']"
        >
          <template #append>
            <q-icon
              name="event"
              class="cursor-pointer"
            >
              <q-popup-proxy
                ref="qDateProxy"
                transition-show="scale"
                transition-hide="scale"
              >
                <q-date v-model="cutoffDate">
                  <div class="row items-center justify-end">
                    <q-btn
                      v-close-popup
                      label="Close"
                      color="primary"
                      flat
                    />
                  </div>
                </q-date>
              </q-popup-proxy>
            </q-icon>
          </template>
        </q-input>
      </div>
    </div>
    <LeafletMap
      class="map"
      :markers="markers"
    />
  </q-page>
</template>

<script lang="ts">
import LeafletMap from 'components/LeafletMap.vue'
import L from 'leaflet'
import { colors, date } from 'quasar'
import type { ComputedRef, Ref } from 'vue'
import { computed, defineComponent, ref } from 'vue'
import { Rating, School } from '../models/school'
import { useStore } from '../store'

const { getPaletteColor } = colors

export default defineComponent({
  name: 'SchoolsPage',
  components: {
    LeafletMap
  },
  setup () {
    const store = useStore()

    const ratings: Ref<Rating[]> = ref([])
    const cutoffDate: Ref<string> = ref('2006/01/01')

    const options = [
      { label: 'Outstanding', value: Rating.Outstanding },
      { label: 'Good', value: Rating.Good },
      { label: 'Requires Improvement', value: Rating.RequiresImprovement },
      { label: 'Inadequate', value: Rating.Inadequate }
    ]

    const schools: ComputedRef<School[]> = computed(
      () => {
        const inRating = (school: School) => school.rating !== undefined && ratings.value.includes(school.rating)
        const afterCutoff = (school: School) => date.getDateDiff(school.inspectionDate ?? new Date().getTime(), cutoffDate.value, 'seconds') > 0
        return store.state.schools.schools
          .filter(inRating)
          .filter(afterCutoff)
      }
    )

    const markers: ComputedRef<L.CircleMarker[]> = computed(() => {
      return schools.value.flatMap((school) => {
        const getColor = (rating: Rating | undefined) => {
          switch (rating) {
          case Rating.Outstanding:
            return getPaletteColor('green')
          case Rating.Good:
            return getPaletteColor('amber')
          case Rating.RequiresImprovement:
            return getPaletteColor('red')
          case Rating.Inadequate:
            return getPaletteColor('black')
          case undefined:
            return getPaletteColor('grey')
          }
        }
        return school.coordinates
          ? [
            new L.CircleMarker(
              { lat: school.coordinates[1], lng: school.coordinates[0] },
              { color: getColor(school.rating), radius: 5 }
            ).bindTooltip(`${school.name} <br> (${date.formatDate(school.inspectionDate, 'YYYY-MM-DD') ?? 'Unknown'})`)
          ]
          : []
      })
    })
    return {
      cutoffDate,
      markers,
      options,
      ratings,
      schools
    }
  }
})
</script>

<style lang="scss" scoped>
.map {
  height: 500px;
}
</style>
