import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import TicketView from '../views/TicketView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: DashboardView,
    },
    {
      path: '/tickets',
      name: 'tickets',
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import('../views/ListView.vue'),
    },
    {
      path: '/tickets/:id',
      name: 'ticket-view',
      component: TicketView,
      // Add a custom props function to force re-render
      props: true,
      // Add key to the route to force component re-render
      beforeEnter: (to) => {
        to.meta.key = to.params.id
      }
    }
  ],
})

export default router
