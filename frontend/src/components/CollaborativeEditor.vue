<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed } from "vue";
import * as Y from "yjs";
import { PermanentUserData } from "yjs";
import { WebsocketProvider } from "y-websocket";
import { EditorView } from "prosemirror-view";
import { EditorState } from "prosemirror-state";
import { schema } from "@/components/editor/schema";
import { useAuthStore } from "@/stores/auth";
import UserAvatar from "./UserAvatar.vue";
import {
  ySyncPlugin,
  yCursorPlugin,
  yUndoPlugin,
  undo,
  redo,
  initProseMirrorDoc,
} from "y-prosemirror";
import { keymap } from "prosemirror-keymap";
import {
  toggleMark,
  setBlockType,
  chainCommands,
  exitCode,
  createParagraphNear,
  liftEmptyBlock,
  splitBlock,
} from "prosemirror-commands";
import {
  wrapInList,
  splitListItem,
  liftListItem,
  sinkListItem,
} from "prosemirror-schema-list";
import "prosemirror-view/style/prosemirror.css";
import { Schema } from "prosemirror-model";
import { Plugin, PluginKey } from "prosemirror-state";

// Import individual components instead of exampleSetup
import { baseKeymap } from "prosemirror-commands";
import { dropCursor } from "prosemirror-dropcursor";
import { gapCursor } from "prosemirror-gapcursor";
import {
  inputRules,
  wrappingInputRule,
  textblockTypeInputRule,
  smartQuotes,
  emDash,
  ellipsis,
} from "prosemirror-inputrules";

// Props
interface Props {
  docId: string;
  placeholder?: string;
  modelValue?: string;
  isBinaryUpdate?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: "Start typing...",
  modelValue: "",
  isBinaryUpdate: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

// Get auth store for user info
const authStore = useAuthStore();

// Refs for template
const editorElement = ref<HTMLElement | null>(null);
const isConnected = ref(false);

// State for connected users
const connectedUsers = ref<{ id: string; user: any }[]>([]);

// Custom dropdown state for toolbar
const typeMenuRef = ref<HTMLElement | null>(null);
const typeButtonRef = ref<HTMLElement | null>(null);
const insertMenuRef = ref<HTMLElement | null>(null);
const insertButtonRef = ref<HTMLElement | null>(null);
const moreMenuRef = ref<HTMLElement | null>(null);
const moreButtonRef = ref<HTMLElement | null>(null);

const showTypeMenu = ref(false);
const showInsertMenu = ref(false);
const showMoreMenu = ref(false);

// Global variables - mirroring the demo approach exactly
let ydoc: Y.Doc | null = null;
let provider: WebsocketProvider | null = null;
let yXmlFragment: Y.XmlFragment | null = null;
let editorView: EditorView | null = null;

// Enhanced logging
const log = {
  info: (message: string, ...args: any[]) =>
    console.log(`[YJS-Editor] ${message}`, ...args),
  error: (message: string, ...args: any[]) =>
    console.error(`[YJS-Editor] ${message}`, ...args),
  debug: (message: string, ...args: any[]) =>
    console.debug(`[YJS-Editor] ${message}`, ...args),
  warn: (message: string, ...args: any[]) =>
    console.warn(`[YJS-Editor] ${message}`, ...args),
};

// Create custom input rules function to replace exampleSetup
const buildInputRules = (schema: Schema) => {
  const rules = [];

  // Heading rules: # for h1, ## for h2, etc.
  if (schema.nodes.heading) {
    for (let i = 1; i <= 6; i++) {
      rules.push(
        textblockTypeInputRule(
          new RegExp(`^(#{${i}})\\s$`),
          schema.nodes.heading,
          { level: i }
        )
      );
    }
  }

  // Blockquote rule: > followed by space
  if (schema.nodes.blockquote) {
    rules.push(wrappingInputRule(/^\s*>\s$/, schema.nodes.blockquote));
  }

  // Code block rule: ``` followed by space
  if (schema.nodes.code_block) {
    rules.push(textblockTypeInputRule(/^```\s$/, schema.nodes.code_block));
  }

  // List rules
  if (schema.nodes.bullet_list) {
    // Bullet list: * or - or + followed by space
    // More permissive rule to catch various list markers
    rules.push(wrappingInputRule(/^\s*([-*+])\s$/, schema.nodes.bullet_list));
  }

  if (schema.nodes.ordered_list) {
    // Ordered list: 1. followed by space
    // Allow any digit sequence followed by period or right parenthesis
    rules.push(
      wrappingInputRule(
        /^\s*(\d+)[.)]\s$/,
        schema.nodes.ordered_list,
        (match) => ({ order: +match[1] }),
        (match, node) => node.childCount + node.attrs.order === +match[1]
      )
    );
  }

  // Smart quotes, ellipsis, em-dash
  rules.push(...smartQuotes, ellipsis, emDash);

  return inputRules({ rules });
};

// Create custom keymap for list behaviors
const createListKeymap = (schema: Schema) => {
  const keys: { [key: string]: any } = {};

  // Add key bindings for list behavior
  if (schema.nodes.bullet_list && schema.nodes.list_item) {
    // Add Enter key handling for bullet lists - this makes lists continue when pressing Enter
    keys["Enter"] = splitListItem(schema.nodes.list_item);

    // Tab to indent list items (increase nesting level)
    keys["Tab"] = sinkListItem(schema.nodes.list_item);

    // Shift-Tab to outdent list items (decrease nesting level)
    keys["Shift-Tab"] = liftListItem(schema.nodes.list_item);

    // Add keyboard shortcuts for toggling lists
    keys["Mod-Shift-8"] = wrapInList(schema.nodes.bullet_list); // Ctrl+Shift+8 for bullet list

    if (schema.nodes.ordered_list) {
      keys["Mod-Shift-9"] = wrapInList(schema.nodes.ordered_list); // Ctrl+Shift+9 for ordered list
    }
  }

  return keys;
};

// Initialize editor following the official Yjs demo pattern
const initEditor = async () => {
  if (!editorElement.value) return;

  try {
    log.info("Initializing collaborative editor with docId:", props.docId);

    // 1. Create new Yjs document
    ydoc = new Y.Doc();

    // 2. Create the websocket provider
    const wsUrl =
      import.meta.env.VITE_WS_SERVER_URL ||
      `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${
        window.location.hostname
      }:8080/api/collaboration/ws`;

    log.info(
      `Connecting to WebSocket server at: ${wsUrl} with document ID: ${props.docId}`
    );

    provider = new WebsocketProvider(wsUrl, props.docId, ydoc);

    // 3. Set base awareness information for user identification
    provider.awareness.setLocalState({
      user: {
        name: getUserDisplayName(),
        color: getRandomColor(),
        uuid: authStore.user?.uuid || undefined,
      },
    });

    // 4. Get the XML fragment and initialize ProseMirror document
    yXmlFragment = ydoc.getXmlFragment("prosemirror");
    const { doc, meta } = initProseMirrorDoc(yXmlFragment, schema);

    // 5. Create the editor view - following the exact pattern in the official demo
    editorView = new EditorView(editorElement.value, {
      state: EditorState.create({
        doc,
        schema,
        plugins: [
          ySyncPlugin(yXmlFragment, { mapping: meta }),
          yCursorPlugin(provider.awareness),
          yUndoPlugin(),
          keymap({
            "Mod-z": undo,
            "Mod-y": redo,
            "Mod-Shift-z": redo,
          }),
          // Add list handling keymap - this is crucial for proper list behavior
          keymap(createListKeymap(schema)),
          // Add individual plugins instead of exampleSetup
          buildInputRules(schema), // Custom markdown input rules
          keymap(baseKeymap), // Basic key bindings
          dropCursor(), // Shows cursor when dragging
          gapCursor(), // Allows clicking between blocks
        ],
      }),
    });

    // 6. Set up connection status handler
    provider.on(
      "status",
      (event: { status: "connected" | "disconnected" | "connecting" }) => {
        isConnected.value = event.status === "connected";
        log.info(`WebSocket connection status: ${event.status}`);
      }
    );

    // 7. Add awareness change listener to update connected users
    provider.awareness.on("change", () => {
      updateConnectedUsers();
    });

    // 8. For debugging purposes
    window.example = {
      provider,
      ydoc,
      yXmlFragment,
      editorView,
    };

    log.debug("Editor initialized successfully");
  } catch (error) {
    log.error("Error initializing editor:", error);
  }
};

// Helper function to get random color for user
const getRandomColor = () => {
  const colors = [
    "#f87171",
    "#fb923c",
    "#fbbf24",
    "#a3e635",
    "#34d399",
    "#22d3ee",
    "#60a5fa",
    "#a78bfa",
  ];
  return colors[Math.floor(Math.random() * colors.length)];
};

// Helper function to get user display name
const getUserDisplayName = () => {
  if (!authStore.user) {
    return "Guest " + Math.floor(Math.random() * 1000);
  }

  // Use the user's name from the auth store
  return authStore.user.name;
};

// Simple function to update connected users UI
const updateConnectedUsers = () => {
  if (!provider) return;

  try {
    const states = provider.awareness.getStates();
    const users: { id: string; user: any }[] = [];

    // Convert Map to array and exclude the current user
    states.forEach((state, clientId) => {
      if (
        state &&
        state.user &&
        provider &&
        clientId !== provider.awareness.clientID
      ) {
        // Only include users with valid user data
        if (state.user.name && typeof state.user.name === "string") {
          users.push({
            id: String(clientId),
            user: state.user,
          });
        }
      }
    });

    connectedUsers.value = users;
  } catch (error) {
    log.error("Error updating connected users:", error);
  }
};

// Simple function to focus the editor
const focusEditor = () => {
  if (editorView) {
    editorView.focus();
  }
};

// Event listeners for click outside
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Node;

  // Handle Type menu
  if (showTypeMenu.value && typeMenuRef.value && typeButtonRef.value) {
    if (
      !typeMenuRef.value.contains(target) &&
      !typeButtonRef.value.contains(target)
    ) {
      showTypeMenu.value = false;
    }
  }

  // Handle Insert menu
  if (showInsertMenu.value && insertMenuRef.value && insertButtonRef.value) {
    if (
      !insertMenuRef.value.contains(target) &&
      !insertButtonRef.value.contains(target)
    ) {
      showInsertMenu.value = false;
    }
  }

  // Handle More menu
  if (showMoreMenu.value && moreMenuRef.value && moreButtonRef.value) {
    if (
      !moreMenuRef.value.contains(target) &&
      !moreButtonRef.value.contains(target)
    ) {
      showMoreMenu.value = false;
    }
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    if (showTypeMenu.value) {
      showTypeMenu.value = false;
      typeButtonRef.value?.focus();
    }
    if (showInsertMenu.value) {
      showInsertMenu.value = false;
      insertButtonRef.value?.focus();
    }
    if (showMoreMenu.value) {
      showMoreMenu.value = false;
      moreButtonRef.value?.focus();
    }
  }
};

const toggleTypeMenu = () => {
  showTypeMenu.value = !showTypeMenu.value;
  if (showTypeMenu.value) {
    showInsertMenu.value = false;
    showMoreMenu.value = false;
  }
};

const toggleInsertMenu = () => {
  showInsertMenu.value = !showInsertMenu.value;
  if (showInsertMenu.value) {
    showTypeMenu.value = false;
    showMoreMenu.value = false;
  }
};

const toggleMoreMenu = () => {
  showMoreMenu.value = !showMoreMenu.value;
  if (showMoreMenu.value) {
    showTypeMenu.value = false;
    showInsertMenu.value = false;
  }
};

// Functions to handle toolbar actions
const setHeading = (level: number) => {
  if (!editorView) return;
  const attrs = { level };
  setBlockType(schema.nodes.heading, attrs)(
    editorView.state,
    editorView.dispatch
  );
};

const toggleBold = () => {
  if (!editorView) return;
  toggleMark(schema.marks.strong)(editorView.state, editorView.dispatch);
};

const toggleItalic = () => {
  if (!editorView) return;
  toggleMark(schema.marks.em)(editorView.state, editorView.dispatch);
};

const toggleBlockquote = () => {
  if (!editorView) return;
  setBlockType(schema.nodes.blockquote, {})(
    editorView.state,
    editorView.dispatch
  );
};

const toggleCodeBlock = () => {
  if (!editorView) return;
  setBlockType(schema.nodes.code_block, {})(
    editorView.state,
    editorView.dispatch
  );
};

const setParagraph = () => {
  if (!editorView) return;
  setBlockType(schema.nodes.paragraph, {})(
    editorView.state,
    editorView.dispatch
  );
};

const toggleBulletList = () => {
  if (!editorView) return;
  wrapInList(schema.nodes.bullet_list)(editorView.state, editorView.dispatch);
};

const toggleOrderedList = () => {
  if (!editorView) return;
  wrapInList(schema.nodes.ordered_list)(editorView.state, editorView.dispatch);
};

const insertLink = () => {
  if (!editorView) return;
  const { state, dispatch } = editorView;
  const url = prompt("Enter URL for the link:", "https://");
  if (url) {
    const { from, to } = state.selection;
    const tr = state.tr;
    if (from === to) {
      const text = prompt("Enter link text:", "Link");
      if (text) {
        tr.insertText(text, from, from);
        tr.addMark(
          from,
          from + text.length,
          schema.marks.link.create({ href: url })
        );
      }
    } else {
      tr.addMark(from, to, schema.marks.link.create({ href: url }));
    }
    dispatch(tr);
  }
};

const undoEdit = () => {
  if (!editorView) return;
  undo(editorView.state);
};

const redoEdit = () => {
  if (!editorView) return;
  redo(editorView.state);
};

// Cleanup function
const cleanup = () => {
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  if (provider) {
    // Remove awareness listener
    if (provider.awareness) {
      provider.awareness.off("change", updateConnectedUsers);
    }

    provider.disconnect();
    provider = null;
  }
  if (ydoc) {
    ydoc.destroy();
    ydoc = null;
  }
  yXmlFragment = null;

  // Reset connected users
  connectedUsers.value = [];

  // Clean up global references
  window.example = undefined;
};

// Watch for changes in props.docId
watch(
  () => props.docId,
  (newDocId, oldDocId) => {
    if (newDocId !== oldDocId) {
      log.info(
        `Document ID changed from ${oldDocId} to ${newDocId}, reinitializing...`
      );
      cleanup();
      // Short delay to ensure cleanup completes
      setTimeout(() => {
        initEditor();
      }, 100);
    }
  }
);

// Watch for changes in the auth user and update awareness
watch(
  () => authStore.user,
  () => {
    if (provider && provider.awareness) {
      const currentState = provider.awareness.getLocalState() || {};
      provider.awareness.setLocalState({
        ...currentState,
        user: {
          ...currentState?.user,
          name: getUserDisplayName(),
        },
      });
      log.info(`Updated collaborative user name to: ${getUserDisplayName()}`);
    }
  }
);

// Add method to update editor state
const updateState = (newState: EditorState) => {
  if (editorView) {
    editorView.updateState(newState);
  }
};

// Watch for changes in model value
watch(
  () => props.modelValue,
  (newValue, oldValue) => {
    if (newValue !== oldValue && editorView && !props.isBinaryUpdate) {
      try {
        // Only update if JSON is valid and the editor is already initialized
        const parsedDoc = JSON.parse(newValue);
        const newDoc = schema.nodeFromJSON(parsedDoc);

        if (newDoc) {
          // Create a transaction that preserves selection
          const tr = editorView.state.tr.replaceWith(
            0,
            editorView.state.doc.content.size,
            newDoc.content
          );
          editorView.dispatch(tr);
          log.debug("Applied updated content from model");
        }
      } catch (error) {
        log.error("Error applying content update:", error);
      }
    }
  }
);

// Helper function to log environment variables for debugging
const logEnvironmentInfo = () => {
  log.info("Environment info:");
  log.info(
    `- VITE_WS_SERVER_URL: ${import.meta.env.VITE_WS_SERVER_URL || "Not set"}`
  );
  log.info(`- window.location.host: ${window.location.host}`);
  log.info(`- window.location.protocol: ${window.location.protocol}`);
  log.info(`- window.location.origin: ${window.location.origin}`);
};

// Add a function to inspect relative positions
const debugRelativePositions = () => {
  if (!editorView || !ydoc || !yXmlFragment || !provider) return;

  try {
    // Get current ProseMirror selection
    const selection = editorView.state.selection;

    // Log all current absolute positions
    log.debug("Current selection (absolute positions):", {
      anchor: selection.anchor,
      head: selection.head,
      from: selection.from,
      to: selection.to,
    });

    // Create relative positions from current selection
    const relAnchor = Y.createRelativePositionFromTypeIndex(
      yXmlFragment,
      selection.anchor
    );
    const relHead = Y.createRelativePositionFromTypeIndex(
      yXmlFragment,
      selection.head
    );

    // Log the relative positions as JSON for inspection
    log.debug("Relative positions:", {
      anchor: JSON.stringify(relAnchor),
      head: JSON.stringify(relHead),
    });

    // Try to convert back to absolute positions
    const absAnchor = Y.createAbsolutePositionFromRelativePosition(
      relAnchor,
      ydoc
    );
    const absHead = Y.createAbsolutePositionFromRelativePosition(relHead, ydoc);

    log.debug("Converted back to absolute positions:", {
      anchor: absAnchor ? absAnchor.index : "conversion failed",
      head: absHead ? absHead.index : "conversion failed",
    });

    // Check what's in the awareness states
    const states = provider.awareness.getStates();
    log.debug(`Awareness states (${states.size} clients):`);

    states.forEach((state, clientId) => {
      if (provider) {
        const isLocal = clientId === provider.awareness.clientID;
        if (state.cursor) {
          log.debug(`Client ${clientId}${isLocal ? " (local)" : ""}:`, {
            anchor: state.cursor.anchor,
            head: state.cursor.head,
            // If these positions are actually stored as relative positions in awareness,
            // let's try to inspect them directly
            anchorRelative:
              typeof state.cursor.anchorRelative === "object"
                ? JSON.stringify(state.cursor.anchorRelative)
                : "not available",
            headRelative:
              typeof state.cursor.headRelative === "object"
                ? JSON.stringify(state.cursor.headRelative)
                : "not available",
          });
        }
      }
    });
  } catch (error) {
    log.error("Error in debugRelativePositions:", error);
  }
};

// Add window debug methods
window.example = undefined; // Initialize with undefined until editor is created

// Update the global interface
declare global {
  interface Window {
    example?: any;
  }
}

onMounted(() => {
  initEditor();
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeydown);
});

onBeforeUnmount(() => {
  cleanup();
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="collaborative-editor">
    <!-- Toolbar -->
    <div class="toolbar">
      <!-- Type Dropdown -->
      <div class="relative">
        <button
          ref="typeButtonRef"
          @click="toggleTypeMenu"
          class="toolbar-button"
          aria-haspopup="true"
          :aria-expanded="showTypeMenu"
          title="Text Style"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M4 7V4h16v3"></path>
            <path d="M9 20h6"></path>
            <path d="M12 4v16"></path>
          </svg>
        </button>

        <!-- Type Menu Dropdown -->
        <div
          v-if="showTypeMenu"
          ref="typeMenuRef"
          class="dropdown-menu"
          role="menu"
          tabindex="-1"
        >
          <button
            @click="
              setParagraph();
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Plain
          </button>
          <button
            @click="
              setHeading(1);
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Heading 1
          </button>
          <button
            @click="
              setHeading(2);
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Heading 2
          </button>
          <button
            @click="
              setHeading(3);
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Heading 3
          </button>
          <button
            @click="
              toggleBlockquote();
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Blockquote
          </button>
          <button
            @click="
              toggleCodeBlock();
              showTypeMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Code Block
          </button>
        </div>
      </div>

      <div class="toolbar-divider"></div>

      <!-- Formatting Buttons -->
      <button @click="toggleBold" class="toolbar-button" title="Bold">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"></path>
          <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"></path>
        </svg>
      </button>
      <button @click="toggleItalic" class="toolbar-button" title="Italic">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="19" y1="4" x2="10" y2="4"></line>
          <line x1="14" y1="20" x2="5" y2="20"></line>
          <line x1="15" y1="4" x2="9" y2="20"></line>
        </svg>
      </button>

      <div class="toolbar-divider"></div>

      <!-- List buttons -->
      <button
        @click="toggleBulletList"
        class="toolbar-button"
        title="Bullet List"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <circle cx="3" cy="6" r="1"></circle>
          <circle cx="3" cy="12" r="1"></circle>
          <circle cx="3" cy="18" r="1"></circle>
        </svg>
      </button>
      <button
        @click="toggleOrderedList"
        class="toolbar-button"
        title="Numbered List"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="10" y1="6" x2="21" y2="6"></line>
          <line x1="10" y1="12" x2="21" y2="12"></line>
          <line x1="10" y1="18" x2="21" y2="18"></line>
          <path d="M4 6h1v4"></path>
          <path d="M4 10h2"></path>
          <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"></path>
        </svg>
      </button>

      <div class="toolbar-divider"></div>

      <!-- Insert Dropdown Menu with expanded options -->
      <div class="relative">
        <button
          ref="insertButtonRef"
          @click="toggleInsertMenu"
          class="toolbar-button"
          aria-haspopup="true"
          :aria-expanded="showInsertMenu"
          title="Insert"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
        </button>

        <!-- Insert Menu Dropdown -->
        <div
          v-if="showInsertMenu"
          ref="insertMenuRef"
          class="dropdown-menu"
          role="menu"
          tabindex="-1"
        >
          <button
            @click="
              toggleBulletList();
              showInsertMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Bullet List
          </button>
          <button
            @click="
              toggleOrderedList();
              showInsertMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Numbered List
          </button>
          <button
            @click="
              toggleBlockquote();
              showInsertMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Blockquote
          </button>
          <button
            @click="
              toggleCodeBlock();
              showInsertMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Code Block
          </button>
          <button
            @click="
              insertLink();
              showInsertMenu = false;
            "
            class="dropdown-item"
            role="menuitem"
          >
            Link
          </button>
        </div>
      </div>

      <div class="toolbar-divider"></div>

      <!-- Undo/Redo Buttons -->
      <button @click="undoEdit" class="toolbar-button" title="Undo">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 7v6h6"></path>
          <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
        </svg>
      </button>
      <button @click="redoEdit" class="toolbar-button" title="Redo">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 7v6h-6"></path>
          <path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3L21 13"></path>
        </svg>
      </button>

      <!-- Spacer to push connection controls to right -->
      <div class="flex-grow"></div>

      <!-- Connected users -->
      <div
        v-if="connectedUsers.length > 0"
        class="flex items-center gap-1 mr-2"
      >
        <div class="text-xs text-slate-300 mr-1">Editing with:</div>
        <div class="flex">
          <div
            v-for="(connectedUser, index) in connectedUsers"
            :key="connectedUser.id"
            class="flex items-center"
            :style="{ marginLeft: index > 0 ? '-8px' : '0' }"
            :title="
              connectedUser.user.name +
              (connectedUser.user.uuid
                ? ' (UUID: ' + connectedUser.user.uuid + ')'
                : '')
            "
            @click="
              () => {
                console.log('User data:', connectedUser.user);
              }
            "
          >
            <UserAvatar
              :name="connectedUser.user.uuid || connectedUser.user.name"
              :showName="false"
              size="xs"
              :clickable="!!connectedUser.user.uuid"
            />
          </div>
        </div>
      </div>

      <!-- Connection status indicator only -->
      <div class="connection-status" :class="{ connected: isConnected }">
        {{ isConnected ? "Connected" : "Syncing locally" }}
      </div>
    </div>

    <!-- Editor content with click handler -->
    <div
      id="editor"
      ref="editorElement"
      @click="focusEditor"
      class="editor-container"
      :data-placeholder="placeholder"
    ></div>
  </div>
</template>

<style>
.collaborative-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid #374151;
  border-radius: 0.375rem;
  overflow: hidden;
  background-color: #1e293b;
  height: 100%;
  width: 100%;
  position: relative;
}

.toolbar {
  display: flex;
  padding: 0.5rem;
  background-color: #314257;
  border-bottom: 1px solid #374151;
  flex-wrap: wrap;
  gap: 0.25rem;
  align-items: center;
}

.toolbar-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem 0.5rem;
  background-color: #45556c;
  border: none;
  border-radius: 0.25rem;
  color: #aebaca;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.toolbar-button:hover {
  background-color: #334155;
  color: #e6eaee;
}

.toolbar-button.active {
  color: #3b82f6;
}

.toolbar-divider {
  width: 1px;
  background-color: #91a1b8;
  margin: 0 0.5rem;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 0.25rem;
  width: 12rem;
  background-color: #1e293b;
  border: 1px solid #374151;
  border-radius: 0.375rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06);
  z-index: 50;
}

.dropdown-item {
  display: block;
  width: 100%;
  padding: 0.5rem 1rem;
  text-align: left;
  font-size: 0.875rem;
  color: #e2e8f0;
  background-color: transparent;
  border: none;
  cursor: pointer;
}

.dropdown-item:hover {
  background-color: #334155;
}

.connection-status {
  font-size: 0.875rem;
  color: #ef4444;
}

.connection-status.connected {
  color: #10b981;
}

.editor-container {
  padding: 1rem;
  background-color: #212c42;
  color: #e2e8f0;
  min-height: 250px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.ProseMirror {
  outline: none;
  min-height: 200px;
  height: 100%;
  flex: 1;
  position: relative;
}

.ProseMirror p {
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

.ProseMirror h1 {
  font-size: 2rem;
  font-weight: 700;
  margin-top: 1rem;
  margin-bottom: 1rem;
  border-bottom: 1px solid #334155;
  padding-bottom: 0.5rem;
  line-height: 1.2;
}

.ProseMirror h2 {
  font-size: 1.5rem;
  font-weight: 700;
  margin-top: 1.5rem;
  margin-bottom: 1rem;
  line-height: 1.3;
}

.ProseMirror h3 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 1rem;
  line-height: 1.4;
}

.ProseMirror blockquote {
  border-left: 4px solid #3b82f6;
  padding-left: 1rem;
  margin-left: 0;
  margin-right: 0;
  color: #94a3b8;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.ProseMirror pre {
  background-color: #0f172b;
  padding: 0.75rem;
  border-radius: 0.375rem;
  overflow-x: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
    "Liberation Mono", "Courier New", monospace;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.ProseMirror code {
  background-color: #0f172b;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
    "Liberation Mono", "Courier New", monospace;
}

.ProseMirror ul,
.ProseMirror ol {
  padding-left: 1.5rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.ProseMirror li {
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

/* Enhanced list styles */
.ProseMirror ul {
  list-style-type: disc;
  color: #e2e8f0;
}

.ProseMirror ul ul {
  list-style-type: circle;
}

.ProseMirror ul ul ul {
  list-style-type: square;
}

.ProseMirror ol {
  list-style-type: decimal;
  color: #e2e8f0;
}

.ProseMirror ol ol {
  list-style-type: lower-alpha;
}

.ProseMirror ol ol ol {
  list-style-type: lower-roman;
}

.ProseMirror li p {
  margin: 0.25rem 0;
}

.ProseMirror a {
  color: #3b82f6;
  text-decoration: underline;
}

.ProseMirror .yRemoteSelection {
  position: absolute;
  border-left: 2px solid;
  border-right: 2px solid;
  pointer-events: none;
  opacity: 0.5;
}

.ProseMirror .yRemoteSelectionHead {
  position: absolute;
  height: 1.2em;
  width: 2px;
  pointer-events: none;
}

/* Empty editor placeholder */
.editor-container:empty::before {
  content: attr(data-placeholder);
  color: #64748b;
  pointer-events: none;
}

/* Flex spacer */
.flex-grow {
  flex-grow: 1;
}

/* This gives the remote user caret. The colors are automatically overwritten*/
.ProseMirror-yjs-cursor {
  position: relative;
  margin-left: -1px;
  margin-right: -1px;
  border-left: 2px solid orange;
  border-right: 2px solid orange;
  border-color: orange;
  word-break: normal;
  pointer-events: none;
  opacity: 1;
  height: 1.2em;
}

/* This renders the username above the caret */
.ProseMirror-yjs-cursor > div {
  position: absolute;
  top: -1.5em;
  left: -2px;
  font-size: 12px;
  background-color: currentColor;
  font-family: sans-serif;
  font-weight: normal;
  line-height: normal;
  user-select: none;
  color: white;
  padding: 1px 5px;
  white-space: nowrap;
  border-radius: 4px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  z-index: 10;
}

/* Additional class for the y-prosemirror cursor plugin */
.ProseMirror .y-prosemirror-cursor {
  position: relative;
  margin-left: -1px;
  margin-right: -1px;
  border-left: 2px solid orange;
  border-right: 2px solid orange;
  border-color: orange;
  word-break: normal;
  pointer-events: none;
}

/* This renders the username with the y-prosemirror cursor */
.ProseMirror .y-prosemirror-cursor > div {
  position: absolute;
  top: -1.5em;
  left: -2px;
  font-size: 12px;
  background-color: currentColor;
  font-family: sans-serif;
  font-weight: normal;
  line-height: normal;
  user-select: none;
  color: white;
  padding: 1px 5px;
  white-space: nowrap;
  border-radius: 4px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  z-index: 10;
}

/* Custom cursor styles that will be applied by our cursorBuilder */
.custom-remote-cursor {
  position: relative;
  margin-left: -1px;
  margin-right: -1px;
  border-left: 2px solid;
  border-right: 2px solid;
  height: 1.2em;
  word-break: normal;
  pointer-events: none;
  z-index: 3;
}

.custom-remote-cursor > div {
  position: absolute;
  top: -1.5em;
  left: -2px;
  font-size: 12px;
  background-color: #4f46e5; /* Default color if user.color is not passed */
  font-family: sans-serif;
  font-weight: normal;
  line-height: normal;
  user-select: none;
  color: white;
  padding: 1px 5px;
  white-space: nowrap;
  border-radius: 4px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  z-index: 10;
}

/* this is a rough fix for the first cursor position when the first paragraph is empty */
.ProseMirror > .ProseMirror-yjs-cursor:first-child,
.ProseMirror > .y-prosemirror-cursor:first-child,
.ProseMirror > .yRemoteSelection:first-child {
  margin-top: 16px;
}

.ProseMirror p:first-child,
.ProseMirror h1:first-child,
.ProseMirror h2:first-child,
.ProseMirror h3:first-child,
.ProseMirror h4:first-child,
.ProseMirror h5:first-child,
.ProseMirror h6:first-child {
  margin-top: 16px;
}

/* Handle empty document state */
.ProseMirror:empty::before {
  content: attr(data-placeholder);
  color: #64748b;
  pointer-events: none;
  display: block;
  margin-top: 16px;
}

/* Ensure cursor has proper position in empty document */
.ProseMirror:empty + .ProseMirror-yjs-cursor,
.ProseMirror:empty + .y-prosemirror-cursor,
.ProseMirror:empty + .yRemoteSelection {
  margin-top: 16px;
}

/* this is a rough fix for the first cursor position when the first paragraph is empty */
.ProseMirror > .ProseMirror-yjs-cursor:first-child,
.ProseMirror > .y-prosemirror-cursor:first-child,
.ProseMirror > .yRemoteSelection:first-child {
  margin-top: 16px;
}
</style>
