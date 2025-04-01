import { Schema } from 'prosemirror-model';

// Basic schema for documents
// This can be expanded depending on your requirements
export const schema = new Schema({
  nodes: {
    // The top level document node
    doc: {
      content: 'block+'
    },
    
    // Paragraph node - a basic block node
    paragraph: {
      group: 'block',
      content: 'inline*',
      toDOM() { return ['p', 0] }
    },
    
    // Headings of different levels
    heading: {
      attrs: { level: { default: 1 } },
      content: 'inline*',
      group: 'block',
      defining: true,
      toDOM(node) { return [`h${node.attrs.level}`, 0] }
    },
    
    // Bullet list
    bullet_list: {
      group: 'block',
      content: 'list_item+',
      toDOM() { return ['ul', 0] }
    },
    
    // Ordered list
    ordered_list: {
      group: 'block',
      content: 'list_item+',
      attrs: { order: { default: 1 } },
      toDOM(node) { return ['ol', { start: node.attrs.order !== 1 ? node.attrs.order : null }, 0] }
    },
    
    // List item
    list_item: {
      content: 'paragraph block*',
      defining: true,
      toDOM() { return ['li', 0] }
    },
    
    // Blockquote
    blockquote: {
      content: 'block+',
      group: 'block',
      toDOM() { return ['blockquote', 0] }
    },
    
    // Code block
    code_block: {
      content: 'text*',
      group: 'block',
      code: true,
      defining: true,
      toDOM() { return ['pre', ['code', 0]] }
    },
    
    // Horizontal rule
    horizontal_rule: {
      group: 'block',
      toDOM() { return ['hr'] }
    },
    
    // Basic text node
    text: {
      group: 'inline'
    },
    
    // Hard break (line break)
    hard_break: {
      inline: true,
      group: 'inline',
      selectable: false,
      toDOM() { return ['br'] }
    },
    
    // Image with basic attributes
    image: {
      inline: true,
      attrs: {
        src: {},
        alt: { default: null },
        title: { default: null }
      },
      group: 'inline',
      draggable: true,
      toDOM(node) { return ['img', node.attrs] }
    }
  },
  
  marks: {
    // Strong text (bold)
    strong: {
      toDOM() { return ['strong', 0] }
    },
    
    // Emphasized text (italic)
    em: {
      toDOM() { return ['em', 0] }
    },
    
    // Code/monospace text
    code: {
      toDOM() { return ['code', 0] }
    },
    
    // Links with href and optional title
    link: {
      attrs: {
        href: {},
        title: { default: null }
      },
      inclusive: false,
      toDOM(node) { return ['a', node.attrs, 0] }
    },
    
    // Strikethrough text
    strikethrough: {
      toDOM() { return ['s', 0] }
    },
    
    // Underlined text
    underline: {
      toDOM() { return ['u', 0] }
    }
  }
}); 