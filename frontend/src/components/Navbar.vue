<script setup lang="ts">
import { RouterLink, useRoute, useRouter } from "vue-router";
import DocumentationNav from "@/components/documentationComponents/DocumentationNav.vue";
import RecentTickets from "@/components/RecentTickets.vue";
import {
  ref,
  watch,
  computed,
  onMounted,
  onBeforeUnmount,
  nextTick,
} from "vue";
import { useResizableSidebar } from "@/composables/useResizableSidebar";

const route = useRoute();
const router = useRouter();
const searchTerm = ref("");

// State for collapsed/expanded navbar
const isCollapsed = ref(false);
const isSmallScreen = ref(false);

// State for section collapsing
const isDocsCollapsed = ref(false);
const isTicketsCollapsed = ref(false);

// Refs for DOM elements - These will be passed to the composable
const navbarRef = ref<HTMLElement | null>(null);
const ticketsSectionRef = ref<HTMLElement | null>(null);
const docsSectionRef = ref<HTMLElement | null>(null);
const resizerRef = ref<HTMLElement | null>(null);

// Define locally for check in onMounted, or expose from composable if preferred
const MIN_SECTION_HEIGHT = 60;

// Use the composable for resizing logic
const {
  ticketsHeight, // The reactive height value from the composable
  isResizing, // The reactive resizing status from the composable
  startResize, // The function to start resizing, attach to resizer handle
  equalizeHeights, // Utility function to equalize heights
} = useResizableSidebar(
  navbarRef,
  ticketsSectionRef,
  docsSectionRef,
  resizerRef
);

// Provide/inject for sharing with App.vue
const emit = defineEmits(["update:collapsed"]);

// Toggle navbar collapsed state
const toggleNav = () => {
  isCollapsed.value = !isCollapsed.value;
  emit("update:collapsed", isCollapsed.value);
  // Store preference in localStorage
  localStorage.setItem("navbarCollapsed", isCollapsed.value.toString());
};

// Toggle documentation section
const toggleDocs = () => {
  isDocsCollapsed.value = !isDocsCollapsed.value;
  localStorage.setItem("docsCollapsed", isDocsCollapsed.value.toString());
};

// Toggle tickets section
const toggleTickets = () => {
  isTicketsCollapsed.value = !isTicketsCollapsed.value;
  localStorage.setItem("ticketsCollapsed", isTicketsCollapsed.value.toString());
};

// Check screen size and set navbar state accordingly
const checkScreenSize = () => {
  const previouslySmall = isSmallScreen.value;
  isSmallScreen.value = window.innerWidth < 1024; // lg breakpoint

  // Get stored preference (if any)
  const storedPref = localStorage.getItem("navbarCollapsed");

  // If screen size changed from large to small
  if (isSmallScreen.value && !previouslySmall) {
    isCollapsed.value = true; // Always collapse on small screens
  }
  // If screen size changed from small to large
  else if (!isSmallScreen.value && previouslySmall) {
    // On larger screens, use the stored preference or default to expanded
    isCollapsed.value = storedPref === "true";
  }
  // On initial load for large screens
  else if (!isSmallScreen.value && !previouslySmall && !storedPref) {
    // Default to expanded on large screens if no preference is stored
    isCollapsed.value = false;
  }

  // Emit the current state
  emit("update:collapsed", isCollapsed.value);
};

// Initialize on mount
onMounted(() => {
  // Initial state - check if we have a stored preference
  const storedPref = localStorage.getItem("navbarCollapsed");
  const storedDocsCollapsed = localStorage.getItem("docsCollapsed");
  const storedTicketsCollapsed = localStorage.getItem("ticketsCollapsed");
  // ticketsHeight is now handled by the composable

  // Check screen size first
  isSmallScreen.value = window.innerWidth < 1024;

  // Set initial state based on screen size and preference
  if (isSmallScreen.value) {
    // On small screens, always start collapsed
    isCollapsed.value = true;
  } else {
    // On larger screens, use stored preference or default to expanded
    isCollapsed.value = storedPref === "true";
  }

  // Set sections collapsed state from localStorage or default to false
  isDocsCollapsed.value = storedDocsCollapsed === "true";
  isTicketsCollapsed.value = storedTicketsCollapsed === "true";

  // Emit initial state
  emit("update:collapsed", isCollapsed.value);

  // Add resize listener for screen size changes
  window.addEventListener("resize", checkScreenSize);

  // Set initial sizes after mount
  nextTick(() => {
    if (!ticketsHeight.value || ticketsHeight.value < MIN_SECTION_HEIGHT) {
      // Use local constant
      if (
        !isCollapsed.value &&
        !isTicketsCollapsed.value &&
        !isDocsCollapsed.value
      ) {
        equalizeHeights();
      }
    }
  });
});

// Clean up on unmount
onBeforeUnmount(() => {
  window.removeEventListener("resize", checkScreenSize);
  // Global listeners for resizing are handled by the composable's onBeforeUnmount
});

// Computed property to check if we're on a documentation page
const isDocumentationPage = computed(() => {
  return route.path.startsWith("/documentation");
});

// Handle documentation search
const handleDocSearch = (query: string) => {
  if (isDocumentationPage.value) {
    // If already on documentation page, update the search query
    searchTerm.value = query;
  } else {
    // If not on documentation page, navigate to documentation with search query
    router.push({
      path: "/documentation",
      query: { search: query },
    });
  }
};

// Navigation links data
const navLinks = [
  {
    to: "/",
    icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6",
    text: "Dashboard",
    exact: true,
    color: "#FDBD10",
  },
  {
    to: "/tickets",
    icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4",
    text: "Tickets",
    color: "#2C80FF",
  },
  {
    to: "/projects",
    icon: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z",
    text: "Projects",
    color: "#00C951",
  },
  {
    to: "/users",
    icon: "M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z",
    text: "Users",
    color: "#FF66B3",
  },
  {
    to: "/devices",
    icon: "M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z",
    text: "Devices",
    color: "#99A1AF",
  },
  {
    to: "/documentation",
    icon: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253",
    text: "Documentation",
    color: "#8B5CF6",
  },
];

// Helper function to check if a route is active
const isRouteActive = (path: string, exact = false) => {
  if (exact) {
    return route.path === path;
  }
  return route.path.startsWith(path);
};
</script>

<template>
  <!-- Desktop Sidebar - Hidden on small screens -->
  <nav
    ref="navbarRef"
    class="h-screen bg-slate-800 border-r border-black flex flex-col flex-shrink-0 print:hidden transition-all duration-300 ease-in-out overflow-hidden lg:fixed lg:left-0 lg:top-0 lg:z-20 lg:flex"
    :class="[
      isCollapsed ? 'lg:w-16' : 'lg:w-64',
      isSmallScreen ? 'hidden' : '',
    ]"
  >
    <div class="flex flex-col p-2 px-2 flex-shrink-0 gap-1">
      <!-- Show logo only when navbar is expanded -->
      <RouterLink
        v-if="!isCollapsed"
        to="/"
        class="flex items-center mb-5 hover:opacity-80 transition-opacity text-[#FDBD10] select-none"
      >
        <img alt="Nosdesk Logo" class="px-3 py-1" src="@/assets/logo.svg" />
      </RouterLink>

      <!-- When collapsed, add a smaller top spacing -->
      <RouterLink
        v-else
        to="/"
        class="flex items-center mb-5 hover:opacity-80 transition-opacity text-[#FDBD10] select-none"
      >
        <img alt="Nosdesk Logo" class="px-3 py-1" src="@/assets/logo.svg" />
      </RouterLink>

      <div class="flex flex-col gap-1 mb-2">
        <RouterLink
          v-for="link in navLinks"
          :key="link.to"
          :to="link.to"
          class="rounded-md transition-colors duration-200 text-white flex items-center gap-3 relative overflow-hidden"
          :class="[
            isRouteActive(link.to, link.exact)
              ? 'bg-slate-700/80 text-white font-medium'
              : 'text-slate-300 hover:bg-slate-700/50 hover:text-white',
            isCollapsed ? 'px-2 py-1.5 justify-center' : 'px-3 py-2',
          ]"
          :title="isCollapsed ? link.text : ''"
        >
          <!-- Active indicator bar -->
          <div
            v-if="isRouteActive(link.to, link.exact)"
            class="absolute left-0 top-0 bottom-0 w-1"
            :style="{ backgroundColor: link.color }"
          ></div>

          <svg
            class="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              :d="link.icon"
            />
          </svg>
          <span v-if="!isCollapsed" class="text-sm whitespace-nowrap">{{
            link.text
          }}</span>
        </RouterLink>
      </div>
    </div>

    <!-- Separator -->
    <div class="border-t border-slate-700/50 my-1"></div>

    <!-- Only show sections when navbar is expanded -->
    <div
      class="flex-1 min-h-0 flex flex-col overflow-hidden"
      v-if="!isCollapsed"
    >
      <!-- Recent Tickets section with collapsible header -->
      <div
        ref="ticketsSectionRef"
        class="tickets-section flex-shrink-0 overflow-hidden flex flex-col transition-all duration-200"
        :style="{
          maxHeight: isTicketsCollapsed ? '32px' : `${ticketsHeight}px`,
        }"
        :class="[isTicketsCollapsed ? 'opacity-90 hover:opacity-100' : '']"
      >
        <div
          class="flex items-center justify-between py-1.5 px-3 cursor-pointer transition-colors duration-200 group"
          :class="
            isTicketsCollapsed ? 'hover:bg-slate-700/30' : 'bg-slate-700/20'
          "
          @click="toggleTickets"
        >
          <h3
            class="text-xs font-medium text-slate-300 uppercase tracking-wider flex items-center gap-1"
          >
            <span class="w-2 h-2 bg-[#2C80FF] rounded-full"></span>
            Recent Tickets
          </h3>
          <button
            class="text-slate-400 group-hover:text-white transition-colors duration-200 bg-slate-700/30 rounded p-0.5"
            :title="isTicketsCollapsed ? 'Expand section' : 'Collapse section'"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-3 w-3"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                :d="isTicketsCollapsed ? 'M19 9l-7 7-7-7' : 'M5 15l7-7 7 7'"
              />
            </svg>
          </button>
        </div>

        <div
          class="overflow-y-auto bg-slate-800/60"
          :class="isTicketsCollapsed ? 'opacity-0 h-0' : 'opacity-100 flex-1'"
        >
          <RecentTickets v-if="!isTicketsCollapsed" />
        </div>
      </div>

      <!-- Resizer between sections -->
      <div
        ref="resizerRef"
        class="resizer-handle group relative mx-1 flex items-center justify-center select-none"
        @mousedown="startResize"
        @touchstart.prevent="startResize"
        :class="{ active: isResizing }"
      >
        <!-- Equalize button removed -->
        <!-- Drag indicator lines removed -->
      </div>

      <!-- Documentation section with collapsible header -->
      <div
        ref="docsSectionRef"
        class="docs-section flex-1 min-h-0 overflow-hidden flex flex-col transition-all duration-200 -mt-px"
        :class="[isDocsCollapsed ? 'opacity-90 hover:opacity-100' : '']"
      >
        <div
          class="flex items-center justify-between py-1.5 px-3 cursor-pointer transition-colors duration-200 group"
          :class="isDocsCollapsed ? 'hover:bg-slate-700/30' : 'bg-slate-700/20'"
          @click="toggleDocs"
        >
          <h3
            class="text-xs font-medium text-slate-300 uppercase tracking-wider flex items-center gap-1"
          >
            <span class="w-2 h-2 bg-[#8B5CF6] rounded-full"></span>
            Documentation
          </h3>
          <button
            class="text-slate-400 group-hover:text-white transition-colors duration-200 bg-slate-700/30 rounded p-0.5"
            :title="isDocsCollapsed ? 'Expand section' : 'Collapse section'"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-3 w-3"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                :d="isDocsCollapsed ? 'M19 9l-7 7-7-7' : 'M5 15l7-7 7 7'"
              />
            </svg>
          </button>
        </div>

        <div
          class="overflow-y-auto flex-1 bg-slate-800/60"
          :class="isDocsCollapsed ? 'opacity-0 h-0' : 'opacity-100'"
        >
          <DocumentationNav v-if="!isDocsCollapsed" @search="handleDocSearch" />
        </div>
      </div>
    </div>

    <!-- Toggle button at the bottom of sidebar -->
    <div class="mt-auto border-t border-slate-700 flex flex-col justify-end">
      <button
        @click="toggleNav"
        class="w-full p-2 text-slate-300 hover:text-white hover:bg-slate-700/70 rounded-md transition-colors group flex"
        :class="isCollapsed ? 'justify-center' : 'justify-between'"
        aria-label="Toggle sidebar"
      >
        <!-- Left side with icon and text -->
        <div class="flex items-center">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 group-hover:text-[#FDBD10] transition-colors"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              v-if="isCollapsed"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 5l7 7-7 7M5 5l7 7-7 7"
            />
            <path
              v-else
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
            />
          </svg>
          <span v-if="!isCollapsed" class="ml-2 text-xs">Collapse Sidebar</span>
        </div>

        <!-- Keyboard shortcut (only shown when expanded) -->
        <kbd
          v-if="!isCollapsed"
          class="hidden md:inline-flex text-[10px] px-1.5 py-0.5 bg-slate-700 rounded text-slate-400 items-center"
        >
          ⌘ K
        </kbd>
      </button>
    </div>
  </nav>

  <!-- Mobile Bottom Navigation -->
  <nav
    class="fixed bottom-0 left-0 right-0 bg-slate-800 border-t border-slate-700 z-20 lg:hidden print:hidden"
    v-if="isSmallScreen"
  >
    <div class="flex justify-around items-center h-14">
      <RouterLink
        v-for="link in navLinks"
        :key="link.to"
        :to="link.to"
        class="flex flex-col items-center justify-center px-2 py-1 rounded-lg transition-colors duration-200 hover:bg-slate-700/50 flex-1 relative"
        :class="isRouteActive(link.to, link.exact) ? '' : 'text-slate-300'"
        :style="isRouteActive(link.to, link.exact) ? { color: link.color } : {}"
      >
        <svg
          class="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            :d="link.icon"
          />
        </svg>
        <span class="text-xs mt-0.5 truncate w-full text-center">{{
          link.text
        }}</span>
      </RouterLink>
    </div>
  </nav>

  <!-- Show semi-transparent overlay on small screens when nav is expanded -->
  <div
    v-if="isSmallScreen && !isCollapsed"
    class="fixed inset-0 bg-black bg-opacity-50 z-10 lg:hidden"
    @click="toggleNav"
  ></div>
</template>

<style scoped>
/* Optimize resizable sections with hardware acceleration hints */
.tickets-section,
.docs-section {
  will-change: max-height;
  transform: translateZ(0); /* Force GPU acceleration */
  backface-visibility: hidden;
  perspective: 1000px;
  transition: max-height 0.2s cubic-bezier(0.25, 1, 0.5, 1); /* Optimized easing function */
}

/* Remove transition during active resizing to prevent lag */
:global(.resize-active) .tickets-section,
:global(.resize-active) .docs-section {
  transition: none !important;
}

/* Styles for resizer handle, active state, etc. */
.resizer-handle {
  touch-action: none;
  position: relative;
  z-index: 1;
  height: 4px;
  margin: 0;
  cursor: ns-resize;
  background-color: rgba(51, 65, 85, 0.25);
  border-top: 1px solid rgba(71, 85, 105, 0.3);
  border-bottom: 1px solid rgba(71, 85, 105, 0.3);
}

.resizer-handle:hover {
  background-color: rgba(51, 65, 85, 0.4);
}

.resizer-handle:active,
.resizer-handle.active {
  background-color: rgba(96, 165, 250, 0.3);
}

/* Keep the blue line indicator on hover/active, but make it more subtle */
.resizer-handle:hover::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  height: 0.5px; /* Thinner line on hover */
  background-color: rgba(96, 165, 250, 0.3); /* Much more transparent blue */
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.5; /* Lower opacity */
  z-index: 5;
  pointer-events: none;
}

/* Slightly more visible but still subtle when actively resizing */
.resizer-handle.active::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  height: 0.5px;
  background-color: rgba(96, 165, 250, 0.5); /* More visible when active */
  top: 50%;
  transform: translateY(-50%);
  opacity: 0.6;
  z-index: 5;
  pointer-events: none;
}

/* Visual feedback for resize cursor position */
:global(.resize-active) {
  cursor: ns-resize !important;
  user-select: none !important;
}

:global(.resize-active *) {
  user-select: none !important;
  pointer-events: none !important;
}

/* Ensure the resizer itself remains interactive during resize */
:global(.resize-active .resizer-handle) {
  pointer-events: auto !important;
}
</style>