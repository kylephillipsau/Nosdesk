import { ref, computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'

type SearchableField<T> = (item: T) => string

export interface SearchOptions<T> {
  items: Ref<T[]>
  searchableFields: SearchableField<T>[]
}

export function useSearch<T>(options: SearchOptions<T>) {
  const { items, searchableFields } = options
  const searchQuery = ref('')

  const filteredItems: ComputedRef<T[]> = computed(() => {
    const query = searchQuery.value.toLowerCase().trim()
    if (!query) return items.value
    
    return items.value.filter(item => {
      return searchableFields.some(field => {
        const value = field(item)
        return value && value.toLowerCase().includes(query)
      })
    })
  })

  return {
    searchQuery,
    filteredItems
  }
} 