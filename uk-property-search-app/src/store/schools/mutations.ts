import type { MutationTree } from 'vuex'
import { School } from '../../models/school'
import { SchoolsStateInterface } from './state'

const mutation: MutationTree<SchoolsStateInterface> = {
  setSchools (state, schools: School[]) {
    state.schools = schools
  }
}

export default mutation
