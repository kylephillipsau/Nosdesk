// App.vue
<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { computed } from 'vue'
import Navbar from './components/Navbar.vue'
import PageHeader from './components/SiteHeader.vue'
import { useTitleManager } from '@/composables/useTitleManager'

const route = useRoute()
const isBlankLayout = computed(() => route.meta.layout === 'blank')

// Use the centralized title manager
const titleManager = useTitleManager();

// Handle route-based ticket information
const ticketInfo = computed(() => {
  // Handle ticket routes
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

// Computed property to determine if we're on a documentation page
const isDocumentationPage = computed(() => {
  return route.name === 'documentation-article';
});

// Computed property for the current page URL (for display purposes)
const currentPageUrl = computed(() => {
  // Only show URL for certain pages
  if (route.name === 'settings' || route.name === 'profile') {
    return window.location.href;
  }
  return undefined;
});
</script>

<template>
  <!-- Blank layout for login -->
  <RouterView v-if="isBlankLayout" />

  <!-- Default layout with navbar and header -->
  <div v-else class="flex w-full h-screen bg-slate-900 overflow-hidden">
    <!-- Fixed navbar -->
    <Navbar class="fixed left-0 top-0 h-screen w-64 z-20" />
    
    <!-- Main content area -->
    <div class="flex flex-col not-print:pl-64 w-full h-screen">
      <!-- Fixed header -->
      <PageHeader 
        class="not-print:fixed top-0 right-0 left-64 print:left-0 h-16 z-10 border-b border-slate-600 bg-slate-800" 
        :useRouteTitle="!isDocumentationPage"
        :title="titleManager.pageTitle.value"
        :showCreateButton="true"
        :ticket="titleManager.currentTicket.value"
        :document="titleManager.currentDocument.value"
        :is-transitioning="titleManager.isTransitioning.value"
        :pageUrl="currentPageUrl"
        @update-ticket-title="titleManager.updateTicketTitle"
        @preview-ticket-title="titleManager.previewTicketTitle"
        @update-document-title="titleManager.updateDocumentTitle"
        @preview-document-title="titleManager.previewDocumentTitle"
        @update-document-icon="titleManager.updateDocumentIcon"
      />
      
      <!-- Scrollable content -->
      <main class="flex-1 not-print:pt-16 overflow-hidden">
        <RouterView 
          v-slot="{ Component }" 
          @update:ticket="titleManager.setTicket"
          @update:document="titleManager.setDocument"
          @update:title="titleManager.setCustomTitle"
        >
          <Transition 
            name="fade" 
            mode="out-in"
            @before-enter="titleManager.startTransition"
            @after-enter="titleManager.endTransition"
            @before-leave="titleManager.startTransition"
            @after-leave="() => {
              if (route.name !== 'ticket') titleManager.clearTicket();
              if (route.name !== 'documentation-article') titleManager.clearDocument();
              titleManager.endTransition();
            }"
          >
            <component :is="Component" :key="$route.fullPath" class="h-full overflow-auto" />
          </Transition>
        </RouterView>
      </main>
    </div>
  </div>
</template>

<style>
/* Global styles */
html, body {
  background-color: rgb(15 23 42); /* bg-slate-900 */
  min-height: 100vh;
  overflow: hidden; /* Prevent double scrollbars */
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