import type { MutationTree } from 'vuex'
import { TubeStation } from '../../models/tube'
import { TubeStateInterface } from './state'

const mutation: MutationTree<TubeStateInterface> = {
  setStations (state, stations: TubeStation[]) {
    state.stations = stations
  }
}

export default mutation
