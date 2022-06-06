import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      { path: 'property', component: () => import('src/pages/PropertyPage.vue') },
      { path: 'schools', component: () => import('src/pages/SchoolsPage.vue') },
      { path: '', redirect: '/property' }
    ]
  },

  // Always leave this as last one,
  // but you can also remove it
  {
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorPage.vue')
  }
]

export default routes
