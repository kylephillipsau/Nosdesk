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
      component: () => import('../views/ListView.vue'),
    },
    {
      path: '/tickets/create',
      name: 'create-ticket',
      component: () => import('../views/CreateTicketView.vue'),
    },
    {
      path: '/tickets/:id',
      name: 'ticket-view',
      component: TicketView,
      props: true,
      beforeEnter: (to) => {
        to.meta.key = to.params.id
      }
    }
  ],
})

export default router