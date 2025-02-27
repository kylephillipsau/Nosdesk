import { ref } from 'vue'
import type { Ref } from 'vue'

interface SelectionOptions<T> {
  getItemId: (item: T) => string | number
  items: Ref<T[]>
}

export function useSelection<T>(options: SelectionOptions<T>) {
  const { getItemId, items } = options
  const selectedIds = ref<(string | number)[]>([])
  const lastSelectedId = ref<string | number | null>(null)

  const isSelected = (item: T) => {
    const id = getItemId(item)
    return selectedIds.value.includes(id)
  }

  const toggleSelection = (event: Event, item: T) => {
    event.stopPropagation()
    const id = getItemId(item)
    
    // Handle shift key for multiple selection
    if (event instanceof MouseEvent && event.shiftKey && lastSelectedId.value !== null) {
      const currentIndex = items.value.findIndex(i => getItemId(i) === id)
      const lastIndex = items.value.findIndex(i => getItemId(i) === lastSelectedId.value)
      
      if (currentIndex !== -1 && lastIndex !== -1) {
        const startIndex = Math.min(currentIndex, lastIndex)
        const endIndex = Math.max(currentIndex, lastIndex)
        
        const idsToSelect = items.value
          .slice(startIndex, endIndex + 1)
          .map(i => getItemId(i))
        
        // Add all items in range to selection if they're not already selected
        idsToSelect.forEach(itemId => {
          if (!selectedIds.value.includes(itemId)) {
            selectedIds.value.push(itemId)
          }
        })
      }
    } else {
      // Regular single selection toggle
      const index = selectedIds.value.indexOf(id)
      if (index === -1) {
        selectedIds.value.push(id)
      } else {
        selectedIds.value.splice(index, 1)
      }
      
      // Update last selected item
      lastSelectedId.value = id
    }
  }

  const toggleAll = (event: Event) => {
    event.stopPropagation()
    const checkbox = event.target as HTMLInputElement
    
    if (checkbox.checked) {
      selectedIds.value = items.value.map(item => getItemId(item))
    } else {
      selectedIds.value = []
    }
    
    lastSelectedId.value = null
  }

  const clearSelection = () => {
    selectedIds.value = []
    lastSelectedId.value = null
  }

  return {
    selectedIds,
    isSelected,
    toggleSelection,
    toggleAll,
    clearSelection
  }
} 