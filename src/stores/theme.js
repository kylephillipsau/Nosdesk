// src/stores/theme.js
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  // Use localStorage to persist theme preference, default to system preference
  const savedTheme = localStorage.getItem('theme')
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const currentTheme = ref(savedTheme || (prefersDark ? 'dark' : 'light'))

  const isDarkMode = computed(() => currentTheme.value === 'dark')

  function toggleTheme() {
    currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light'
    localStorage.setItem('theme', currentTheme.value)
    updateTheme()
  }

  function updateTheme() {
    document.documentElement.setAttribute('data-theme', currentTheme.value)
  }

  // Initialize theme on store creation
  updateTheme()

  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (!localStorage.getItem('theme')) {
      currentTheme.value = e.matches ? 'dark' : 'light'
      updateTheme()
    }
  })

  return {
    currentTheme,
    isDarkMode,
    toggleTheme,
  }
})