import { ref, computed, watch } from 'vue';
import { useRoute } from 'vue-router';

export function usePageTitle() {
  const route = useRoute();
  const customTitle = ref<string | null>(null);

  const pageTitle = computed(() => {
    // Custom title takes precedence if set
    if (customTitle.value) {
      return customTitle.value;
    }

    // Get title from route meta
    const title = route.meta?.title as string;
    
    // Update document title if needed
    if (title && document.title !== `${title} | nosDesk`) {
      document.title = `${title} | nosDesk`;
    }
    
    return title || 'nosDesk';
  });

  // Watch for route changes
  watch(
    () => route.name,
    () => {
      // Reset custom title when route changes
      customTitle.value = null;
    }
  );

  const setCustomTitle = (title: string | null) => {
    customTitle.value = title;
    if (title) {
      document.title = `${title} | nosDesk`;
    }
  };

  return {
    pageTitle,
    setCustomTitle
  };
}
