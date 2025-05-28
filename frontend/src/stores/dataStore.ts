import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import userService from '@/services/userService'
import type { User, PaginatedResponse, PaginationParams } from '@/services/userService'

// Cache configuration
const CACHE_TTL = 5 * 60 * 1000 // 5 minutes
const BACKGROUND_REFRESH_THRESHOLD = 2 * 60 * 1000 // 2 minutes

interface CacheEntry<T> {
  data: T
  timestamp: number
  loading: boolean
  error: string | null
}

interface PaginatedCacheEntry {
  data: User[]
  total: number
  totalPages: number
  timestamp: number
  loading: boolean
  error: string | null
  params: PaginationParams
}

// Request deduplication
const activeRequests = new Map<string, Promise<any>>()

export const useDataStore = defineStore('data', () => {
  // Users cache
  const usersCache = ref(new Map<string, PaginatedCacheEntry>())
  const individualUsersCache = ref(new Map<string, CacheEntry<User>>())
  
  // Global loading states
  const globalLoading = ref(false)
  
  // Helper function to create cache key
  const createCacheKey = (params: PaginationParams): string => {
    return JSON.stringify({
      page: params.page,
      pageSize: params.pageSize,
      sortField: params.sortField || 'id',
      sortDirection: params.sortDirection || 'asc',
      search: params.search || '',
      role: params.role || ''
    })
  }
  
  // Check if cache entry is valid
  const isCacheValid = (timestamp: number): boolean => {
    return Date.now() - timestamp < CACHE_TTL
  }
  
  // Check if cache needs background refresh
  const needsBackgroundRefresh = (timestamp: number): boolean => {
    return Date.now() - timestamp > BACKGROUND_REFRESH_THRESHOLD
  }
  
  // Get paginated users with smart caching
  const getPaginatedUsers = async (params: PaginationParams, forceRefresh = false): Promise<PaginatedResponse<User>> => {
    const cacheKey = createCacheKey(params)
    const cached = usersCache.value.get(cacheKey)
    
    // Return cached data if valid and not forcing refresh
    if (!forceRefresh && cached && isCacheValid(cached.timestamp) && !cached.loading) {
      // Check if we need background refresh
      if (needsBackgroundRefresh(cached.timestamp)) {
        // Trigger background refresh without waiting
        refreshPaginatedUsersInBackground(params, cacheKey)
      }
      
      return {
        data: cached.data,
        total: cached.total,
        page: params.page,
        pageSize: params.pageSize,
        totalPages: cached.totalPages
      }
    }
    
    // Check for ongoing request
    const requestKey = `paginated-users-${cacheKey}`
    if (activeRequests.has(requestKey)) {
      return activeRequests.get(requestKey)
    }
    
    // Create new request
    const requestPromise = fetchPaginatedUsersFromAPI(params, cacheKey)
    activeRequests.set(requestKey, requestPromise)
    
    try {
      const result = await requestPromise
      return result
    } finally {
      activeRequests.delete(requestKey)
    }
  }
  
  // Fetch from API and update cache
  const fetchPaginatedUsersFromAPI = async (params: PaginationParams, cacheKey: string): Promise<PaginatedResponse<User>> => {
    // Update loading state
    const existingCache = usersCache.value.get(cacheKey)
    usersCache.value.set(cacheKey, {
      data: existingCache?.data || [],
      total: existingCache?.total || 0,
      totalPages: existingCache?.totalPages || 0,
      timestamp: existingCache?.timestamp || 0,
      loading: true,
      error: null,
      params
    })
    
    try {
      const response = await userService.getPaginatedUsers(params, `cached-${cacheKey}`)
      
      // Update cache with new data
      usersCache.value.set(cacheKey, {
        data: response.data,
        total: response.total,
        totalPages: response.totalPages,
        timestamp: Date.now(),
        loading: false,
        error: null,
        params
      })
      
      // Also update individual user cache
      response.data.forEach(user => {
        individualUsersCache.value.set(user.uuid, {
          data: user,
          timestamp: Date.now(),
          loading: false,
          error: null
        })
      })
      
      return response
    } catch (error) {
      // Update cache with error
      usersCache.value.set(cacheKey, {
        data: existingCache?.data || [],
        total: existingCache?.total || 0,
        totalPages: existingCache?.totalPages || 0,
        timestamp: existingCache?.timestamp || 0,
        loading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        params
      })
      
      throw error
    }
  }
  
  // Background refresh without blocking UI
  const refreshPaginatedUsersInBackground = async (params: PaginationParams, cacheKey: string) => {
    try {
      console.log('Background refreshing users cache for:', cacheKey)
      await fetchPaginatedUsersFromAPI(params, cacheKey)
    } catch (error) {
      console.warn('Background refresh failed:', error)
    }
  }
  
  // Get individual user by UUID
  const getUserByUuid = async (uuid: string, forceRefresh = false): Promise<User | null> => {
    const cached = individualUsersCache.value.get(uuid)
    
    // Return cached data if valid
    if (!forceRefresh && cached && isCacheValid(cached.timestamp) && !cached.loading) {
      if (needsBackgroundRefresh(cached.timestamp)) {
        // Trigger background refresh
        refreshUserInBackground(uuid)
      }
      return cached.data
    }
    
    // Check for ongoing request
    const requestKey = `user-${uuid}`
    if (activeRequests.has(requestKey)) {
      return activeRequests.get(requestKey)
    }
    
    // Create new request
    const requestPromise = fetchUserFromAPI(uuid)
    activeRequests.set(requestKey, requestPromise)
    
    try {
      const result = await requestPromise
      return result
    } finally {
      activeRequests.delete(requestKey)
    }
  }
  
  // Fetch individual user from API
  const fetchUserFromAPI = async (uuid: string): Promise<User | null> => {
    // Update loading state
    individualUsersCache.value.set(uuid, {
      data: individualUsersCache.value.get(uuid)?.data || {} as User,
      timestamp: individualUsersCache.value.get(uuid)?.timestamp || 0,
      loading: true,
      error: null
    })
    
    try {
      const user = await userService.getUserByUuid(uuid)
      
      if (user) {
        // Update cache
        individualUsersCache.value.set(uuid, {
          data: user,
          timestamp: Date.now(),
          loading: false,
          error: null
        })
      }
      
      return user
    } catch (error) {
      // Update cache with error
      individualUsersCache.value.set(uuid, {
        data: individualUsersCache.value.get(uuid)?.data || {} as User,
        timestamp: individualUsersCache.value.get(uuid)?.timestamp || 0,
        loading: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      })
      
      return null
    }
  }
  
  // Background refresh for individual user
  const refreshUserInBackground = async (uuid: string) => {
    try {
      console.log('Background refreshing user cache for:', uuid)
      await fetchUserFromAPI(uuid)
    } catch (error) {
      console.warn('Background user refresh failed:', error)
    }
  }
  
  // Get user name from cache (for quick lookups)
  const getUserName = (uuid: string): string | null => {
    const cached = individualUsersCache.value.get(uuid)
    return cached?.data?.name || null
  }
  
  // Get user avatar from cache
  const getUserAvatar = (uuid: string, preferThumb = true): string | null => {
    const cached = individualUsersCache.value.get(uuid)
    if (!cached?.data) return null
    
    if (preferThumb && cached.data.avatar_thumb) {
      return cached.data.avatar_thumb
    }
    return cached.data.avatar_url || null
  }
  
  // Invalidate cache for specific user
  const invalidateUser = (uuid: string) => {
    individualUsersCache.value.delete(uuid)
    
    // Also invalidate paginated caches that might contain this user
    for (const [key, cache] of usersCache.value.entries()) {
      if (cache.data.some(user => user.uuid === uuid)) {
        usersCache.value.delete(key)
      }
    }
  }
  
  // Invalidate all user caches
  const invalidateAllUsers = () => {
    usersCache.value.clear()
    individualUsersCache.value.clear()
  }
  
  // Update user in cache (for optimistic updates)
  const updateUserInCache = (uuid: string, updates: Partial<User>) => {
    const cached = individualUsersCache.value.get(uuid)
    if (cached) {
      cached.data = { ...cached.data, ...updates }
      cached.timestamp = Date.now()
    }
    
    // Update in paginated caches too
    for (const cache of usersCache.value.values()) {
      const userIndex = cache.data.findIndex(user => user.uuid === uuid)
      if (userIndex !== -1) {
        cache.data[userIndex] = { ...cache.data[userIndex], ...updates }
      }
    }
  }
  
  // Add new user to cache
  const addUserToCache = (user: User) => {
    individualUsersCache.value.set(user.uuid, {
      data: user,
      timestamp: Date.now(),
      loading: false,
      error: null
    })
    
    // Invalidate paginated caches since they need to be refreshed
    usersCache.value.clear()
  }
  
  // Remove user from cache
  const removeUserFromCache = (uuid: string) => {
    individualUsersCache.value.delete(uuid)
    
    // Remove from paginated caches
    for (const cache of usersCache.value.values()) {
      cache.data = cache.data.filter(user => user.uuid !== uuid)
    }
  }
  
  // Get cache statistics
  const getCacheStats = computed(() => ({
    paginatedCaches: usersCache.value.size,
    individualUsers: individualUsersCache.value.size,
    activeRequests: activeRequests.size
  }))
  
  // Cleanup expired cache entries
  const cleanupExpiredCache = () => {
    const now = Date.now()
    
    // Clean individual users cache
    for (const [uuid, cache] of individualUsersCache.value.entries()) {
      if (now - cache.timestamp > CACHE_TTL * 2) { // Keep cache longer than TTL for background refresh
        individualUsersCache.value.delete(uuid)
      }
    }
    
    // Clean paginated cache
    for (const [key, cache] of usersCache.value.entries()) {
      if (now - cache.timestamp > CACHE_TTL * 2) {
        usersCache.value.delete(key)
      }
    }
  }
  
  // Auto cleanup every 10 minutes
  setInterval(cleanupExpiredCache, 10 * 60 * 1000)
  
  return {
    // Main API methods
    getPaginatedUsers,
    getUserByUuid,
    getUserName,
    getUserAvatar,
    
    // Cache management
    invalidateUser,
    invalidateAllUsers,
    updateUserInCache,
    addUserToCache,
    removeUserFromCache,
    
    // State
    globalLoading,
    getCacheStats,
    
    // Utilities
    cleanupExpiredCache
  }
}) 