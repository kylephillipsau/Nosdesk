// App.vue
<script setup lang="ts">
import { RouterView } from 'vue-router'
import Navbar from './components/Navbar.vue'
</script>

<template>
  <div class="flex h-screen w-full bg-slate-900">
    <Navbar />
    <main class="flex-1 min-w-0 overflow-auto">
      <Transition 
        name="fade" 
        mode="out-in"
      >
        <RouterView :key="$route.fullPath" />
      </Transition>
    </main>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease-in;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>