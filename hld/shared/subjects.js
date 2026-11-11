// The subjects ("chapters") the Quizzes page opens with. Each topic (a series of quizzes, a single quiz, or "mine") belongs to exactly one subject;
// subjects hold topics, topics hold quizzes. Java, Python and language-independent topics are never put in the same topic.
export const SUBJECTS = [
  { id: 'java', icon: '☕', title: 'Java and Spring Boot', description: 'OOP, collections, multithreading, Spring Boot, JPA, DSA with Java code, LangChain4j and more.', match: /^java-/ },
  { id: 'python', icon: '🐍', title: 'Python and FastAPI', description: 'OOP, asyncio, FastAPI, SQLAlchemy, DSA with Python code, LangChain and LangGraph and more.', match: /^python-/ },
  { id: 'dsa', icon: '🧮', title: 'Data structures and algorithms', description: 'Complexity, patterns and puzzles in pseudocode: hashing, graphs, DP, greedy, bits, mock-interview habits.', match: /^core-dsa-/ },
  { id: 'databases', icon: '🗄️', title: 'SQL and databases', description: 'SQL, indexes, isolation, PostgreSQL, Redis, MongoDB, consistency models and scaling.', match: /^(core-sql|core-databases|postgres-system-design|redis-system-design)$/ },
  { id: 'messaging', icon: '📨', title: 'Messaging and streaming', description: 'Kafka, queues, delivery guarantees, outbox, sagas.', match: /^(core-messaging|kafka-system-design)$/ },
  { id: 'system-design', icon: '🏗️', title: 'System design', description: 'The design framework, estimation, classic designs, scaling reads and writes.', match: /^(core-system-design|quiz-scaling-reads|scaling-writes-pattern)$/ },
  { id: 'backend', icon: '⚙️', title: 'Backend engineering concepts', description: 'HTTP and APIs, concurrency theory, design patterns, resilience, observability, security, testing.', match: /^core-(web-and-data-concepts|concurrency-theory|design-patterns|resilience|observability|security|testing)$/ },
  { id: 'ai', icon: '🤖', title: 'AI engineering', description: 'LLM basics, RAG, evals and agent safety, without tying you to one library.', match: /^core-(llm-basics|rag|llm-evals|agent-safety)$/ },
  { id: 'career', icon: '💼', title: 'Behavioural and company prep', description: 'STAR stories, leadership principles and company-domain knowledge.', match: /^core-(behavioural|company-domains)$/ },
  { id: 'mine', icon: '✨', title: 'My generated quizzes', description: 'Quizzes you created with AI. Saved in this browser.', match: /^mine$/ },
  { id: 'other', icon: '📚', title: 'More quizzes', description: 'Everything else.', match: /^$/ },
];

export function subjectIdForTopic(topicId) {
  const s = SUBJECTS.find((x) => x.id !== 'other' && x.match.test(topicId));
  return s ? s.id : 'other';
}

// Learning order of the topics inside a subject (anything not listed comes last, alphabetically).
const TOPIC_ORDER = [
  'java-oop', 'java-collections-generics', 'java-exceptions-streams', 'java-strings-memory', 'java-design-patterns', 'java-low-level-design', 'java-multithreading',
  'java-dsa-fundamentals', 'java-dsa-advanced', 'java-spring-boot', 'java-data-access', 'java-messaging-resilience', 'java-observability-security-testing',
  'java-langchain4j-spring-ai', 'java-rag-evals-safety',
  'python-oop', 'python-collections-typing', 'python-exceptions-generators', 'python-strings-memory', 'python-design-patterns', 'python-low-level-design', 'python-multithreading-asyncio',
  'python-dsa-fundamentals', 'python-dsa-advanced', 'python-fastapi', 'python-data-access', 'python-messaging-resilience', 'python-observability-security-testing',
  'python-langchain-langgraph', 'python-rag-evals-safety',
  'core-dsa-fundamentals', 'core-dsa-advanced',
  'core-sql', 'postgres-system-design', 'redis-system-design', 'core-databases',
  'core-messaging', 'kafka-system-design',
  'core-system-design', 'quiz-scaling-reads', 'scaling-writes-pattern',
  'core-web-and-data-concepts', 'core-concurrency-theory', 'core-design-patterns', 'core-resilience', 'core-observability', 'core-security', 'core-testing',
  'core-llm-basics', 'core-rag', 'core-llm-evals', 'core-agent-safety',
  'core-behavioural', 'core-company-domains',
];
export const topicRank = (topicId) => { const i = TOPIC_ORDER.indexOf(topicId); return i === -1 ? TOPIC_ORDER.length : i; };
