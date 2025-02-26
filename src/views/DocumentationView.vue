<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import ArticleCard from '@/components/documentationComponents/ArticleCard.vue'
import documentationService from '@/services/documentationService'
import type { Article, Category } from '@/services/documentationService'

const route = useRoute()
const router = useRouter()
const searchQuery = ref('')
const isLoading = ref(false)
const categories = ref<Category[]>([])

// Load categories and articles
const loadCategoriesAndArticles = async () => {
  isLoading.value = true
  try {
    // Get categories
    categories.value = await documentationService.getCategories()
    
    // For each category, load its articles
    for (const category of categories.value) {
      const articlePromises = category.articles.map(async (articleId) => {
        return await documentationService.getArticleById(articleId)
      })
      
      const articles = await Promise.all(articlePromises)
      category.articles = articles.filter(Boolean) as unknown as string[]
    }
  } catch (error) {
    console.error('Error loading categories and articles:', error)
  } finally {
    isLoading.value = false
  }
}

const filteredArticles = computed(() => {
  const query = searchQuery.value.toLowerCase()
  if (!query) return []
  
  return categories.value.flatMap(category => 
    (category.articles as unknown as Article[]).filter(article => 
      article.title.toLowerCase().includes(query) ||
      article.description.toLowerCase().includes(query)
    )
  )
})

// Watch for route query changes to update search
watch(() => route.query.search, (newSearch) => {
  if (newSearch && typeof newSearch === 'string') {
    searchQuery.value = newSearch
  } else if (!newSearch) {
    searchQuery.value = ''
  }
}, { immediate: true })

// Handle search input
const handleSearch = (query: string) => {
  searchQuery.value = query
  
  // Update URL query parameter without reloading the page
  if (query) {
    router.replace({ query: { ...route.query, search: query } })
  } else {
    const { search, ...restQuery } = route.query
    router.replace({ query: restQuery })
  }
}

// Check for search query on mount and load categories and articles
onMounted(async () => {
  if (route.query.search && typeof route.query.search === 'string') {
    searchQuery.value = route.query.search
  }
  
  await loadCategoriesAndArticles()
})
</script>

<template>
  <div class="h-full">
    <!-- Main Content -->
    <main class="w-full min-h-screen flex justify-center">
      <div class="flex flex-1 flex-col max-w-5xl mx-auto px-4 py-6 gap-4 h-full">
        <!-- Search bar -->
        <div class="mb-6">
          <input 
            v-model="searchQuery"
            type="text"
            placeholder="Search documentation..."
            class="w-full px-4 py-2 bg-slate-700 text-white rounded-lg placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            @input="handleSearch(searchQuery)"
          />
        </div>

        <!-- Search Results -->
        <div v-if="searchQuery" class="space-y-4">
          <h2 class="text-lg font-medium text-white mb-4">Search Results</h2>
          <div v-if="filteredArticles.length === 0" class="text-slate-400">
            No articles found for "{{ searchQuery }}"
          </div>
          <div v-else class="space-y-4">
            <ArticleCard
              v-for="article in filteredArticles"
              :key="article.id"
              v-bind="article"
              :show-full-title="true"
              :show-edit-button="false"
              :show-status="false"
            />
          </div>
        </div>

        <!-- Default Content -->
        <div v-else class="flex flex-col gap-4">
          <div v-for="category in categories" :key="category.id" class="flex flex-col gap-2">
            <h2 class="text-lg font-medium text-white">{{ category.name }}</h2>
            <div class="grid gap-4 md:grid-cols-2">
              <ArticleCard
                v-for="article in category.articles"
                :key="(article as any).id"
                :id="(article as any).id"
                :title="(article as any).title"
                :description="(article as any).description"
                :category="(article as any).category"
                :author="(article as any).author"
                :last-updated="(article as any).lastUpdated"
                :status="(article as any).status"
                :show-edit-button="false"
                :show-status="false"
              />
            </div>
          </div>
        </div>

        <!-- Loading State -->
        <div v-if="isLoading" class="flex justify-center items-center min-h-[200px]">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      </div>
    </main>
  </div>
</template>