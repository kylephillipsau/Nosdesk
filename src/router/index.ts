import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import TicketView from '../views/TicketView.vue'
import LoginView from '../views/LoginView.vue'
import ErrorView from '../views/ErrorView.vue'
import ListView from '../views/ListView.vue'
import SettingsView from '../views/SettingsView.vue'

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth: boolean;
    title?: string;
    layout?: string;
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginView,
      meta: {
        layout: 'blank',
        requiresAuth: false,
        title: 'Sign In'
      }
    },
    {
      path: '/',
      name: 'home',
      component: DashboardView,
      meta: {
        requiresAuth: true,
        title: 'Dashboard'
      }
    },
    {
      path: '/tickets',
      name: 'tickets',
      component: ListView,
      meta: {
        requiresAuth: true,
        title: 'Tickets'
      }
    },
    {
      path: '/tickets/create',
      name: 'create-ticket',
      component: () => import('../views/CreateTicketView.vue'),
      meta: {
        requiresAuth: true,
        title: 'Create Ticket'
      }
    },
    {
      path: '/tickets/:id',
      name: 'ticket-view',
      component: TicketView,
      props: true,
      meta: {
        requiresAuth: true,
        title: 'View Ticket'
      },
      beforeEnter: (to) => {
        to.meta.key = to.params.id
      }
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView,
      meta: {
        requiresAuth: true,
        title: 'Settings'
      }
    },
    {
      path: '/error/:code?/:message?',
      name: 'error',
      component: ErrorView,
      props: true,
      meta: {
        layout: 'blank',
        requiresAuth: false,
        title: 'Error'
      }
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/error/404'
    }
  ],
})

// Update document title on navigation
router.beforeResolve((to, from, next) => {
  let title = 'nosDesk';
  
  if (to.meta?.title) {
    title = to.meta.title as string;
    // For ticket view, append the ticket ID if available
    if (to.name === 'ticket-view' && to.params.id) {
      title = `Ticket #${to.params.id}`;
    }
  } else if (to.name) {
    title = to.name.toString()
      .split('-')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }
  
  // Update both document title and meta title
  document.title = `${title} | nosDesk`;
  if (to.meta) {
    to.meta.title = title;
  }

  next();
});

// Handle unsaved changes
router.beforeEach((to, from, next) => {
  // @ts-ignore - We'll add this property to the window object
  if (window.hasUnsavedChanges && !window.confirm('You have unsaved changes. Are you sure you want to leave?')) {
    next(false);
  } else {
    next();
  }
});

// Handle route errors
router.onError((error) => {
  router.push({
    name: 'error',
    params: {
      code: '500',
      message: 'Something went wrong'
    }
  })
})

export default router