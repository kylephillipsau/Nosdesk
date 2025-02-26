import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

interface ExpandedState {
  [categoryId: string]: boolean;
}

export const useDocumentationNavStore = defineStore('documentationNav', () => {
  // State for expanded folders
  const expandedFolders = ref<ExpandedState>({})
  
  // State for sidebar visibility
  const isSidebarOpen = ref(false)
  
  // Initialize from localStorage if available
  if (localStorage.getItem('docNavExpandedFolders')) {
    expandedFolders.value = JSON.parse(localStorage.getItem('docNavExpandedFolders')!)
  }
  
  if (localStorage.getItem('docNavSidebarOpen')) {
    isSidebarOpen.value = JSON.parse(localStorage.getItem('docNavSidebarOpen')!)
  } else {
    // Default to open on desktop, closed on mobile
    isSidebarOpen.value = window.innerWidth >= 768
  }
  
  // Save to localStorage when updated
  watch(expandedFolders, (newState) => {
    localStorage.setItem('docNavExpandedFolders', JSON.stringify(newState))
  }, { deep: true })
  
  watch(isSidebarOpen, (newState) => {
    localStorage.setItem('docNavSidebarOpen', JSON.stringify(newState))
  })
  
  // Toggle folder expansion
  const toggleFolder = (categoryId: string) => {
    expandedFolders.value = {
      ...expandedFolders.value,
      [categoryId]: !expandedFolders.value[categoryId]
    }
  }
  
  // Expand a specific folder
  const expandFolder = (categoryId: string) => {
    expandedFolders.value[categoryId] = true
  }
  
  // Collapse a specific folder
  const collapseFolder = (categoryId: string) => {
    expandedFolders.value[categoryId] = false
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
    expandedFolders,
    isSidebarOpen,
    toggleFolder,
    expandFolder,
    collapseFolder,
    toggleSidebar,
    openSidebar,
    closeSidebar,
    updateSidebarForScreenSize
  }
}) 