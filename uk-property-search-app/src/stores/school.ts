import { defineStore } from 'pinia'
import schoolsApi from '../api/schools-api'
import { School } from '../models/school'

export const useSchoolStore = defineStore('school', {
  state: () => ({
    schools: <School[]>[]
  }),
  actions: {
    async init () {
      this.schools = await schoolsApi.fetchSchools()
    }
  },
  persist: true
})
