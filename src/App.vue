// App.vue
<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { computed, ref } from 'vue'
import Navbar from './components/Navbar.vue'
import PageHeader from './components/SiteHeader.vue'

const route = useRoute()
const isBlankLayout = computed(() => route.meta.layout === 'blank')

// Add reactive ticket data
const currentTicket = ref<{ id: number; title: string } | null>(null);
const isTransitioning = ref(false);

const handleTicketData = (ticketData: { id: number; title: string } | null) => {
  currentTicket.value = ticketData;
};

const handleUpdateTicketTitle = (newTitle: string) => {
  if (currentTicket.value) {
    currentTicket.value.title = newTitle;
  }
};

const handleBeforeEnter = () => {
  isTransitioning.value = true;
};

const handleAfterEnter = () => {
  isTransitioning.value = false;
};

const handleBeforeLeave = () => {
  isTransitioning.value = true;
};

const handleAfterLeave = () => {
  // Only clear ticket data after the leave animation completes
  if (route.name !== 'ticket') {
    currentTicket.value = null;
  }
  isTransitioning.value = false;
};

// Get ticket information from the route if we're in a ticket view
const ticketInfo = computed(() => {
  if (route.name === 'ticket' && route.params.id) {
    const title = route.meta.pageTitle as string || '';
    // Remove the ticket number from the title if it exists
    const titleWithoutNumber = title.replace(/^#\d+\s*/, '');
    return {
      id: Number(route.params.id),
      title: titleWithoutNumber
    }
  }
  return null
})
</script>

<template>
  <!-- Blank layout for login -->
  <RouterView v-if="isBlankLayout" />

  <!-- Default layout with navbar and header -->
  <div v-else class="flex w-full h-screen bg-slate-900">
    <!-- Fixed navbar -->
    <Navbar class="fixed left-0 top-0 h-screen w-64 z-20" />
    
    <!-- Main content area -->
    <div class="flex flex-col pl-64 w-full min-h-screen">
      <!-- Fixed header -->
      <PageHeader 
        class="fixed top-0 right-0 left-64 h-16 z-10 border-b border-slate-600 bg-slate-800" 
        :useRouteTitle="true" 
        :showCreateButton="true"
        :ticket="currentTicket"
        :is-transitioning="isTransitioning"
        @update-ticket-title="handleUpdateTicketTitle"
      />
      
      <!-- Scrollable content -->
      <main class="flex-1 pt-16">
        <RouterView 
          v-slot="{ Component }" 
          @update:ticket="handleTicketData"
        >
          <Transition 
            name="fade" 
            mode="out-in"
            @before-enter="handleBeforeEnter"
            @after-enter="handleAfterEnter"
            @before-leave="handleBeforeLeave"
            @after-leave="handleAfterLeave"
          >
            <component :is="Component" :key="$route.fullPath" />
          </Transition>
        </RouterView>
      </main>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

<style>
/* Global styles */
html, body {
  background-color: rgb(15 23 42); /* bg-slate-900 */
  min-height: 100vh;
}

/* Custom scrollbar styles */
::-webkit-scrollbar {
  width: 0.875rem;  /* 14px at default font size */
  height: 0.875rem;
}

::-webkit-scrollbar-track {
  background: rgb(30 41 59); /* bg-slate-800 */
}

::-webkit-scrollbar-thumb {
  background: rgb(100 116 139); /* bg-slate-500 */
  border-radius: 0.25rem;
}

::-webkit-scrollbar-thumb:hover {
  background: rgb(148 163 184); /* bg-slate-400 */
}

/* Firefox scrollbar styles */
* {
  scrollbar-width: auto; /* Changed from 'thin' to 'auto' for wider scrollbar */
  scrollbar-color: rgb(100 116 139) rgb(30 41 59); /* thumb track */
}

/* Ensure reduced motion preferences are respected */
@media (prefers-reduced-motion: reduce) {
  ::-webkit-scrollbar-thumb {
    transition: none;
  }
}

/* Adjust scrollbar size for smaller screens */
@media (max-width: 640px) {
  ::-webkit-scrollbar {
    width: 0.75rem;  /* 12px at default font size */
    height: 0.75rem;
  }
}
</style>