import { ref, onMounted } from 'vue'
import type { Ref } from 'vue'

export interface DataLoaderOptions<T> {
  fetchData: () => Promise<T[]>
  initialData?: T[]
  loadOnMount?: boolean
}

export function useDataLoader<T>(options: DataLoaderOptions<T>) {
  const { fetchData, initialData = [], loadOnMount = true } = options
  
  const data = ref<T[]>(initialData) as Ref<T[]>
  const isLoading = ref(false)
  const error = ref<Error | null>(null)

  const load = async () => {
    isLoading.value = true
    error.value = null
    
    try {
      data.value = await fetchData()
    } catch (err) {
      error.value = err instanceof Error ? err : new Error('Unknown error occurred')
      console.error('Error loading data:', err)
    } finally {
      isLoading.value = false
    }
  }

  if (loadOnMount) {
    onMounted(load)
  }

  return {
    data,
    isLoading,
    error,
    load
  }
} 