<script setup lang="ts">
import { RouterLink } from 'vue-router'
import RecentTickets from './RecentTickets.vue'
</script>

<template>
  <nav class="w-64 h-screen bg-slate-800 border-r border-black flex flex-col flex-shrink-0 print:hidden">
    <!-- Fixed header section -->
    <div class="p-2 space-y-2 flex-shrink-0">
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
      </div>
    </div>

    <!-- Scrollable section -->
    <div class="flex-1 min-h-0">
      <RecentTickets />
    </div>
  </nav>
</template>