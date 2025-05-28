// Global request manager for handling API request cancellation
import { ref, onUnmounted } from 'vue'
import userService from '@/services/userService'
import { cancelAllRequests as cancelDeviceRequests } from '@/services/deviceService'
import { cancelAllRequests as cancelTicketRequests } from '@/services/ticketService'
import { useImagePreloader } from '@/utils/imagePreloader'

// Global abort controller for page-level cancellation
const globalAbortController = ref<AbortController | null>(null)

export const useRequestManager = () => {
  const { clearQueue } = useImagePreloader()

  // Create a new page-level abort controller
  const createPageController = (): AbortController => {
    // Cancel any existing page controller
    if (globalAbortController.value) {
      globalAbortController.value.abort()
    }

    // Create new controller
    const controller = new AbortController()
    globalAbortController.value = controller

    return controller
  }

  // Cancel all requests for the current page
  const cancelPageRequests = (): void => {
    if (globalAbortController.value) {
      globalAbortController.value.abort()
      globalAbortController.value = null
    }

    // Cancel service-specific requests
    userService.cancelAllRequests()
    cancelDeviceRequests()
    cancelTicketRequests()

    // Clear image preloader queue
    clearQueue()
  }

  // Auto-cleanup on component unmount
  onUnmounted(() => {
    cancelPageRequests()
  })

  return {
    createPageController,
    cancelPageRequests,
    globalAbortController: globalAbortController.value
  }
}

// Utility for creating request-specific abort controllers
export const createRequestController = (requestKey: string): AbortController => {
  const controller = new AbortController()
  
  // Link to global controller if it exists
  if (globalAbortController.value) {
    globalAbortController.value.signal.addEventListener('abort', () => {
      controller.abort()
    })
  }

  return controller
}

// Hook for handling navigation-based cancellation
export const useNavigationCancellation = () => {
  const { cancelPageRequests } = useRequestManager()

  // Call this when navigating away from a page
  const handleNavigation = () => {
    cancelPageRequests()
  }

  return {
    handleNavigation
  }
} 