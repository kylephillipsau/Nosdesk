<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import documentationService from '@/services/documentationService'
import { useDocumentationNavStore } from '@/stores/documentationNav'
import type { Category, Article } from '@/services/documentationService'

const route = useRoute()
const router = useRouter()
const categories = ref<Category[]>([])
const articles = ref<Record<string, Article>>({})
const isLoading = ref(true)
const docNavStore = useDocumentationNavStore()

// Load categories and articles
const loadCategories = async () => {
  isLoading.value = true
  try {
    categories.value = await documentationService.getCategories()
    
    // Load all articles to get their metadata including icons
    const allArticles = await documentationService.getAllArticles()
    
    // Create a map of article IDs to article data
    articles.value = allArticles.reduce((acc, article) => {
      acc[article.id] = article
      return acc
    }, {} as Record<string, Article>)
  } catch (error) {
    console.error('Error loading categories:', error)
  } finally {
    isLoading.value = false
  }
}

// Get article title by ID
const getArticleTitle = (articleId: string): string => {
  if (articles.value[articleId]) {
    return articles.value[articleId].title
  }
  return articleId
}

// Get article icon by ID
const getArticleIcon = (articleId: string): string => {
  if (articles.value[articleId] && articles.value[articleId].icon) {
    return articles.value[articleId].icon || '📄'
  }
  return '📄' // Default icon
}

// Check if an icon is an SVG
const isIconSvg = (icon: string): boolean => {
  return icon.startsWith('<svg')
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

// Handle folder click - navigate to category note or toggle folder
const handleFolderClick = (categoryId: string) => {
  // Check if we're already on this category's page
  const isCategoryPage = route.name === 'documentation-category' && 
                         route.params.categoryId === categoryId;
  
  if (isCategoryPage) {
    // If already on the category page, just toggle the folder
    docNavStore.toggleFolder(categoryId);
  } else {
    // If not on the category page, navigate to it and ensure folder is expanded
    router.push(`/documentation/category/${categoryId}`);
    docNavStore.expandFolder(categoryId);
  }
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
            <!-- Folder Header - Acts as both folder toggle and link to folder note -->
            <h3
              class="text-sm font-medium px-2 py-1 cursor-pointer hover:text-white hover:bg-slate-700 rounded transition-colors flex items-center"
              :class="{
                'text-slate-300': !(route.name === 'documentation-category' && route.params.categoryId === category.id),
                'text-white bg-slate-700': route.name === 'documentation-category' && route.params.categoryId === category.id
              }"
              @click="handleFolderClick(category.id)"
            >
              <!-- Expand/Collapse Arrow -->
              <span class="mr-1 text-slate-400 transition-transform duration-200" :class="{ 'rotate-90': docNavStore.expandedFolders[category.id] }">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </span>
              
              <!-- Folder Icon -->
              <span class="mr-2 flex-shrink-0 w-5 h-5 flex items-center justify-center">
                <span v-if="category.icon && !isIconSvg(category.icon)" class="text-lg">{{ category.icon }}</span>
                <span v-else-if="category.icon && isIconSvg(category.icon)" v-html="category.icon" class="w-4 h-4"></span>
                <span v-else class="text-lg">{{ docNavStore.expandedFolders[category.id] ? '📂' : '📁' }}</span>
              </span>
              
              <!-- Folder Name -->
              <span>{{ category.name }}</span>
            </h3>
            
            <!-- Child Articles with vertical line (only shown when folder is expanded) -->
            <div v-if="docNavStore.expandedFolders[category.id]" class="pl-3 ml-2">
              <ul class="space-y-1 border-l border-slate-700">
                <li v-for="articleId in category.articles" :key="articleId">
                  <RouterLink
                    :to="`/documentation/${articleId}`"
                    class="flex items-center gap-2 px-2 py-1 text-sm text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
                    :class="{ 'bg-slate-700 text-white': route.params.id === articleId }"
                  >
                    <!-- Document Icon -->
                    <span class="mr-2 flex-shrink-0 w-5 h-5 flex items-center justify-center">
                      <span v-if="!isIconSvg(getArticleIcon(articleId))" class="text-lg">{{ getArticleIcon(articleId) }}</span>
                      <span v-else v-html="getArticleIcon(articleId)" class="w-4 h-4"></span>
                    </span>
                    
                    <!-- Document Title -->
                    <span class="truncate">{{ getArticleTitle(articleId) }}</span>
                  </RouterLink>
                </li>
              </ul>
            </div>
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