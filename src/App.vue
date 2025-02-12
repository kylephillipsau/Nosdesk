<script setup lang="ts">
import { RouterView } from 'vue-router'
import Navbar from './components/Navbar.vue'
</script>

<template>
  <div class="flex h-screen w-screen bg-slate-900">
    <Navbar />
    <main class="flex-1 overflow-auto">
      <RouterView :key="$route.fullPath" />
    </main>
  </div>
</template>