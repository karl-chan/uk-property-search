import type { Module } from 'vuex'
import { StateInterface } from '../index'
import actions from './actions'
import getters from './getters'
import mutations from './mutations'
import state, { PropertyStateInterface } from './state'

const exampleModule: Module<PropertyStateInterface, StateInterface> = {
  namespaced: true,
  actions,
  getters,
  mutations,
  state
}

export default exampleModule
