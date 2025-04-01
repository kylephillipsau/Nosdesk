import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

interface ExpandedState {
  [pageId: string]: boolean;
}

export const useDocumentationNavStore = defineStore('documentationNav', () => {
  // State for expanded pages
  const expandedPages = ref<ExpandedState>({})
  
  // State for sidebar visibility
  const isSidebarOpen = ref(false)
  
  // State for current page path
  const currentPagePath = ref<string[]>([])
  
  // State for page hierarchy
  const pageHierarchy = ref<Record<string, string[]>>({})
  
  // Add a flag for refreshing the navigation
  const needsRefresh = ref(false)
  
  // Initialize from localStorage if available
  if (localStorage.getItem('docNavExpandedPages')) {
    expandedPages.value = JSON.parse(localStorage.getItem('docNavExpandedPages')!)
  }
  
  if (localStorage.getItem('docNavSidebarOpen')) {
    isSidebarOpen.value = JSON.parse(localStorage.getItem('docNavSidebarOpen')!)
  } else {
    // Default to open on desktop, closed on mobile
    isSidebarOpen.value = window.innerWidth >= 768
  }
  
  // Save to localStorage when updated
  watch(expandedPages, (newState) => {
    localStorage.setItem('docNavExpandedPages', JSON.stringify(newState))
  }, { deep: true })
  
  watch(isSidebarOpen, (newState) => {
    localStorage.setItem('docNavSidebarOpen', JSON.stringify(newState))
  })
  
  // Set refresh flag to true
  const refreshPages = () => {
    needsRefresh.value = true
    
    // Reset it after a short delay to avoid multiple refreshes
    setTimeout(() => {
      needsRefresh.value = false
    }, 100)
  }
  
  // Check if refresh is needed
  const isRefreshNeeded = () => {
    return needsRefresh.value
  }
  
  // Toggle page expansion
  const togglePage = (pageId: string) => {
    expandedPages.value = {
      ...expandedPages.value,
      [pageId]: !expandedPages.value[pageId]
    }
  }
  
  // Expand a specific page
  const expandPage = (pageId: string) => {
    expandedPages.value[pageId] = true
  }
  
  // Collapse a specific page
  const collapsePage = (pageId: string) => {
    expandedPages.value[pageId] = false
  }
  
  // Expand all parents of a page
  const expandParents = (pageId: string, parentMap: Record<string, string | null>) => {
    let currentId = pageId;
    
    while (parentMap[currentId]) {
      const parentId = parentMap[currentId];
      if (parentId) {
        expandPage(parentId);
        currentId = parentId;
      } else {
        break;
      }
    }
  }
  
  // Set the current page path
  const setCurrentPagePath = (path: string[]) => {
    currentPagePath.value = path;
  }
  
  // Update page hierarchy
  const updatePageHierarchy = (hierarchy: Record<string, string[]>) => {
    pageHierarchy.value = hierarchy;
  }
  
  // Get children of a page
  const getChildrenOfPage = (pageId: string): string[] => {
    return pageHierarchy.value[pageId] || [];
  }
  
  // Toggle sidebar visibility
  const toggleSidebar = () => {
    isSidebarOpen.value = !isSidebarOpen.value
  }
  
  // Open sidebar
  const openSidebar = () => {
    isSidebarOpen.value = true
  }
  
  // Close sidebar
  const closeSidebar = () => {
    isSidebarOpen.value = false
  }
  
  // Set sidebar state based on screen size
  const updateSidebarForScreenSize = () => {
    const isMobile = window.innerWidth < 768
    isSidebarOpen.value = !isMobile
  }
  
  return {
    expandedPages,
    isSidebarOpen,
    currentPagePath,
    pageHierarchy,
    needsRefresh,
    refreshPages,
    isRefreshNeeded,
    togglePage,
    expandPage,
    collapsePage,
    expandParents,
    setCurrentPagePath,
    updatePageHierarchy,
    getChildrenOfPage,
    toggleSidebar,
    openSidebar,
    closeSidebar,
    updateSidebarForScreenSize
  }
})