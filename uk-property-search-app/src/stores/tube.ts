import { keyBy } from 'lodash'
import { defineStore } from 'pinia'
import tubeApi from '../api/tube-api'
import { TubeStation } from '../models/tube'

export const useTubeStore = defineStore('tube', {
  state: () => ({
    stations: <TubeStation[]>[]
  }),
  getters: {
    postcodeToStations: (state) => keyBy(state.stations, s => s.postcode)
  },
  actions: {
    async init () {
      this.stations = await tubeApi.fetchStations()
    }
  }
})
