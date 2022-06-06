import type { MutationTree } from 'vuex'
import { PropertySummary } from '../../models/property'
import { PropertyStateInterface } from './state'

const mutation: MutationTree<PropertyStateInterface> = {
  setProperties (state, properties: PropertySummary[]) {
    state.properties = properties
  }
}

export default mutation
