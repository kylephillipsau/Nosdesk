import welcomeData from '@/data/documentation/welcome.json';
import quickStartData from '@/data/documentation/quick-start.json';
import createTicketData from '@/data/documentation/create-ticket.json';
import categoriesData from '@/data/documentation/categories.json';

export interface Article {
  id: string;
  title: string;
  description: string;
  content?: string;
  category: string;
  author: string;
  lastUpdated: string;
  status: 'published' | 'draft';
  icon?: string; // Optional icon for the document
}

export interface Category {
  id: string;
  name: string;
  articles: string[];
  content?: string;  // Optional content for the category itself
  lastUpdated?: string;
  author?: string;
  icon?: string;     // Optional icon for the category
}

// Map of article IDs to article data
const articlesMap: Record<string, Article> = {
  'welcome': welcomeData as Article,
  'quick-start': quickStartData as Article,
  'create-ticket': createTicketData as Article
};

// Categories data
const categories: Category[] = categoriesData as Category[];

/**
 * Get all categories with their article metadata
 */
export const getCategories = async (): Promise<Category[]> => {
  return categories.map(category => ({
    ...category,
    // Don't include article content in the list view
    articles: category.articles
  }));
};

/**
 * Get all articles with metadata (no content)
 */
export const getAllArticles = async (): Promise<Article[]> => {
  return Object.values(articlesMap).map(article => {
    const { content, ...metadata } = article;
    return metadata;
  });
};

/**
 * Get article by ID
 */
export const getArticleById = async (id: string): Promise<Article | null> => {
  return articlesMap[id] || null;
};

/**
 * Search articles by query
 */
export const searchArticles = async (query: string): Promise<Article[]> => {
  const lowerQuery = query.toLowerCase();
  return Object.values(articlesMap)
    .filter(article => 
      article.title.toLowerCase().includes(lowerQuery) ||
      article.description.toLowerCase().includes(lowerQuery)
    )
    .map(article => {
      const { content, ...metadata } = article;
      return metadata;
    });
};

/**
 * Save article
 */
export const saveArticle = async (article: Article): Promise<Article> => {
  // In a real app, this would save to a backend API
  // For now, we'll just update our local state
  articlesMap[article.id] = article;
  
  // Update category if needed
  const currentCategory = categories.find(c => c.id === article.category);
  if (currentCategory && !currentCategory.articles.includes(article.id)) {
    currentCategory.articles.push(article.id);
  }
  
  return article;
};

/**
 * Create a new article
 */
export const createArticle = async (article: Omit<Article, 'id'>): Promise<Article> => {
  // Generate a new ID
  const newId = `article-${Date.now()}`;
  const newArticle = {
    ...article,
    id: newId
  } as Article;
  
  // Save the article
  articlesMap[newId] = newArticle;
  
  // Add to category
  const category = categories.find(c => c.id === article.category);
  if (category) {
    category.articles.push(newId);
  }
  
  return newArticle;
};

/**
 * Get category by ID
 */
export const getCategoryById = async (id: string): Promise<Category | null> => {
  const category = categories.find(c => c.id === id);
  return category || null;
};

/**
 * Save category content
 */
export const saveCategoryContent = async (categoryId: string, content: string, author: string = 'System Admin'): Promise<Category | null> => {
  const category = categories.find(c => c.id === categoryId);
  if (!category) return null;
  
  category.content = content;
  category.lastUpdated = new Date().toISOString();
  category.author = author;
  
  return category;
};

export default {
  getCategories,
  getAllArticles,
  getArticleById,
  searchArticles,
  saveArticle,
  createArticle,
  getCategoryById,
  saveCategoryContent
}; 