import type { ActionTree } from 'vuex'
import tubeApi from '../../api/tube-api'
import { StateInterface } from '../index'
import { TubeStateInterface } from './state'

const actions: ActionTree<TubeStateInterface, StateInterface> = {
  async init ({ commit }) {
    const stations = await tubeApi.fetchStations()
    commit('setStations', stations)
  }
}

export default actions
