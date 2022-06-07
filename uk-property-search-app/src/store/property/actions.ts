import type { ActionTree } from 'vuex'
import propertyApi from '../../api/property-api'
import { StateInterface } from '../index'
import { PropertyStateInterface } from './state'

const actions: ActionTree<PropertyStateInterface, StateInterface> = {
  async init ({ commit }) {
    const properties = await propertyApi.fetchProperties()
    commit('setProperties', properties)
  }
}

export default actions
