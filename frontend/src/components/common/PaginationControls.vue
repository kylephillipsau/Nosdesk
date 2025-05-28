<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue'

const props = defineProps<{
  currentPage?: number
  totalPages?: number
  pageSize?: number
  pageSizeOptions?: number[]
  showImport?: boolean
}>()

const emit = defineEmits<{
  'update:currentPage': [page: number]
  'update:pageSize': [size: number]
  'import': []
}>()

// Add isMobile ref to track screen size
const isMobile = ref(false)
const showPageInput = ref(false)
const pageInputValue = ref('')
const pageInput = ref<HTMLInputElement | null>(null)

// Function to check screen size
const checkScreenSize = () => {
  isMobile.value = window.innerWidth < 768 // md breakpoint
}

// Initialize on mount
onMounted(() => {
  checkScreenSize()
  window.addEventListener('resize', checkScreenSize)
})

// Clean up on unmount
onBeforeUnmount(() => {
  window.removeEventListener('resize', checkScreenSize)
})

// Watch for currentPage changes to update input value
watch(() => props.currentPage, (newPage) => {
  if (newPage) {
    pageInputValue.value = newPage.toString()
  }
}, { immediate: true })

// Pagination methods
const changePage = (page: number) => {
  if (page >= 1 && page <= (props.totalPages || 1)) {
    emit('update:currentPage', page)
  }
}

const changePageSize = (event: Event) => {
  const select = event.target as HTMLSelectElement
  emit('update:pageSize', parseInt(select.value))
}

// Handle direct page input
const handlePageInput = () => {
  const page = parseInt(pageInputValue.value)
  if (!isNaN(page) && page >= 1 && page <= (props.totalPages || 1)) {
    changePage(page)
  } else {
    // Reset to current page if invalid
    pageInputValue.value = (props.currentPage || 1).toString()
  }
  showPageInput.value = false
}

const handlePageInputKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    handlePageInput()
  } else if (event.key === 'Escape') {
    pageInputValue.value = (props.currentPage || 1).toString()
    showPageInput.value = false
  }
}

// Generate smart page numbers for pagination
const pageNumbers = computed(() => {
  if (!props.totalPages || props.totalPages <= 1) return []
  
  const currentPage = props.currentPage || 1
  const totalPages = props.totalPages
  
  // For mobile, show fewer pages
  const maxVisible = isMobile.value ? 3 : 5
  
  if (totalPages <= maxVisible + 2) {
    // Show all pages if total is small
    return Array.from({ length: totalPages }, (_, i) => i + 1)
  }
  
  const pages: (number | string)[] = []
  
  // Always show first page
  pages.push(1)
  
  // Calculate range around current page
  const start = Math.max(2, currentPage - Math.floor(maxVisible / 2))
  const end = Math.min(totalPages - 1, currentPage + Math.floor(maxVisible / 2))
  
  // Add ellipsis after first page if needed
  if (start > 2) {
    pages.push('...')
  }
  
  // Add pages around current
  for (let i = start; i <= end; i++) {
    pages.push(i)
  }
  
  // Add ellipsis before last page if needed
  if (end < totalPages - 1) {
    pages.push('...')
  }
  
  // Always show last page if it's not already included
  if (totalPages > 1) {
    pages.push(totalPages)
  }
  
  return pages
})

// Default page size options if not provided
const defaultPageSizeOptions = [10, 25, 50, 100]

// Computed values for better UX
const currentPageDisplay = computed(() => props.currentPage || 1)
const totalPagesDisplay = computed(() => props.totalPages || 1)
const hasMultiplePages = computed(() => totalPagesDisplay.value > 1)
</script>

<template>
  <div class="flex-shrink-0 bg-slate-800 border-t border-slate-700">
    <!-- Mobile Layout -->
    <div v-if="isMobile" class="flex flex-col gap-3 p-3">
      <!-- Top row: Page info and navigation -->
      <div class="flex items-center justify-between">
        <!-- Page info with input -->
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-400">Page</span>
                     <div v-if="showPageInput" class="flex items-center gap-1">
             <input
               v-model="pageInputValue"
               @blur="handlePageInput"
               @keydown="handlePageInputKeydown"
               type="number"
               :min="1"
               :max="totalPagesDisplay"
               class="w-16 px-2 py-1 text-sm bg-slate-700 border border-slate-600 text-white rounded focus:ring-blue-500 focus:border-blue-500 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
               ref="pageInput"
             />
             <span class="text-sm text-gray-400">of {{ totalPagesDisplay }}</span>
           </div>
                     <button
             v-else
             @click="showPageInput = true; $nextTick(() => pageInput?.focus())"
             class="text-sm text-blue-400 hover:text-blue-300 underline font-mono"
           >
             {{ currentPageDisplay }} of {{ totalPagesDisplay }}
           </button>
        </div>
        
        <!-- Navigation buttons -->
        <div v-if="hasMultiplePages" class="flex items-center gap-1">
          <button 
            @click="changePage(currentPageDisplay - 1)"
            :disabled="currentPageDisplay <= 1"
            :class="[
              'p-2 rounded-md text-sm transition-colors',
              currentPageDisplay <= 1 
                ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
                : 'bg-slate-700 text-white hover:bg-slate-600'
            ]"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          
          <button 
            @click="changePage(currentPageDisplay + 1)"
            :disabled="currentPageDisplay >= totalPagesDisplay"
            :class="[
              'p-2 rounded-md text-sm transition-colors',
              currentPageDisplay >= totalPagesDisplay 
                ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
                : 'bg-slate-700 text-white hover:bg-slate-600'
            ]"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>
      
      <!-- Bottom row: Page size and import -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-400">
          <select 
            :value="pageSize" 
            @change="changePageSize"
            class="bg-slate-700 border border-slate-600 text-white text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 py-1 px-2"
          >
            <option 
              v-for="size in (pageSizeOptions || defaultPageSizeOptions)" 
              :key="size" 
              :value="size"
            >
              {{ size }}
            </option>
          </select>
          <span>per page</span>
        </div>
        
        <button
          v-if="showImport"
          @click="emit('import')"
          class="px-3 py-1 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:ring-2 focus:outline-none focus:ring-blue-800 flex items-center gap-1 transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0l-4 4m4-4v12" />
          </svg>
          Import
        </button>
      </div>
    </div>

    <!-- Desktop Layout -->
    <div v-else class="flex items-center justify-between p-3">
      <!-- Left: Page size selector -->
      <div class="flex items-center gap-2 text-sm text-gray-400 min-w-0 flex-shrink-0">
        <span>Show</span>
        <select 
          :value="pageSize" 
          @change="changePageSize"
          class="bg-slate-700 border border-slate-600 text-white text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 py-1 px-2"
        >
          <option 
            v-for="size in (pageSizeOptions || defaultPageSizeOptions)" 
            :key="size" 
            :value="size"
          >
            {{ size }}
          </option>
        </select>
        <span>per page</span>
      </div>
      
      <!-- Center: Page navigation (fixed width container) -->
      <div v-if="hasMultiplePages" class="flex items-center gap-4 min-w-0 flex-1 justify-center">
        <!-- Previous page button -->
        <button 
          @click="changePage(currentPageDisplay - 1)"
          :disabled="currentPageDisplay <= 1"
          :class="[
            'p-2 rounded-md text-sm transition-colors flex-shrink-0',
            currentPageDisplay <= 1 
              ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
              : 'bg-slate-700 text-white hover:bg-slate-600'
          ]"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        
        <!-- Page numbers container with FIXED width to prevent layout shifts -->
        <div class="flex items-center gap-1 w-96 justify-center">
          <template v-for="page in pageNumbers" :key="page">
            <button 
              v-if="typeof page === 'number'"
              @click="changePage(page)"
              :class="[
                'py-1 text-sm rounded-md transition-colors flex-shrink-0 w-10 text-center',
                page === currentPageDisplay
                  ? 'bg-blue-600 text-white' 
                  : 'bg-slate-700 text-white hover:bg-slate-600'
              ]"
            >
              {{ page }}
            </button>
            <span 
              v-else
              class="py-1 text-sm text-gray-400 flex-shrink-0 w-10 text-center"
            >
              ...
            </span>
          </template>
        </div>
        
        <!-- Next page button -->
        <button 
          @click="changePage(currentPageDisplay + 1)"
          :disabled="currentPageDisplay >= totalPagesDisplay"
          :class="[
            'p-2 rounded-md text-sm transition-colors flex-shrink-0',
            currentPageDisplay >= totalPagesDisplay 
              ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
              : 'bg-slate-700 text-white hover:bg-slate-600'
          ]"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </button>
      </div>
      
      <!-- Right: Page info and Import button -->
      <div class="flex items-center gap-3 min-w-0 flex-shrink-0">
        <!-- Page info with direct input -->
        <div class="flex items-center gap-2 text-sm text-gray-400">
          <span>Page</span>
                     <div v-if="showPageInput" class="flex items-center gap-1">
             <input
               v-model="pageInputValue"
               @blur="handlePageInput"
               @keydown="handlePageInputKeydown"
               type="number"
               :min="1"
               :max="totalPagesDisplay"
               class="w-16 px-2 py-1 text-sm bg-slate-700 border border-slate-600 text-white rounded focus:ring-blue-500 focus:border-blue-500 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
               ref="pageInput"
             />
             <span>of {{ totalPagesDisplay }}</span>
           </div>
                     <button
             v-else
             @click="showPageInput = true; $nextTick(() => pageInput?.focus())"
             class="text-blue-400 hover:text-blue-300 underline transition-colors font-mono"
           >
             {{ currentPageDisplay }} of {{ totalPagesDisplay }}
           </button>
        </div>
        
        <!-- Import button -->
        <button
          v-if="showImport"
          @click="emit('import')"
          class="px-3 py-1 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:ring-2 focus:outline-none focus:ring-blue-800 flex items-center gap-1 transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0l-4 4m4-4v12" />
          </svg>
          Import
        </button>
      </div>
    </div>
  </div>
</template> 