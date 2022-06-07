import { keyBy } from 'lodash'
import type { GetterTree } from 'vuex'
import { StateInterface } from '../index'
import { TubeStateInterface } from './state'

const getters: GetterTree<TubeStateInterface, StateInterface> = {
  postcodeToStation (state) {
    return keyBy(state.stations, s => s.postcode)
  }
}

export default getters
