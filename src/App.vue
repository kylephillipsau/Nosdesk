// App.vue
<script setup lang="ts">
import { RouterView, useRoute } from 'vue-router'
import { computed } from 'vue'
import Navbar from './components/Navbar.vue'
import PageHeader from './components/SiteHeader.vue'

const route = useRoute()
const isBlankLayout = computed(() => route.meta.layout === 'blank')
</script>

<template>
  <!-- Blank layout for login -->
  <RouterView v-if="isBlankLayout" />

  <!-- Default layout with navbar and header -->
  <div v-else class="flex h-screen w-full bg-slate-900">
    <Navbar />
    <main class="flex-1 min-w-0 overflow-auto">
      <PageHeader :useRouteTitle="true" :showCreateButton="true"></PageHeader>
      <RouterView v-slot="{ Component }">
        <Transition 
          name="fade" 
          mode="out-in"
        >
          <component :is="Component" :key="$route.fullPath" />
        </Transition>
      </RouterView>
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