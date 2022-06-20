import { identity, keyBy, sortBy, uniq } from 'lodash'
import { defineStore } from 'pinia'
import tubeApi from '../api/tube-api'
import { TubeStation } from '../models/tube'

export const useTubeStore = defineStore('tube', {
  state: () => ({
    stations: <TubeStation[]>[]
  }),
  getters: {
    postcodeToStations: (state) => keyBy(state.stations, s => s.postcode),
    allLines: (state) => sortBy(
      uniq(state.stations.flatMap(s => s.lines)),
      identity
    )
  },
  actions: {
    async init () {
      this.stations = await tubeApi.fetchStations()
    }
  },
  persist: true
})
