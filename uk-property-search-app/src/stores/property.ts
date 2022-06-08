import { defineStore } from 'pinia'
import propertyApi from '../api/property-api'
import { PropertySummary } from '../models/property'

export const usePropertyStore = defineStore('property', {
  state: () => ({
    properties: <PropertySummary[]>[]
  }),
  getters: {
  },
  actions: {
    async init () {
      this.properties = await propertyApi.fetchProperties()
    }
  }
})
