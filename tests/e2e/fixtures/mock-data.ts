/**
 * Mock data fixtures for E2E tests.
 *
 * Provides realistic test data that mimics the Tauri backend responses.
 */

import type {
  Conversation,
  ConversationSummary,
  ProjectInfo,
  SearchResult,
  Message,
  ContentBlock,
} from "../../../src/lib/types";

// Helper to create ISO timestamps relative to now
function daysAgo(days: number): string {
  const date = new Date();
  date.setDate(date.getDate() - days);
  return date.toISOString();
}

function hoursAgo(hours: number): string {
  const date = new Date();
  date.setHours(date.getHours() - hours);
  return date.toISOString();
}

// ========== Project Info ==========

export const mockProjects: ProjectInfo[] = [
  {
    projectPath: "/Users/dev/projects/web-app",
    projectName: "web-app",
    conversationCount: 15,
    lastActivity: hoursAgo(2),
  },
  {
    projectPath: "/Users/dev/projects/api-server",
    projectName: "api-server",
    conversationCount: 8,
    lastActivity: daysAgo(1),
  },
  {
    projectPath: "/Users/dev/projects/mobile-app",
    projectName: "mobile-app",
    conversationCount: 5,
    lastActivity: daysAgo(3),
  },
];

// ========== Conversation Summaries ==========

export const mockConversationSummaries: ConversationSummary[] = [
  {
    id: "conv-001",
    projectName: "web-app",
    startTime: hoursAgo(2),
    lastTime: hoursAgo(1),
    preview: "Help me implement user authentication with JWT tokens",
    messageCount: 12,
    bookmarked: true,
  },
  {
    id: "conv-002",
    projectName: "web-app",
    startTime: daysAgo(1),
    lastTime: daysAgo(1),
    preview: "How do I add dark mode to my React application?",
    messageCount: 8,
    bookmarked: false,
  },
  {
    id: "conv-003",
    projectName: "api-server",
    startTime: daysAgo(2),
    lastTime: daysAgo(2),
    preview: "Fix the database connection timeout issue",
    messageCount: 6,
    bookmarked: false,
  },
  {
    id: "conv-004",
    projectName: "mobile-app",
    startTime: daysAgo(3),
    lastTime: daysAgo(3),
    preview: "Implement push notifications for iOS and Android",
    messageCount: 15,
    bookmarked: true,
  },
  {
    id: "conv-005",
    projectName: "web-app",
    startTime: daysAgo(5),
    lastTime: daysAgo(5),
    preview: "Optimize the webpack build configuration for production",
    messageCount: 4,
    bookmarked: false,
  },
];

// ========== Full Conversations ==========

function createTextBlock(text: string): ContentBlock {
  return { type: "text", content: text };
}

function createCodeBlock(code: string, language: string): ContentBlock {
  return { type: "code", content: code, language };
}

function createMessage(
  id: string,
  role: Message["role"],
  content: ContentBlock[],
  timestamp: string
): Message {
  return { id, role, content, timestamp };
}

export const mockConversations: Record<string, Conversation> = {
  "conv-001": {
    id: "conv-001",
    projectPath: "/Users/dev/projects/web-app",
    projectName: "web-app",
    startTime: hoursAgo(2),
    lastTime: hoursAgo(1),
    messages: [
      createMessage(
        "msg-001",
        "user",
        [createTextBlock("Help me implement user authentication with JWT tokens")],
        hoursAgo(2)
      ),
      createMessage(
        "msg-002",
        "assistant",
        [
          createTextBlock(
            "I'll help you implement JWT authentication. First, let's install the required packages:"
          ),
          createCodeBlock("npm install jsonwebtoken bcryptjs", "bash"),
          createTextBlock("Now create the authentication middleware:"),
          createCodeBlock(
            `import jwt from 'jsonwebtoken';

export function authenticateToken(req, res, next) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];

  if (!token) {
    return res.sendStatus(401);
  }

  jwt.verify(token, process.env.JWT_SECRET, (err, user) => {
    if (err) return res.sendStatus(403);
    req.user = user;
    next();
  });
}`,
            "typescript"
          ),
        ],
        hoursAgo(2)
      ),
      createMessage(
        "msg-003",
        "user",
        [createTextBlock("How do I generate the token on login?")],
        hoursAgo(1)
      ),
      createMessage(
        "msg-004",
        "assistant",
        [
          createTextBlock("Here's how to generate a JWT token when a user logs in:"),
          createCodeBlock(
            `import jwt from 'jsonwebtoken';
import bcrypt from 'bcryptjs';

async function login(email: string, password: string) {
  const user = await findUserByEmail(email);

  if (!user || !await bcrypt.compare(password, user.passwordHash)) {
    throw new Error('Invalid credentials');
  }

  const token = jwt.sign(
    { userId: user.id, email: user.email },
    process.env.JWT_SECRET,
    { expiresIn: '24h' }
  );

  return { token, user };
}`,
            "typescript"
          ),
        ],
        hoursAgo(1)
      ),
    ],
    totalTokens: { input: 450, output: 620 },
    bookmarked: true,
    tags: ["authentication", "jwt"],
  },
  "conv-002": {
    id: "conv-002",
    projectPath: "/Users/dev/projects/web-app",
    projectName: "web-app",
    startTime: daysAgo(1),
    lastTime: daysAgo(1),
    messages: [
      createMessage(
        "msg-010",
        "user",
        [createTextBlock("How do I add dark mode to my React application?")],
        daysAgo(1)
      ),
      createMessage(
        "msg-011",
        "assistant",
        [
          createTextBlock(
            "I'll show you how to implement dark mode using CSS variables and React context. First, create a theme context:"
          ),
          createCodeBlock(
            `import { createContext, useContext, useState, useEffect } from 'react';

const ThemeContext = createContext();

export function ThemeProvider({ children }) {
  const [theme, setTheme] = useState('light');

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}`,
            "jsx"
          ),
        ],
        daysAgo(1)
      ),
    ],
    totalTokens: { input: 120, output: 280 },
    bookmarked: false,
    tags: ["ui", "theming"],
  },
  "conv-003": {
    id: "conv-003",
    projectPath: "/Users/dev/projects/api-server",
    projectName: "api-server",
    startTime: daysAgo(2),
    lastTime: daysAgo(2),
    messages: [
      createMessage(
        "msg-020",
        "user",
        [createTextBlock("Fix the database connection timeout issue")],
        daysAgo(2)
      ),
      createMessage(
        "msg-021",
        "assistant",
        [
          createTextBlock(
            "Database connection timeouts usually occur due to pool exhaustion or network issues. Let me help you diagnose and fix this."
          ),
          createTextBlock("First, let's check your connection pool settings:"),
          createCodeBlock(
            `const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
  max: 20,                 // Increase pool size
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 5000,
});`,
            "javascript"
          ),
        ],
        daysAgo(2)
      ),
    ],
    totalTokens: { input: 80, output: 150 },
    bookmarked: false,
    tags: ["database", "debugging"],
  },
  "conv-004": {
    id: "conv-004",
    projectPath: "/Users/dev/projects/mobile-app",
    projectName: "mobile-app",
    startTime: daysAgo(3),
    lastTime: daysAgo(3),
    messages: [
      createMessage(
        "msg-030",
        "user",
        [createTextBlock("Implement push notifications for iOS and Android")],
        daysAgo(3)
      ),
      createMessage(
        "msg-031",
        "assistant",
        [
          createTextBlock(
            "I'll guide you through implementing push notifications using Firebase Cloud Messaging (FCM) for both platforms."
          ),
        ],
        daysAgo(3)
      ),
    ],
    totalTokens: { input: 60, output: 100 },
    bookmarked: true,
    tags: ["mobile", "notifications"],
  },
  "conv-005": {
    id: "conv-005",
    projectPath: "/Users/dev/projects/web-app",
    projectName: "web-app",
    startTime: daysAgo(5),
    lastTime: daysAgo(5),
    messages: [
      createMessage(
        "msg-040",
        "user",
        [createTextBlock("Optimize the webpack build configuration for production")],
        daysAgo(5)
      ),
      createMessage(
        "msg-041",
        "assistant",
        [createTextBlock("Here are the key optimizations for your webpack production build...")],
        daysAgo(5)
      ),
    ],
    totalTokens: { input: 50, output: 80 },
    bookmarked: false,
    tags: ["webpack", "optimization"],
  },
};

// ========== Search Results ==========

export function createSearchResults(query: string): SearchResult[] {
  const lowerQuery = query.toLowerCase();
  const results: SearchResult[] = [];

  // Search through conversations
  Object.values(mockConversations).forEach((conv) => {
    let matchCount = 0;
    let snippet = "";

    conv.messages.forEach((msg) => {
      msg.content.forEach((block) => {
        if (block.content.toLowerCase().includes(lowerQuery)) {
          matchCount++;
          if (!snippet) {
            const idx = block.content.toLowerCase().indexOf(lowerQuery);
            const start = Math.max(0, idx - 30);
            const end = Math.min(block.content.length, idx + query.length + 30);
            snippet = block.content.slice(start, end);
          }
        }
      });
    });

    if (matchCount > 0) {
      results.push({
        conversationId: conv.id,
        snippet,
        matchCount,
        rank: matchCount,
      });
    }
  });

  return results.sort((a, b) => b.matchCount - a.matchCount);
}
