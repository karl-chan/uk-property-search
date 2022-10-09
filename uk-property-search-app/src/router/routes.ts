import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      { path: 'property/prices', component: () => import('src/pages/property/PropertyPricesPage.vue') },
      { path: 'property/sizes', component: () => import('src/pages/property/PropertySizesPage.vue') },
      { path: 'property/listings-age', component: () => import('src/pages/property/PropertyListingsAgePage.vue') },
      { path: 'property/popularity', component: () => import('src/pages/property/PropertyPopularityPage.vue') },
      { path: 'schools', component: () => import('src/pages/SchoolsPage.vue') },
      { path: 'crimes', component: () => import('src/pages/CrimesPage.vue') },
      { path: '', redirect: '/property/prices' }
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
