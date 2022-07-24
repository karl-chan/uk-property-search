
import { defineStore } from 'pinia'
import lastUpdatedApi from '../api/last-updated-api'
import { LastUpdated } from '../models/last-updated'

export const useLastUpdatedStore = defineStore('lastUpdated', {
  state: () => ({
    lastUpdated: <LastUpdated>{}
  }),
  getters: {
  },
  actions: {
    async init () {
      this.lastUpdated = await lastUpdatedApi.fetchLastUpdated()
    }
  },
  persist: true
})
