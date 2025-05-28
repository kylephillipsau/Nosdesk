// Image preloader utility for critical images and performance optimization
export interface PreloadOptions {
  priority?: 'high' | 'medium' | 'low'
  timeout?: number
  retries?: number
  crossOrigin?: 'anonymous' | 'use-credentials' | ''
  signal?: AbortSignal
}

interface PreloadTask {
  url: string
  options: PreloadOptions
  promise: Promise<void>
  retryCount: number
  abortController: AbortController
}

class ImagePreloader {
  private queue: PreloadTask[] = []
  private processing = false
  private maxConcurrent = 3
  private activeRequests = 0
  private globalAbortController: AbortController | null = null

  // Preload a single image with cancellation support
  async preloadImage(url: string, options: PreloadOptions = {}): Promise<void> {
    const {
      priority = 'medium',
      timeout = 10000,
      retries = 2,
      crossOrigin = 'anonymous',
      signal
    } = options

    return new Promise((resolve, reject) => {
      const img = new Image()
      let timeoutId: number | null = null

      const cleanup = () => {
        if (timeoutId) {
          clearTimeout(timeoutId)
          timeoutId = null
        }
      }

      const handleLoad = () => {
        cleanup()
        resolve()
      }

      const handleError = () => {
        cleanup()
        reject(new Error(`Failed to load image: ${url}`))
      }

      const handleAbort = () => {
        cleanup()
        reject(new Error(`Image load cancelled: ${url}`))
      }

      // Handle external abort signal
      if (signal) {
        if (signal.aborted) {
          reject(new Error(`Image load cancelled: ${url}`))
          return
        }
        signal.addEventListener('abort', handleAbort)
      }

      // Set up timeout
      if (timeout > 0) {
        timeoutId = window.setTimeout(() => {
          cleanup()
          if (signal) signal.removeEventListener('abort', handleAbort)
          reject(new Error(`Image load timeout: ${url}`))
        }, timeout)
      }

      // Configure image
      img.onload = () => {
        if (signal) signal.removeEventListener('abort', handleAbort)
        handleLoad()
      }
      img.onerror = () => {
        if (signal) signal.removeEventListener('abort', handleAbort)
        handleError()
      }
      
      if (crossOrigin) {
        img.crossOrigin = crossOrigin
      }

      // Start loading
      img.src = url
    })
  }

  // Add image to preload queue with cancellation support
  queuePreload(url: string, options: PreloadOptions = {}): Promise<void> {
    const existingTask = this.queue.find(task => task.url === url && !task.abortController.signal.aborted)
    if (existingTask) {
      return existingTask.promise
    }

    // Create abort controller for this task
    const abortController = new AbortController()
    
    // Link to global abort controller if it exists
    if (this.globalAbortController) {
      this.globalAbortController.signal.addEventListener('abort', () => {
        abortController.abort()
      })
    }

    const task: PreloadTask = {
      url,
      options: { ...options, signal: abortController.signal },
      retryCount: 0,
      abortController,
      promise: this.createPreloadPromise(url, { ...options, signal: abortController.signal })
    }

    // Insert based on priority
    const priorityOrder = { high: 0, medium: 1, low: 2 }
    const taskPriority = priorityOrder[options.priority || 'medium']
    
    let insertIndex = this.queue.length
    for (let i = 0; i < this.queue.length; i++) {
      const queuePriority = priorityOrder[this.queue[i].options.priority || 'medium']
      if (taskPriority < queuePriority) {
        insertIndex = i
        break
      }
    }

    this.queue.splice(insertIndex, 0, task)
    this.processQueue()

    return task.promise
  }

  private async createPreloadPromise(url: string, options: PreloadOptions): Promise<void> {
    const maxRetries = options.retries || 2
    let lastError: Error | null = null

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        // Check if cancelled before each attempt
        if (options.signal?.aborted) {
          throw new Error(`Image load cancelled: ${url}`)
        }
        
        await this.preloadImage(url, options)
        return
      } catch (error) {
        lastError = error as Error
        
        // Don't retry if cancelled
        if (lastError.message.includes('cancelled')) {
          throw lastError
        }
        
        if (attempt < maxRetries) {
          // Exponential backoff
          const delay = Math.min(1000 * Math.pow(2, attempt), 5000)
          await new Promise(resolve => setTimeout(resolve, delay))
        }
      }
    }

    throw lastError
  }

  private async processQueue(): Promise<void> {
    if (this.processing || this.activeRequests >= this.maxConcurrent) {
      return
    }

    this.processing = true

    while (this.queue.length > 0 && this.activeRequests < this.maxConcurrent) {
      const task = this.queue.shift()
      if (!task || task.abortController.signal.aborted) continue

      this.activeRequests++
      
      task.promise
        .catch(error => {
          // Silently handle cancelled requests
          if (!error.message.includes('cancelled')) {
            console.warn('Image preload failed:', error)
          }
        })
        .finally(() => {
          this.activeRequests--
          this.processQueue()
        })
    }

    this.processing = false
  }

  // Preload multiple images with cancellation support
  async preloadImages(urls: string[], options: PreloadOptions = {}): Promise<void[]> {
    const promises = urls.map(url => this.queuePreload(url, options))
    return Promise.allSettled(promises).then(results => 
      results.map(result => {
        if (result.status === 'rejected' && !result.reason.message?.includes('cancelled')) {
          console.warn('Image preload failed:', result.reason)
        }
      })
    )
  }

  // Preload critical images immediately
  async preloadCritical(urls: string[]): Promise<void> {
    await this.preloadImages(urls, { priority: 'high', timeout: 5000 })
  }

  // Clear the queue and cancel all pending requests
  clearQueue(): void {
    // Cancel all individual tasks
    this.queue.forEach(task => task.abortController.abort())
    this.queue = []
    
    // Cancel global controller if it exists
    if (this.globalAbortController) {
      this.globalAbortController.abort()
      this.globalAbortController = null
    }
  }

  // Set global abort controller for batch cancellation
  setGlobalAbortController(controller: AbortController): void {
    this.globalAbortController = controller
  }

  // Get queue status
  getQueueStatus(): { pending: number; active: number } {
    return {
      pending: this.queue.filter(task => !task.abortController.signal.aborted).length,
      active: this.activeRequests
    }
  }
}

// Create singleton instance
export const imagePreloader = new ImagePreloader()

// Utility functions for common use cases
export const preloadUserAvatars = async (userUuids: string[]): Promise<void> => {
  // This would need to be adapted based on your avatar URL structure
  const avatarUrls = userUuids
    .map(uuid => `/uploads/users/avatars/${uuid}`)
    .filter(Boolean)

  await imagePreloader.preloadImages(avatarUrls, { priority: 'medium' })
}

export const preloadCriticalImages = async (urls: string[]): Promise<void> => {
  await imagePreloader.preloadCritical(urls)
}

// Hook for Vue components
export const useImagePreloader = () => {
  return {
    preloadImage: imagePreloader.queuePreload.bind(imagePreloader),
    preloadImages: imagePreloader.preloadImages.bind(imagePreloader),
    preloadCritical: imagePreloader.preloadCritical.bind(imagePreloader),
    clearQueue: imagePreloader.clearQueue.bind(imagePreloader),
    getQueueStatus: imagePreloader.getQueueStatus.bind(imagePreloader),
    setGlobalAbortController: imagePreloader.setGlobalAbortController.bind(imagePreloader)
  }
} 