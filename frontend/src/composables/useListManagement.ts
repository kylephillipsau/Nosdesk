import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'

interface ListOptions<T> {
  itemIdField?: string
  defaultPageSize?: number
  defaultSortField?: string
  defaultSortDirection?: 'asc' | 'desc'
  fetchFunction: (params: any) => Promise<{ data: T[], total: number, totalPages: number }>
  routeBuilder?: (item: T) => string
}

interface FilterOption {
  name: string
  value: string
  options: Array<{ value: string, label: string }>
  width?: string
}

export function useListManagement<T extends Record<string, any>>(options: ListOptions<T>) {
  const router = useRouter()
  
  // Data state
  const items = ref<T[]>([])
  const loading = ref(true)
  const error = ref<string | null>(null)
  
  // Pagination state
  const currentPage = ref(1)
  const pageSize = ref(options.defaultPageSize || 25)
  const pageSizeOptions = [10, 25, 50, 100]
  const totalItems = ref(0)
  const totalPages = ref(1)
  
  // Sorting state
  const sortField = ref(options.defaultSortField || 'id')
  const sortDirection = ref<'asc' | 'desc'>(options.defaultSortDirection || 'asc')
  
  // Search and filter state
  const searchQuery = ref('')
  const filters = ref<Record<string, string>>({})
  
  // Selection state
  const selectedItems = ref<string[]>([])
  const lastSelectedItemId = ref<string | null>(null)
  
  // Computed properties
  const itemIdField = options.itemIdField || 'id'
  
  // Build filter options for UI
  const buildFilterOptions = (filterConfigs: Record<string, { options: Array<{ value: string, label: string }>, width?: string, allLabel?: string }>) => {
    return Object.entries(filterConfigs).map(([name, config]) => ({
      name,
      value: filters.value[name] || 'all',
      options: [
        { value: 'all', label: config.allLabel || `All ${name.charAt(0).toUpperCase() + name.slice(1)}` },
        ...config.options
      ],
      width: config.width || 'w-[120px]'
    }))
  }
  
  // Fetch data function
  const fetchItems = async () => {
    loading.value = true
    error.value = null
    
    try {
      // Normalize filter values to lowercase for case-insensitive filtering
      const normalizedFilters = Object.fromEntries(
        Object.entries(filters.value)
          .filter(([_, value]) => value !== 'all')
          .map(([key, value]) => [key, value.toLowerCase()])
      )
      
      const params = {
        page: currentPage.value,
        pageSize: pageSize.value,
        sortField: sortField.value,
        sortDirection: sortDirection.value,
        search: searchQuery.value,
        ...normalizedFilters
      }
      
      const response = await options.fetchFunction(params)
      items.value = response.data
      totalItems.value = response.total
      totalPages.value = response.totalPages
    } catch (err) {
      console.error('Failed to fetch items:', err)
      error.value = 'Failed to load items. Please try again later.'
    } finally {
      loading.value = false
    }
  }
  
  // Selection logic
  const toggleSelection = (event: Event, itemId: string) => {
    event.stopPropagation()
    
    // Handle shift key for range selection
    if (
      event instanceof MouseEvent &&
      event.shiftKey &&
      lastSelectedItemId.value !== null
    ) {
      const currentIndex = items.value.findIndex(
        (item) => item[itemIdField].toString() === itemId
      )
      const lastIndex = items.value.findIndex(
        (item) => item[itemIdField].toString() === lastSelectedItemId.value
      )
      
      if (currentIndex !== -1 && lastIndex !== -1) {
        const startIndex = Math.min(currentIndex, lastIndex)
        const endIndex = Math.max(currentIndex, lastIndex)
        
        const itemsToSelect = items.value
          .slice(startIndex, endIndex + 1)
          .map((item) => item[itemIdField].toString())
        
        itemsToSelect.forEach((id) => {
          if (!selectedItems.value.includes(id)) {
            selectedItems.value.push(id)
          }
        })
      }
    }
    // Handle Ctrl/Cmd key for toggling individual items
    else if (event instanceof MouseEvent && (event.ctrlKey || event.metaKey)) {
      const index = selectedItems.value.indexOf(itemId)
      if (index === -1) {
        selectedItems.value.push(itemId)
      } else {
        selectedItems.value.splice(index, 1)
      }
      lastSelectedItemId.value = itemId
    }
    // Regular click - toggle selection
    else {
      const index = selectedItems.value.indexOf(itemId)
      if (index === -1) {
        selectedItems.value.push(itemId)
      } else {
        selectedItems.value.splice(index, 1)
      }
      lastSelectedItemId.value = itemId
    }
  }
  
  const toggleAllItems = (event: Event) => {
    event.stopPropagation()
    const checkbox = event.target as HTMLInputElement
    
    if (checkbox.checked) {
      selectedItems.value = items.value.map((item) => item[itemIdField].toString())
    } else {
      selectedItems.value = []
    }
    
    lastSelectedItemId.value = null
  }
  
  // Navigation
  const navigateToItem = (item: T) => {
    if (options.routeBuilder) {
      router.push(options.routeBuilder(item))
    }
  }
  
  // Filter and search handlers
  const handleFilterUpdate = (name: string, value: string) => {
    filters.value[name] = value
    currentPage.value = 1
  }
  
  const resetFilters = () => {
    searchQuery.value = ''
    filters.value = {}
    currentPage.value = 1
    fetchItems()
  }
  
  // Sort handlers
  const handleSortUpdate = (field: string, direction: 'asc' | 'desc') => {
    sortField.value = field
    sortDirection.value = direction
    currentPage.value = 1
  }
  
  // Pagination handlers
  const handlePageChange = (page: number) => {
    currentPage.value = page
  }
  
  const handlePageSizeChange = (size: number) => {
    pageSize.value = size
    currentPage.value = 1
  }
  
  // Date formatting utility
  const formatDate = (dateString: string) => {
    try {
      const date = new Date(dateString)
      if (isNaN(date.getTime())) return dateString
      
      return date.toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      })
    } catch (error) {
      console.error('Error formatting date:', error)
      return dateString
    }
  }
  
  // Watchers
  watch(
    [currentPage, pageSize, sortField, sortDirection, searchQuery],
    () => {
      fetchItems()
    }
  )
  
  // Watch filters separately to avoid excessive calls
  watch(
    () => filters.value,
    () => {
      fetchItems()
    },
    { deep: true }
  )
  
  // Initialize
  onMounted(() => {
    fetchItems()
  })
  
  return {
    // Data
    items,
    loading,
    error,
    
    // Pagination
    currentPage,
    pageSize,
    pageSizeOptions,
    totalItems,
    totalPages,
    
    // Sorting
    sortField,
    sortDirection,
    
    // Search and filters
    searchQuery,
    filters,
    
    // Selection
    selectedItems,
    
    // Methods
    fetchItems,
    toggleSelection,
    toggleAllItems,
    navigateToItem,
    handleFilterUpdate,
    resetFilters,
    handleSortUpdate,
    handlePageChange,
    handlePageSizeChange,
    buildFilterOptions,
    formatDate,
  }
} 