<!-- MarkdownEditor.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { EditorState, Transaction } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { Schema, DOMParser } from 'prosemirror-model';
import { schema as basicSchema } from 'prosemirror-schema-basic';
import { addListNodes, wrapInList } from 'prosemirror-schema-list';
import { exampleSetup } from 'prosemirror-example-setup';
import { InputRule, inputRules } from 'prosemirror-inputrules';
import { MarkdownSerializer, defaultMarkdownSerializer, MarkdownParser, defaultMarkdownParser } from 'prosemirror-markdown';
import { toggleMark, lift as pmLift, selectParentNode as pmSelectParentNode } from 'prosemirror-commands';
import { undo as pmUndo, redo as pmRedo } from 'prosemirror-history';
import 'prosemirror-view/style/prosemirror.css';

const props = defineProps<{
  modelValue: string;
  placeholder?: string;
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'save': [value: string];
}>()

// Editor references
const editorRef = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;

// Custom schema with headings, lists, and code blocks
const mySchema = new Schema({
  nodes: addListNodes(basicSchema.spec.nodes.append({
    heading: {
      content: 'inline*',
      group: 'block',
      attrs: { level: { default: 1 } },
      parseDOM: [
        { tag: 'h1', attrs: { level: 1 } },
        { tag: 'h2', attrs: { level: 2 } },
        { tag: 'h3', attrs: { level: 3 } },
        { tag: 'h4', attrs: { level: 4 } },
        { tag: 'h5', attrs: { level: 5 } },
        { tag: 'h6', attrs: { level: 6 } },
      ],
      toDOM(node) { return ['h' + node.attrs.level, 0]; }
    },
    blockquote: {
      content: 'block+',
      group: 'block',
      parseDOM: [{ tag: 'blockquote' }],
      toDOM() { return ['blockquote', 0] }
    },
    code_block: {
      content: 'text*',
      group: 'block',
      parseDOM: [{ tag: 'pre' }],
      toDOM() { return ['pre', 0]; }
    }
  }), 'paragraph block*', 'block'),
  marks: basicSchema.spec.marks, // Ensure strong and em are included
});

// Create Markdown parser and serializer
const parser = new MarkdownParser(mySchema, defaultMarkdownParser.tokenizer, defaultMarkdownParser.tokens);
const serializer = defaultMarkdownSerializer;

// Define input rules for Markdown-like list behavior
const markdownInputRules = inputRules({
  rules: [
    new InputRule(/^\s*[-*]\s$/, (state: EditorState, match: RegExpMatchArray, start: number, end: number) => {
      const tr = state.tr.delete(start, end);
      return wrapInList(mySchema.nodes.bullet_list)(state) ? tr : null;
    }),
    new InputRule(/^\s*\d+\.\s$/, (state: EditorState, match: RegExpMatchArray, start: number, end: number) => {
      const tr = state.tr.delete(start, end);
      return wrapInList(mySchema.nodes.ordered_list)(state) ? tr : null;
    }),
  ],
});

// Custom dropdown state
const typeMenuRef = ref<HTMLElement | null>(null);
const typeButtonRef = ref<HTMLElement | null>(null);
const insertMenuRef = ref<HTMLElement | null>(null);
const insertButtonRef = ref<HTMLElement | null>(null);
const moreMenuRef = ref<HTMLElement | null>(null);
const moreButtonRef = ref<HTMLElement | null>(null);

const showTypeMenu = ref(false);
const showInsertMenu = ref(false);
const showMoreMenu = ref(false);

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as Node;
  
  // Handle Type menu
  if (showTypeMenu.value && typeMenuRef.value && typeButtonRef.value) {
    if (!typeMenuRef.value.contains(target) && !typeButtonRef.value.contains(target)) {
      showTypeMenu.value = false;
    }
  }
  
  // Handle Insert menu
  if (showInsertMenu.value && insertMenuRef.value && insertButtonRef.value) {
    if (!insertMenuRef.value.contains(target) && !insertButtonRef.value.contains(target)) {
      showInsertMenu.value = false;
    }
  }
  
  // Handle More menu
  if (showMoreMenu.value && moreMenuRef.value && moreButtonRef.value) {
    if (!moreMenuRef.value.contains(target) && !moreButtonRef.value.contains(target)) {
      showMoreMenu.value = false;
    }
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  // Save on Ctrl+S or Cmd+S
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    emit('save', view ? serializer.serialize(view.state.doc) : props.modelValue);
    return;
  }

  if (event.key === 'Escape') {
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

// Event listeners for click outside
onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});

// Function to focus the editor
const focusEditor = () => {
  if (view) {
    view.focus();
  }
};

// Functions to handle actions
const setBlockType = (type: string, attrs: any = {}) => {
  if (!view) return;
  const { state, dispatch } = view;
  const { tr } = state;
  const { from, to } = state.selection;
  tr.setBlockType(from, to, mySchema.nodes[type], attrs);
  dispatch(tr);
};

const toggleBulletList = () => {
  if (!view) return;
  wrapInList(mySchema.nodes.bullet_list)(view.state, view.dispatch);
};

const toggleOrderedList = () => {
  if (!view) return;
  wrapInList(mySchema.nodes.ordered_list)(view.state, view.dispatch);
};

const insertLink = () => {
  if (!view) return;
  const { state, dispatch } = view;
  const url = prompt('Enter URL for the link:', 'https://');
  if (url) {
    const { from, to } = state.selection;
    const tr = state.tr;
    if (from === to) {
      const text = prompt('Enter link text:', 'Link');
      if (text) {
        tr.insertText(text, from, from);
        tr.addMark(from, from + text.length, mySchema.marks.link.create({ href: url }));
      }
    } else {
      tr.addMark(from, to, mySchema.marks.link.create({ href: url }));
    }
    dispatch(tr);
  }
};

const toggleStrong = () => {
  if (!view) return;
  toggleMark(mySchema.marks.strong)(view.state, view.dispatch);
};

const toggleEm = () => {
  if (!view) return;
  toggleMark(mySchema.marks.em)(view.state, view.dispatch);
};

const undo = () => {
  if (!view) return;
  pmUndo(view.state, view.dispatch);
};

const redo = () => {
  if (!view) return;
  pmRedo(view.state, view.dispatch);
};

const liftBlock = () => {
  if (!view) return;
  pmLift(view.state, view.dispatch);
};

const selectParent = () => {
  if (!view) return;
  pmSelectParentNode(view.state, view.dispatch);
};

// Save content
const saveContent = () => {
  if (view) {
    emit('save', serializer.serialize(view.state.doc));
  }
};

// Watch for external changes to modelValue
watch(() => props.modelValue, (newValue) => {
  if (view && newValue !== serializer.serialize(view.state.doc)) {
    const state = EditorState.create({
      schema: mySchema,
      plugins: [...exampleSetup({ schema: mySchema, menuBar: false }), markdownInputRules],
      doc: newValue ? parser.parse(newValue) : undefined
    });
    view.updateState(state);
  }
}, { immediate: true });

onMounted(() => {
  if (!editorRef.value) return;

  const state = EditorState.create({
    schema: mySchema,
    plugins: [...exampleSetup({ schema: mySchema, menuBar: false }), markdownInputRules],
    doc: props.modelValue ? parser.parse(props.modelValue) : undefined
  });

  view = new EditorView(editorRef.value, {
    state,
    dispatchTransaction(transaction: Transaction) {
      if (!view) return;
      
      view.updateState(view.state.apply(transaction));
      
      // Only emit content changes if the document actually changed
      if (transaction.docChanged) {
        const content = serializer.serialize(view.state.doc);
        emit('update:modelValue', content);
      }
    }
  });
});
</script>

<template>
  <div class="markdown-editor">
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
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
            v-for="(type, index) in [
              { label: 'Plain', action: () => setBlockType('paragraph') },
              { label: 'Heading 1', action: () => setBlockType('heading', { level: 1 }) },
              { label: 'Heading 2', action: () => setBlockType('heading', { level: 2 }) },
              { label: 'Heading 3', action: () => setBlockType('heading', { level: 3 }) },
              { label: 'Blockquote', action: () => setBlockType('blockquote') },
              { label: 'Code Block', action: () => setBlockType('code_block') }
            ]"
            :key="index"
            @click="type.action(); showTypeMenu = false"
            class="dropdown-item"
            role="menuitem"
          >
            {{ type.label }}
          </button>
        </div>
      </div>

      <div class="toolbar-divider"></div>

      <!-- Formatting Buttons -->
      <button
        @click="toggleStrong"
        class="toolbar-button"
        title="Bold"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"></path>
          <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"></path>
        </svg>
      </button>
      <button
        @click="toggleEm"
        class="toolbar-button"
        title="Italic"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="19" y1="4" x2="10" y2="4"></line>
          <line x1="14" y1="20" x2="5" y2="20"></line>
          <line x1="15" y1="4" x2="9" y2="20"></line>
        </svg>
      </button>

      <div class="toolbar-divider"></div>

      <!-- Insert Dropdown -->
      <div class="relative">
        <button
          ref="insertButtonRef"
          @click="toggleInsertMenu"
          class="toolbar-button"
          aria-haspopup="true"
          :aria-expanded="showInsertMenu"
          title="Insert"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
            v-for="(item, index) in [
              { label: 'Bullet List', action: toggleBulletList },
              { label: 'Numbered List', action: toggleOrderedList },
              { label: 'Link', action: insertLink }
            ]"
            :key="index"
            @click="item.action(); showInsertMenu = false"
            class="dropdown-item"
            role="menuitem"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <div class="toolbar-divider"></div>

      <!-- Undo/Redo Buttons -->
      <button
        @click="undo"
        class="toolbar-button"
        title="Undo"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 7v6h6"></path>
          <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
        </svg>
      </button>
      <button
        @click="redo"
        class="toolbar-button"
        title="Redo"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 7v6h-6"></path>
          <path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3L21 13"></path>
        </svg>
      </button>
    </div>
    
    <!-- Editor content with click handler -->
    <div 
      ref="editorRef" 
      @click="focusEditor"
      class="editor-container"
      :data-placeholder="placeholder || 'Write your content here...'"
    >
    </div>
  </div>
</template>

<style scoped>
.markdown-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid #374151;
  border-radius: 0.375rem;
  overflow: hidden;
  background-color: #1e293b;
  height: 100%;
}

.toolbar {
  display: flex;
  padding: 0.5rem;
  background-color: #314257;
  border-bottom: 1px solid #374151;
  flex-wrap: wrap;
  gap: 0.25rem;
  flex-shrink: 0; /* Prevent toolbar from shrinking */
}

.toolbar-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem 0.5rem;
  background-color: transparent;
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
  background-color: #334155;
  color: #3b82f6;
}

.toolbar-divider {
  width: 1px;
  background-color: #374151;
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
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
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

.editor-container {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  background-color: #212C42;
  color: #e2e8f0;
  min-height: 0; /* Allow container to shrink */
}

/* ProseMirror specific styles */
:deep(.ProseMirror) {
  outline: none;
  min-height: 100%;
}

:deep(.ProseMirror p) {
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

:deep(.ProseMirror h1) {
  font-size: 2rem;
  font-weight: 700;
  margin-top: 1rem;
  margin-bottom: 1rem;
  border-bottom: 1px solid #334155;
  padding-bottom: 0.5rem;
  line-height: 1.2;
}

:deep(.ProseMirror h2) {
  font-size: 1.5rem;
  font-weight: 700;
  margin-top: 1.5rem;
  margin-bottom: 1rem;
  line-height: 1.3;
}

:deep(.ProseMirror h3) {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 1rem;
  line-height: 1.4;
}

:deep(.ProseMirror blockquote) {
  border-left: 4px solid #3b82f6;
  padding-left: 1rem;
  margin-left: 0;
  margin-right: 0;
  color: #94a3b8;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

:deep(.ProseMirror pre) {
  background-color: #0F172B;
  padding: 0.75rem;
  border-radius: 0.375rem;
  overflow-x: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

:deep(.ProseMirror code) {
  background-color: #0F172B;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

:deep(.ProseMirror ul), :deep(.ProseMirror ol) {
  padding-left: 1.5rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

:deep(.ProseMirror li) {
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

:deep(.ProseMirror a) {
  color: #3b82f6;
  text-decoration: underline;
}

:deep(ul) {
  list-style-type: disc;
  padding-left: 1.5rem;
}

:deep(ol) {
  list-style-type: decimal;
  padding-left: 1.5rem;
}

:deep(li) {
  margin-bottom: 0.25rem;
}

/* Placeholder text */
:deep(.ProseMirror p.is-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: #64748b;
  pointer-events: none;
  height: 0;
}

/* Empty editor placeholder */
.editor-container:empty::before {
  content: attr(data-placeholder);
  color: #64748b;
  pointer-events: none;
}
</style> 