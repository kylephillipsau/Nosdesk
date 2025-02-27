// App.vue
<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { computed, ref } from 'vue'
import Navbar from './components/Navbar.vue'
import PageHeader from './components/SiteHeader.vue'
import { usePageTitle } from '@/composables/usePageTitle'

const route = useRoute()
const isBlankLayout = computed(() => route.meta.layout === 'blank')
const { setCustomTitle } = usePageTitle();

// Add reactive ticket data
const currentTicket = ref<{ id: number; title: string } | null>(null);
const currentDocument = ref<{ id: string; title: string; icon: string } | null>(null);
const documentationTitle = ref<string | null>(null);
const isTransitioning = ref(false);

const handleTicketData = (ticketData: { id: number; title: string } | null) => {
  currentTicket.value = ticketData;
};

const handleDocumentData = (documentData: { id: string; title: string; icon: string } | null) => {
  console.log('App received document data:', documentData);
  currentDocument.value = documentData;
  if (documentData) {
    documentationTitle.value = documentData.title;
    setCustomTitle(documentData.title);
  }
};

const handleUpdateTicketTitle = (newTitle: string) => {
  if (currentTicket.value) {
    currentTicket.value.title = newTitle;
    // Don't update page title here, as it's handled by the route
  }
};

const handleUpdateDocumentTitle = (newTitle: string) => {
  if (currentDocument.value) {
    currentDocument.value.title = newTitle;
    documentationTitle.value = newTitle;
    setCustomTitle(newTitle);
  }
};

const handleUpdateDocumentIcon = (newIcon: string) => {
  if (currentDocument.value) {
    currentDocument.value.icon = newIcon;
    // No need to update page title for icon changes
  }
};

// Handle direct title updates from documentation pages
const handleUpdateTitle = (title: string) => {
  console.log('App received title update:', title);
  documentationTitle.value = title;
  setCustomTitle(title);
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
  
  // Clear documentation title if leaving documentation page
  if (route.name !== 'documentation-article') {
    documentationTitle.value = null;
    currentDocument.value = null;
  }
  
  isTransitioning.value = false;
};

// Get ticket information from the route if we're in a ticket view
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

// Computed property for the current page title
const currentPageTitle = computed(() => {
  console.log('Computing currentPageTitle:', {
    isDocumentationPage: isDocumentationPage.value,
    documentationTitle: documentationTitle.value
  });
  
  if (isDocumentationPage.value && documentationTitle.value) {
    return documentationTitle.value;
  }
  return undefined;
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
  <div v-else class="flex w-full h-screen bg-slate-900">
    <!-- Fixed navbar -->
    <Navbar class="fixed left-0 top-0 h-screen w-64 z-20" />
    
    <!-- Main content area -->
    <div class="flex flex-col not-print:pl-64 w-full min-h-screen">
      <!-- Fixed header -->
      <PageHeader 
        class="not-print:fixed top-0 right-0 left-64 print:left-0 h-16 z-10 border-b border-slate-600 bg-slate-800" 
        :useRouteTitle="!isDocumentationPage"
        :title="currentPageTitle"
        :showCreateButton="true"
        :ticket="currentTicket"
        :document="currentDocument"
        :is-transitioning="isTransitioning"
        :pageUrl="currentPageUrl"
        @update-ticket-title="handleUpdateTicketTitle"
        @update-document-title="handleUpdateDocumentTitle"
        @update-document-icon="handleUpdateDocumentIcon"
      />
      
      <!-- Debug route info -->
      <div v-if="false" class="hidden">
        Current route: {{ route.name }}, 
        isDocumentationPage: {{ isDocumentationPage }},
        documentationTitle: {{ documentationTitle }},
        currentDocument: {{ currentDocument }}
      </div>
      
      <!-- Scrollable content -->
      <main class="flex-1 not-print:pt-16">
        <RouterView 
          v-slot="{ Component }" 
          @update:ticket="handleTicketData"
          @update:document="handleDocumentData"
          @update:title="handleUpdateTitle"
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