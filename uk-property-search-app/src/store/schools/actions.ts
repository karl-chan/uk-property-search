import type { ActionTree } from 'vuex'
import schoolsApi from '../../api/schools-api'
import { StateInterface } from '../index'
import { SchoolsStateInterface } from './state'

const actions: ActionTree<SchoolsStateInterface, StateInterface> = {
  async init ({ commit }) {
    const schools = await schoolsApi.fetchSchools()
    commit('setSchools', schools)
  }
}

export default actions
