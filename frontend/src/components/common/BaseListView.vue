<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{
  title: string
  searchQuery: string
  isLoading: boolean
  isEmpty: boolean
  addButtonText?: string
  showAddButton?: boolean
  emptyMessage?: string
  error?: string | null
  // Filter props
  filters?: Array<{
    name: string
    value: string
    options: Array<{ value: string, label: string }>
    width?: string
  }>
  // Results count
  resultsCount?: number
  // Sorting props
  sortField?: string
  sortDirection?: 'asc' | 'desc'
  // Column definitions for the table
  columns?: Array<{
    field: string
    label: string
    sortable?: boolean
    width?: string
    class?: string
  }>
  // Selection props
  selectedItems?: string[]
  itemIdField?: string
  enableSelection?: boolean
  visibleItems?: any[]
  // Pagination props
  currentPage?: number
  totalPages?: number
  pageSize?: number
  pageSizeOptions?: number[]
}>()

const emit = defineEmits<{
  'update:searchQuery': [value: string]
  'update:filter': [name: string, value: string]
  'update:sort': [field: string, direction: 'asc' | 'desc']
  'update:selectedItems': [selectedIds: string[]]
  'toggle-selection': [event: Event, itemId: string]
  'toggle-all': [event: Event, checked: boolean]
  'add': []
  'reset-filters': []
  'retry': []
  'update:currentPage': [page: number]
  'update:pageSize': [size: number]
}>()

const defaultEmptyMessage = computed(() => {
  return `No ${props.title.toLowerCase()} found.`
})

// Handle sort toggle
const toggleSort = (field: string) => {
  if (props.sortField === field) {
    // Toggle direction if clicking the same field
    const newDirection = props.sortDirection === 'asc' ? 'desc' : 'asc'
    emit('update:sort', field, newDirection)
  } else {
    // Set new field and default to ascending
    emit('update:sort', field, 'asc')
  }
}

// Handle selection toggle for a single item
const toggleSelection = (event: Event, itemId: string) => {
  // We need to pass the original event to the parent component
  // so it can handle shift/ctrl/cmd key modifiers
  emit('toggle-selection', event, itemId)
}

// Handle toggle all items
const toggleAllItems = (event: Event) => {
  const checkbox = event.target as HTMLInputElement
  emit('toggle-all', event, checkbox.checked)
}

// Compute if all visible items are selected
const allSelected = computed(() => {
  if (!props.selectedItems || !props.visibleItems || props.visibleItems.length === 0) {
    return false
  }
  
  const idField = props.itemIdField || 'id'
  return props.visibleItems.every(item => 
    props.selectedItems?.includes(item[idField].toString())
  )
})

// Pagination methods
const changePage = (page: number) => {
  if (page >= 1 && page <= (props.totalPages || 1)) {
    emit('update:currentPage', page)
  }
}

const changePageSize = (event: Event) => {
  const select = event.target as HTMLSelectElement
  emit('update:pageSize', parseInt(select.value))
  // Reset to first page when changing page size
  emit('update:currentPage', 1)
}

// Generate page numbers for pagination
const pageNumbers = computed(() => {
  if (!props.totalPages || props.totalPages <= 1) return []
  
  const currentPage = props.currentPage || 1
  const totalPages = props.totalPages
  
  // Always show first, last, current, and pages around current
  const pages: number[] = []
  
  // Always add page 1
  pages.push(1)
  
  // Add ellipsis indicator if needed
  if (currentPage > 3) {
    pages.push(-1) // -1 represents ellipsis
  }
  
  // Add pages around current page
  for (let i = Math.max(2, currentPage - 1); i <= Math.min(totalPages - 1, currentPage + 1); i++) {
    if (i > 1 && i < totalPages) {
      pages.push(i)
    }
  }
  
  // Add ellipsis indicator if needed
  if (currentPage < totalPages - 2) {
    pages.push(-1) // -1 represents ellipsis
  }
  
  // Always add last page if it's not already added
  if (totalPages > 1) {
    pages.push(totalPages)
  }
  
  return pages
})

// Default page size options if not provided
const defaultPageSizeOptions = [10, 25, 50, 100]
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Search and filter bar -->
    <div
      v-if="!isLoading && !error"
      class="sticky top-0 z-20 bg-slate-800 border-b border-slate-700 shadow-md"
    >
      <div class="p-2 flex items-center gap-2 flex-wrap">
        <!-- Search input -->
        <div class="relative flex-grow min-w-[150px]">
          <div
            class="absolute inset-y-0 left-0 flex items-center pl-2 pointer-events-none"
          >
            <svg
              class="w-3.5 h-3.5 text-gray-400"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 20 20"
            >
              <path
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"
              />
            </svg>
          </div>
          <input
            type="text"
            :value="searchQuery"
            @input="e => emit('update:searchQuery', (e.target as HTMLInputElement).value)"
            class="block w-full py-1 pl-8 pr-2 text-sm border rounded-md bg-slate-700 border-slate-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
            :placeholder="`Search ${title.toLowerCase()}...`"
          />
        </div>

        <!-- Filters -->
        <template v-if="filters && filters.length > 0">
          <div 
            v-for="filter in filters" 
            :key="filter.name"
            :class="[filter.width || 'w-[120px]']"
          >
            <select
              :value="filter.value"
              @change="e => emit('update:filter', filter.name, (e.target as HTMLSelectElement).value)"
              class="bg-slate-700 border border-slate-600 text-white text-sm rounded-md focus:ring-blue-500 focus:border-blue-500 block w-full py-1 px-2"
            >
              <option
                v-for="option in filter.options"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Reset filters button -->
          <button
            @click="emit('reset-filters')"
            class="px-2 py-1 text-xs font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:ring-2 focus:outline-none focus:ring-blue-800"
          >
            Reset
          </button>
        </template>

        <!-- Add button -->
        <button
          v-if="showAddButton !== false"
          @click="emit('add')"
          class="px-2 py-1 text-xs font-medium text-white bg-green-600 rounded-md hover:bg-green-700 focus:ring-2 focus:outline-none focus:ring-green-800 ml-auto"
        >
          {{ addButtonText || `Add ${title.slice(0, -1)}` }}
        </button>

        <!-- Results count -->
        <div v-if="resultsCount !== undefined" class="text-xs text-gray-400">
          {{ resultsCount }} result{{ resultsCount !== 1 ? "s" : "" }}
        </div>
      </div>
    </div>

    <!-- Main content container -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Loading state -->
      <div v-if="isLoading" class="flex justify-center items-center h-64">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="flex flex-col items-center gap-4 justify-center p-8 text-center text-red-400">
        {{ error }}
        <button @click="emit('retry')" class="mt-4 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors">
          Try Again
        </button>
      </div>

      <!-- Empty state -->
      <div v-else-if="isEmpty" class="p-8 text-center text-gray-400">
        {{ emptyMessage || defaultEmptyMessage }}
      </div>

      <!-- Content -->
      <div v-else class="flex-1 flex flex-col overflow-hidden">
        <!-- Table Header (if columns are provided) -->
        <div v-if="columns && columns.length > 0" class="sticky top-0 z-10 bg-slate-800 border-b border-slate-700 text-sm text-gray-200">
          <div class="flex min-w-[800px]">
            <!-- Selection checkbox in header -->
            <div v-if="enableSelection" class="p-3 w-10 flex-shrink-0">
              <input
                type="checkbox"
                class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                :checked="allSelected && visibleItems && visibleItems.length > 0"
                :indeterminate="selectedItems && selectedItems.length > 0 && !allSelected"
                @change="toggleAllItems"
              />
            </div>
            
            <!-- Custom header prefix slot -->
            <slot name="header-prefix"></slot>
            
            <!-- Generated column headers -->
            <div 
              v-for="column in columns" 
              :key="column.field"
              :class="[
                'p-3 font-medium', 
                column.sortable !== false ? 'cursor-pointer select-none' : '',
                column.width || (column.field === 'id' ? 'w-20 flex-shrink-0' : 'flex-1 min-w-0'),
                column.class || ''
              ]"
              @click="column.sortable !== false && toggleSort(column.field)"
            >
              <div class="flex items-center gap-1">
                {{ column.label }}
                <span v-if="column.sortable !== false && sortField === column.field" class="text-white">
                  {{ sortDirection === 'asc' ? '↑' : '↓' }}
                </span>
              </div>
            </div>
            
            <!-- Slot for additional header columns -->
            <slot name="header-suffix"></slot>
          </div>
        </div>
        
        <!-- Table Body -->
        <div class="flex-1 overflow-y-auto">
          <slot></slot>
        </div>
        
        <!-- Pagination controls -->
        <div 
          v-if="totalPages && totalPages > 1" 
          class="flex items-center justify-between p-2 border-t border-slate-700 bg-slate-800"
        >
          <!-- Page size selector -->
          <div class="flex items-center gap-2 text-sm text-gray-400">
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
          
          <!-- Page navigation -->
          <div class="flex items-center gap-1">
            <!-- Previous page button -->
            <button 
              @click="changePage((currentPage || 1) - 1)"
              :disabled="(currentPage || 1) <= 1"
              :class="[
                'px-2 py-1 rounded-md text-sm',
                (currentPage || 1) <= 1 
                  ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
                  : 'bg-slate-700 text-white hover:bg-slate-600'
              ]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            
            <!-- Page numbers -->
            <div class="flex items-center gap-1">
              <button 
                v-for="page in pageNumbers" 
                :key="page"
                @click="page !== -1 && changePage(page)"
                :class="[
                  'px-3 py-1 text-sm rounded-md mx-0.5',
                  page === -1 
                    ? 'bg-transparent text-gray-400 cursor-default' 
                    : page === (currentPage || 1)
                      ? 'bg-blue-600 text-white' 
                      : 'bg-slate-700 text-white hover:bg-slate-600'
                ]"
                :disabled="page === -1"
              >
                {{ page === -1 ? '...' : page }}
              </button>
            </div>
            
            <!-- Next page button -->
            <button 
              @click="changePage((currentPage || 1) + 1)"
              :disabled="(currentPage || 1) >= (totalPages || 1)"
              :class="[
                'px-2 py-1 rounded-md text-sm',
                (currentPage || 1) >= (totalPages || 1) 
                  ? 'bg-slate-700 text-gray-500 cursor-not-allowed' 
                  : 'bg-slate-700 text-white hover:bg-slate-600'
              ]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
          
          <!-- Page info -->
          <div class="text-sm text-gray-400">
            Page {{ currentPage || 1 }} of {{ totalPages }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Ensure consistent layout */
.flex-1 {
  flex: 1 1 0%;
}

/* Prevent horizontal scrolling unless necessary */
body {
  overflow-x: hidden;
}

/* Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #1e293b;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}
</style> 