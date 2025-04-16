<script setup lang="ts">
import { RouterLink, useRoute, useRouter } from 'vue-router'
import DocumentationNav from '@/components/documentationComponents/DocumentationNav.vue'
import RecentTickets from '@/components/RecentTickets.vue'
import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue'

const route = useRoute()
const router = useRouter()
const searchTerm = ref('')

// State for collapsed/expanded navbar
const isCollapsed = ref(false)
const isSmallScreen = ref(false)

// Provide/inject for sharing with App.vue
const emit = defineEmits(['update:collapsed'])

// Toggle navbar collapsed state
const toggleNav = () => {
  isCollapsed.value = !isCollapsed.value
  emit('update:collapsed', isCollapsed.value)
  // Store preference in localStorage
  localStorage.setItem('navbarCollapsed', isCollapsed.value.toString())
}

// Check screen size and set navbar state accordingly
const checkScreenSize = () => {
  const previouslySmall = isSmallScreen.value
  isSmallScreen.value = window.innerWidth < 1024 // lg breakpoint
  
  // Get stored preference (if any)
  const storedPref = localStorage.getItem('navbarCollapsed')
  
  // If screen size changed from large to small
  if (isSmallScreen.value && !previouslySmall) {
    isCollapsed.value = true // Always collapse on small screens
  } 
  // If screen size changed from small to large
  else if (!isSmallScreen.value && previouslySmall) {
    // On larger screens, use the stored preference or default to expanded
    isCollapsed.value = storedPref === 'true'
  }
  // On initial load for large screens
  else if (!isSmallScreen.value && !previouslySmall && !storedPref) {
    // Default to expanded on large screens if no preference is stored
    isCollapsed.value = false
  }
  
  // Emit the current state
  emit('update:collapsed', isCollapsed.value)
}

// Initialize on mount
onMounted(() => {
  // Initial state - check if we have a stored preference
  const storedPref = localStorage.getItem('navbarCollapsed')
  
  // Check screen size first
  isSmallScreen.value = window.innerWidth < 1024
  
  // Set initial state based on screen size and preference
  if (isSmallScreen.value) {
    // On small screens, always start collapsed
    isCollapsed.value = true
  } else {
    // On larger screens, use stored preference or default to expanded
    isCollapsed.value = storedPref === 'true'
  }
  
  // Emit initial state
  emit('update:collapsed', isCollapsed.value)
  
  // Add resize listener
  window.addEventListener('resize', checkScreenSize)
})

// Clean up on unmount
onBeforeUnmount(() => {
  window.removeEventListener('resize', checkScreenSize)
})

// Computed property to check if we're on a documentation page
const isDocumentationPage = computed(() => {
  return route.path.startsWith('/documentation')
})

// Handle documentation search
const handleDocSearch = (query: string) => {
  if (isDocumentationPage.value) {
    // If already on documentation page, update the search query
    searchTerm.value = query
  } else {
    // If not on documentation page, navigate to documentation with search query
    router.push({
      path: '/documentation',
      query: { search: query }
    })
  }
}

// Navigation links data
const navLinks = [
  {
    to: '/',
    icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
    text: 'Dashboard',
    exact: true
  },
  {
    to: '/tickets',
    icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4',
    text: 'Tickets'
  },
  {
    to: '/projects',
    icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
    text: 'Projects'
  },
  {
    to: '/users',
    icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
    text: 'Users'
  },
  {
    to: '/devices',
    icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z',
    text: 'Devices'
  },
  {
    to: '/documentation',
    icon: 'M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253',
    text: 'Documentation'
  }
]

// Helper function to check if a route is active
const isRouteActive = (path: string, exact = false) => {
  if (exact) {
    return route.path === path
  }
  return route.path.startsWith(path)
}
</script>

<template>
  <!-- Desktop Sidebar - Hidden on small screens -->
  <nav 
    class="h-screen bg-slate-800 border-r border-black flex flex-col flex-shrink-0 print:hidden gap-1 transition-all duration-300 ease-in-out overflow-hidden lg:fixed lg:left-0 lg:top-0 lg:z-20 lg:flex"
    :class="[isCollapsed ? 'lg:w-16' : 'lg:w-64']"
  >
    <div class="flex flex-col p-2 space-y-2 flex-shrink-0 gap-2">
      <RouterLink to="/" class="flex items-center mb-8 hover:opacity-80 transition-opacity text-[#FDBD10] select-none">
        <img 
          alt="Nosdesk Logo" 
          class="px-4 py-2 Nosdesk Logo"
          :class="{ 'mx-auto': isCollapsed }"
          src="@/assets/logo.svg"
        />
      </RouterLink>

      <div class="flex flex-col space-y-2">
        <RouterLink 
          v-for="link in navLinks" 
          :key="link.to"
          :to="link.to" 
          class="rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="[
            isRouteActive(link.to, link.exact) ? 'bg-slate-700' : '',
            isCollapsed ? 'px-2 py-2 justify-center' : 'px-4 py-2'
          ]"
          :title="isCollapsed ? link.text : ''"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="link.icon" />
          </svg>
          <span v-if="!isCollapsed">{{ link.text }}</span>
        </RouterLink>
      </div>
    </div>

    <!-- Show both RecentTickets and DocumentationNav when not collapsed -->
    <div class="flex-1 min-h-0 flex flex-col overflow-hidden gap-1" v-if="!isCollapsed">
      <!-- Recent Tickets section -->
      <div class="flex-shrink-0 min-h-0 overflow-y-auto">
        <RecentTickets />
      </div>

      <!-- Documentation section -->
      <div class="flex-1 min-h-0 overflow-y-auto">
        <DocumentationNav @search="handleDocSearch" />
      </div>
    </div>
    
    <!-- Toggle button at the bottom of sidebar -->
    <div class="mt-auto border-t border-slate-700 p-2">
      <button 
        @click="toggleNav" 
        class="w-full flex items-center justify-center md:justify-between p-2 text-slate-400 hover:text-white hover:bg-slate-700/50 rounded-md transition-colors"
        aria-label="Toggle sidebar"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path v-if="isCollapsed" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
          <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        </svg>
        <span v-if="!isCollapsed" class="ml-2 text-sm">Collapse Sidebar</span>
      </button>
    </div>
  </nav>

  <!-- Mobile Bottom Navigation -->
  <nav class="fixed bottom-0 left-0 right-0 bg-slate-800 border-t border-slate-700 z-20 lg:hidden print:hidden">
    <div class="flex justify-around items-center h-16">
      <RouterLink 
        v-for="link in navLinks" 
        :key="link.to"
        :to="link.to" 
        class="flex flex-col items-center justify-center px-2 py-1 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex-1"
        :class="isRouteActive(link.to, link.exact) ? 'bg-slate-700/50 text-blue-400' : 'text-slate-300'"
      >
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="link.icon" />
        </svg>
        <span class="text-xs mt-1 truncate w-full text-center">{{ link.text }}</span>
      </RouterLink>
    </div>
  </nav>

  <!-- Show semi-transparent overlay on small screens when nav is expanded -->
  <div
    v-if="isSmallScreen && !isCollapsed"
    class="fixed inset-0 bg-black bg-opacity-50 z-10 lg:hidden"
    @click="toggleNav"
  ></div>
</template>

<style scoped>
/* Add transition for smooth animation */
.transition-all {
  transition-property: all;
}
</style>