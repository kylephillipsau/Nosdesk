<script setup lang="ts">
import { RouterLink } from 'vue-router'
import RecentTickets from './RecentTickets.vue'
import { ref } from 'vue'

const searchTerm = ref('')
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

      <div class="px-2">
        <div class="relative w-full">
          <input 
            v-model="searchTerm" 
            type="text" 
            placeholder="Search..." 
            class="w-full pl-10 pr-4 py-1 rounded-full bg-slate-700 text-white placeholder-gray-400 focus:outline-none focus:bg-slate-600 transition-all duration-300"
          />
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>
      </div>

      <div class="flex flex-col space-y-2">
        <RouterLink 
          to="/" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700"
        >
          Dashboard
        </RouterLink>
        <RouterLink 
          to="/tickets" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700"
        >
          Tickets
        </RouterLink>
        <RouterLink 
          to="/projects" 
          class="px-4 py-2 rounded-lg transition-colors duration-200 text-white hover:bg-slate-700"
        >
          Projects
        </RouterLink>
      </div>
    </div>

    <!-- Scrollable section -->
    <div class="flex-1 min-h-0">
      <RecentTickets />
    </div>
  </nav>
</template>