<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import documentationService from '@/services/documentationService'
import { useDocumentationNavStore } from '@/stores/documentationNav'
import type { Category, Article } from '@/services/documentationService'

const route = useRoute()
const categories = ref<Category[]>([])
const isLoading = ref(true)
const docNavStore = useDocumentationNavStore()

// Load categories and articles
const loadCategories = async () => {
  isLoading.value = true
  try {
    categories.value = await documentationService.getCategories()
  } catch (error) {
    console.error('Error loading categories:', error)
  } finally {
    isLoading.value = false
  }
}

// Find parent categories for auto-expansion
const findParentCategories = (targetPath: string): string[] => {
  const parents: string[] = []
  categories.value.forEach(category => {
    if (targetPath.includes(category.id) && route.params.id !== category.id) {
      parents.push(category.id)
    }
  })
  return parents
}

// Watch route changes to auto-expand categories
watch(() => route.path, (newPath) => {
  const parentCategories = findParentCategories(newPath)
  parentCategories.forEach(categoryId => {
    docNavStore.expandFolder(categoryId)
  })
})

onMounted(async () => {
  await loadCategories()
  
  // Set initial sidebar state based on screen size
  docNavStore.updateSidebarForScreenSize()

  // Handle window resize
  const handleResize = () => {
    docNavStore.updateSidebarForScreenSize()
  }
  
  window.addEventListener('resize', handleResize)
  
  // Cleanup
  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
  })
})
</script>

<template>
  <section class="relative h-full flex flex-col overflow-hidden">
    <h3 class="px-4 text-sm font-medium text-gray-400 uppercase mb-2 flex-shrink-0">Recent Documents</h3>

      <div class="flex-1 overflow-y-auto p-2">
        <ul class="space-y-4 pt-2 border-t-gray-700 border-t-2">
          <li v-for="category in categories" :key="category.id" class="space-y-2">
            <h3
              class="text-sm font-medium text-slate-300 px-2 py-1 cursor-pointer hover:text-white hover:bg-slate-700 rounded transition-colors"
              @click="docNavStore.toggleFolder(category.id)"
            >
              {{ category.name }}
            </h3>
            
            <ul v-if="docNavStore.expandedFolders[category.id]" class="space-y-1">
              <li v-for="articleId in category.articles" :key="articleId" class="pl-2">
                <RouterLink
                  :to="`/documentation/${articleId}`"
                  class="block px-2 py-1 text-sm text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
                  :class="{ 'bg-slate-700 text-white': route.params.id === articleId }"
                >
                  {{ typeof articleId === 'string' ? articleId : (articleId as any).title }}
                </RouterLink>
              </li>
            </ul>
          </li>
        </ul>
      </div>
  </section>
</template>

<style scoped>
.overflow-y-auto::-webkit-scrollbar {
  width: 4px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background-color: #475569;
  border-radius: 2px;
}
</style>