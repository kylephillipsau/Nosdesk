import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import TicketView from '../views/TicketView.vue'
import LoginView from '../views/LoginView.vue'
import ErrorView from '../views/ErrorView.vue'
import TicketsListView from '../views/TicketsListView.vue'
import SettingsView from '../views/SettingsView.vue'
import ProjectsView from '../views/ProjectsView.vue'
import ProjectDetailView from '../views/ProjectDetailView.vue'
import UserProfileView from '../views/UserProfileView.vue'
import DocumentationView from '@/views/DocumentationView.vue'
import DocumentationPageView from '@/views/DocumentationPageView.vue'

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
      component: TicketsListView,
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
      path: '/users/:username',
      name: 'user-profile',
      component: UserProfileView,
      props: true,
      meta: {
        requiresAuth: true,
        title: 'User Profile'
      },
      beforeEnter: (to) => {
        to.meta.title = `${to.params.username}'s Profile`
      }
    },
    {
      path: '/projects',
      name: 'projects',
      component: ProjectsView,
      meta: {
        requiresAuth: true,
        title: 'Projects'
      }
    },
    {
      path: '/projects/:id',
      name: 'project-detail',
      component: ProjectDetailView,
      props: true,
      meta: {
        requiresAuth: true,
        title: 'Project Details'
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
      path: '/users',
      name: 'users',
      component: () => import('../views/UsersListView.vue'),
      meta: {
        requiresAuth: true,
        title: 'Users'
      }
    },
    {
      path: '/devices',
      name: 'devices',
      component: () => import('../views/DevicesListView.vue'),
      meta: {
        requiresAuth: true,
        title: 'Devices'
      }
    },
    {
      path: '/devices/:id',
      name: 'device-view',
      component: () => import('../views/DeviceView.vue'),
      props: true,
      meta: {
        requiresAuth: true,
        title: 'Device Details'
      },
      beforeEnter: (to) => {
        to.meta.title = `Device #${to.params.id}`
      }
    },
    {
      path: '/documentation',
      name: 'documentation',
      component: DocumentationView,
      meta: {
        requiresAuth: true,
        title: 'Documentation'
      }
    },
    {
      path: '/documentation/category/:categoryId',
      name: 'documentation-category',
      component: DocumentationPageView,
      meta: {
        requiresAuth: true,
        title: 'Category Documentation'
      },
      beforeEnter: async (to) => {
        // Set a generic title initially
        to.meta.title = 'Category Documentation';
        to.meta.isCategory = true;
      }
    },
    {
      path: '/documentation/:id',
      name: 'documentation-article',
      component: DocumentationPageView,
      meta: {
        requiresAuth: true,
        title: 'Documentation'
      },
      beforeEnter: async (to) => {
        // Set a generic title initially
        to.meta.title = 'Documentation Article';
        console.log('Setting initial documentation article title');
        
        // Check if this is a ticket note
        if (to.params.id && typeof to.params.id === 'string' && to.params.id.startsWith('ticket-')) {
          const ticketId = to.params.id.replace('ticket-', '');
          to.meta.title = `Ticket #${ticketId} Notes`;
          to.meta.isTicketNote = true;
          to.meta.ticketId = ticketId;
        }
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