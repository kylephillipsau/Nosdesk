<script setup lang="ts">
import { RouterLink, useRoute, useRouter } from 'vue-router'
import DocumentationNav from '@/components/documentationComponents/DocumentationNav.vue'
import RecentTickets from '@/components/RecentTickets.vue'
import { ref, watch, computed } from 'vue'

const route = useRoute()
const router = useRouter()
const searchTerm = ref('')

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
</script>

<template>
  <nav class="w-64 h-screen bg-slate-800 border-r border-black flex flex-col flex-shrink-0 print:hidden">
    <div class="flex flex-col p-2 space-y-2 flex-shrink-0 gap-2">
      <RouterLink to="/" class="flex items-center mb-8 hover:opacity-80 transition-opacity text-[#FDBD10]">
        <img 
          alt="Vue logo" 
          class="px-4 py-2 not-only:mr-3"
          src="@/assets/logo.svg"
        />
      </RouterLink>

      <div class="flex flex-col space-y-2">
        <RouterLink 
          to="/" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path === '/' }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
          Dashboard
        </RouterLink>
        <RouterLink 
          to="/tickets" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path.startsWith('/tickets') }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
          </svg>
          Tickets
        </RouterLink>
        <RouterLink 
          to="/projects" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path.startsWith('/projects') }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          Projects
        </RouterLink>
        <RouterLink 
          to="/users" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path.startsWith('/users') }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
          Users
        </RouterLink>
        <RouterLink 
          to="/devices" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path.startsWith('/devices') }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          Devices
        </RouterLink>
        <RouterLink 
          to="/documentation" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700 flex items-center gap-3"
          :class="{ 'bg-slate-700': route.path.startsWith('/documentation') }"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
          </svg>
          Documentation
        </RouterLink>
      </div>
    </div>

    <!-- Conditionally show either DocumentationNav or RecentTickets -->
    <div class="flex-1 min-h-0 overflow-hidden">
      <DocumentationNav v-if="isDocumentationPage" @search="handleDocSearch" />
      <RecentTickets v-else />
    </div>
  </nav>
</template>